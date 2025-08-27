//! Example musli-web server based on [`axum`].
//!
//! Run with:
//!
//! ```sh
//! cargo run
//! ```
//!
//! [`axum`]: https://docs.rs/axum

use std::error::Error;
use std::pin::pin;

use anyhow::Result;
use axum::Router;
use axum::extract::State;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::any;
use musli_web::api::Endpoint;
use musli_web::{axum08, ws};
use tokio::sync::broadcast::Sender;
use tokio::time::{self, Duration};

#[derive(Debug, Clone)]
enum Broadcast {
    Tick { tick: u32 },
}

struct MyHandler;

impl ws::Handler for MyHandler {
    type Response = bool;

    async fn handle(
        &mut self,
        kind: &str,
        incoming: &mut ws::Incoming<'_>,
        outgoing: &mut ws::Outgoing<'_>,
    ) -> Self::Response {
        tracing::info!("Handling: {kind}");

        match kind {
            api::Hello::KIND => {
                let Some(request) = incoming.read::<api::HelloRequest<'_>>() else {
                    return false;
                };

                outgoing.write(api::HelloResponse {
                    message: request.message,
                });

                outgoing.write(api::HelloResponse {
                    message: request.message.to_uppercase().as_str(),
                });

                true
            }
            _ => false,
        }
    }
}

async fn handler(ws: WebSocketUpgrade, State(sender): State<Sender<Broadcast>>) -> Response {
    ws.on_upgrade(move |socket: WebSocket| async move {
        let mut subscribe = sender.subscribe();

        let mut server = pin!(axum08::server(socket, MyHandler));

        loop {
            tokio::select! {
                m = subscribe.recv() => {
                    let Ok(message) = m else {
                        continue;
                    };

                    let result = match message {
                        Broadcast::Tick { tick } => {
                            server.as_mut().broadcast(api::TickEvent { message: "tick", tick })
                        },
                    };

                    if let Err(error) = result {
                        tracing::error!("Broadcast failed: {error}");

                        let mut error = error.source();

                        while let Some(e) = error.take() {
                            tracing::error!("Caused by: {e}");
                            error = e.source();
                        }
                    }
                }
                result = server.as_mut().run() => {
                    if let Err(error) = result {
                        tracing::error!("Websocket error: {error}");

                        let mut error = error.source();

                        while let Some(e) = error.take() {
                            tracing::error!("Caused by: {e}");
                            error = e.source();
                        }
                    }

                    break;
                }
            }
        }
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (sender, _) = tokio::sync::broadcast::channel::<Broadcast>(1024);

    let app = Router::new().route("/ws", any(handler).with_state(sender.clone()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    let mut serve = pin!(axum::serve(listener, app).into_future());
    let mut interval = time::interval(Duration::from_secs(1));
    let mut tick = 0;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                _ = sender.send(Broadcast::Tick { tick });
                tick = tick.wrapping_add(1);
            },
            result = serve.as_mut() => {
                result?;
                break;
            }
        }
    }

    Ok(())
}
