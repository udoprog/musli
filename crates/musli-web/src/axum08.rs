//! The server implementation for [axum].
//!
//! Use [`server()`] to set up the server and feed it incoming requests.
//!
//! [axum]: <https://docs.rs/axum>

use core::pin::Pin;
use core::task::Poll;
use core::task::{Context, ready};

use bytes::Bytes;

use axum_core05::Error;
use axum08::extract::ws::{CloseFrame, Message, WebSocket};

use crate::ws::{self, Handler, Server, ServerImplementation, Socket};

/// Construct a new axum server with the specified handler.
///
/// # Examples
///
/// ```
/// # extern crate axum08 as axum;
/// use std::error::Error;
/// use std::pin::pin;
///
/// use axum::Router;
/// use axum::extract::State;
/// use axum::extract::ws::{WebSocket, WebSocketUpgrade};
/// use axum::response::Response;
/// use axum::routing::any;
/// use tokio::sync::broadcast::Sender;
/// use tokio::time::{self, Duration};
///
/// use musli_web::api::Endpoint;
/// use musli_web::axum08;
/// use musli_web::ws;
///
/// mod api {
///     use musli::{Decode, Encode};
///     use musli_web::api;
///
///     #[derive(Encode, Decode)]
///     pub struct HelloRequest<'de> {
///         pub message: &'de str,
///     }
///
///     #[derive(Encode, Decode)]
///     pub struct HelloResponse<'de> {
///         pub message: &'de str,
///     }
///
///     #[derive(Encode, Decode)]
///     pub struct TickEvent<'de> {
///         pub message: &'de str,
///         pub tick: u32,
///     }
///
///     api::define! {
///         endpoint Hello {
///             request<'de> = HelloRequest<'de>;
///             response<'de> = HelloResponse<'de>;
///         }
///
///         broadcast Tick {
///             body<'de> = TickEvent<'de>;
///         }
///     }
/// }
///
/// #[derive(Debug, Clone)]
/// enum Broadcast {
///     Tick { tick: u32 },
/// }
///
/// struct MyHandler;
///
/// impl ws::Handler for MyHandler {
///     type Error = &'static str;
///
///     async fn handle(
///         &mut self,
///         kind: &str,
///         incoming: &mut ws::Incoming<'_>,
///         outgoing: &mut ws::Outgoing<'_>,
///     ) -> Result<(), Self::Error> {
///         tracing::info!("Handling: {kind}");
///
///         match kind {
///             api::Hello::KIND => {
///                 let Some(request) = incoming.read::<api::HelloRequest<'_>>() else {
///                     return Ok(());
///                 };
///
///                 outgoing.write(api::HelloResponse {
///                     message: request.message,
///                 });
///             }
///             _ => {}
///         }
///
///         Ok(())
///     }
/// }
///
/// async fn handler(ws: WebSocketUpgrade, State(sender): State<Sender<Broadcast>>) -> Response {
///     ws.on_upgrade(move |socket: WebSocket| async move {
///         let mut subscribe = sender.subscribe();
///
///         let mut server = pin!(axum08::server(socket, MyHandler));
///
///         loop {
///             tokio::select! {
///                 m = subscribe.recv() => {
///                     let Ok(message) = m else {
///                         continue;
///                     };
///
///                     let result = match message {
///                         Broadcast::Tick { tick } => {
///                             server.as_mut().broadcast(api::TickEvent { message: "tick", tick })
///                         },
///                     };
///
///                     if let Err(error) = result {
///                         tracing::error!("Broadcast failed: {error}");
///
///                         let mut error = error.source();
///
///                         while let Some(e) = error.take() {
///                             tracing::error!("Caused by: {e}");
///                             error = e.source();
///                         }
///                     }
///                 }
///                 result = server.as_mut().run() => {
///                     if let Err(error) = result {
///                         tracing::error!("Websocket error: {error}");
///
///                         let mut error = error.source();
///
///                         while let Some(e) = error.take() {
///                             tracing::error!("Caused by: {e}");
///                             error = e.source();
///                         }
///                     }
///
///                     break;
///                 }
///             }
///         }
///     })
/// }
/// ```
#[inline]
pub fn server<H>(socket: WebSocket, handler: H) -> Server<AxumServer, H>
where
    H: Handler,
{
    Server::new(socket, handler)
}

/// Marker type used in combination with [`Server`] to indicate that the
/// implementation uses axum.
///
/// See [`server()`] for how this is constructed and used.
#[non_exhaustive]
pub enum AxumServer {}

impl Socket for WebSocket {
    type Message = Message;
    type Error = Error;

    #[inline]
    #[allow(private_interfaces)]
    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Result<ws::Message, Self::Error>>> {
        let Some(result) = ready!(futures_core03::Stream::poll_next(self, ctx)) else {
            return Poll::Ready(None);
        };

        let message = match result {
            Ok(message) => message,
            Err(err) => return Poll::Ready(Some(Err(err))),
        };

        let message = match message {
            Message::Text(..) => ws::Message::Text,
            Message::Binary(data) => ws::Message::Binary(data),
            Message::Ping(data) => ws::Message::Ping(data),
            Message::Pong(data) => ws::Message::Pong(data),
            Message::Close(..) => ws::Message::Close,
        };

        Poll::Ready(Some(Ok(message)))
    }

    #[inline]
    async fn send(&mut self, message: Self::Message) -> Result<(), Self::Error> {
        WebSocket::send(self, message).await
    }
}

impl ServerImplementation for AxumServer {
    type Error = Error;
    type Message = Message;
    type Socket = WebSocket;

    #[inline]
    fn ping(data: Bytes) -> Self::Message {
        Message::Ping(data)
    }

    #[inline]
    fn pong(data: Bytes) -> Self::Message {
        Message::Pong(data)
    }

    #[inline]
    fn binary(data: &[u8]) -> Self::Message {
        Message::Binary(Bytes::from(data.to_vec()))
    }

    #[inline]
    fn close(code: u16, reason: &str) -> Self::Message {
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))
    }
}
