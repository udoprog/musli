//! End-to-end tests for the `tokio-tungstenite` client.
//!
//! Every test spins up a real [`axum`] server bound to a random port on
//! `127.0.0.1` and talks to it over a real websocket connection.

#![cfg(all(feature = "tungstenite029", feature = "axum08"))]

use std::net::SocketAddr;
use std::time::Duration;

use axum08 as axum;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::any;
use musli_web::api::ChannelId;
use musli_web::client::{self, Handle, State as ClientState};
use musli_web::{tungstenite029, ws};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::task::JoinHandle;

/// The upper bound for how long any individual step of a test is allowed to
/// take. Every step is expected to complete in a fraction of this.
const TIMEOUT: Duration = Duration::from_secs(20);

/// Await a future, panicking if it takes longer than [`TIMEOUT`].
macro_rules! timeout {
    ($expr:expr) => {
        match tokio::time::timeout(TIMEOUT, $expr).await {
            Ok(value) => value,
            Err(..) => panic!("Timed out waiting for `{}`", stringify!($expr)),
        }
    };
}

mod api {
    use musli::{Decode, Encode};
    use musli_web::api::{self, ChannelId};

    #[derive(Debug, Encode, Decode)]
    pub struct HelloRequest<'de> {
        pub message: &'de str,
    }

    #[derive(Debug, Encode, Decode)]
    pub struct HelloResponse<'de> {
        pub message: &'de str,
        pub channel: ChannelId,
    }

    #[derive(Debug, Encode, Decode)]
    pub struct FailRequest<'de> {
        pub reason: &'de str,
    }

    /// A body which carries nothing.
    #[derive(Debug, Encode, Decode)]
    pub struct Empty;

    /// A request which the server never responds to.
    #[derive(Debug, Encode, Decode)]
    pub struct HangRequest;

    #[derive(Debug, Encode, Decode)]
    pub struct TickEvent<'de> {
        pub message: &'de str,
        pub tick: u32,
        pub channel: ChannelId,
    }

    api::define! {
        /// An endpoint which echoes the message it is given.
        pub type Hello;

        impl Endpoint for Hello {
            impl<'de> Request for HelloRequest<'de>;
            type Response<'de> = HelloResponse<'de>;
        }

        /// An endpoint which always fails with the reason it is given.
        pub type Fail;

        impl Endpoint for Fail {
            impl<'de> Request for FailRequest<'de>;
            type Response<'de> = Empty;
        }

        /// An endpoint which the server never responds to.
        pub type Hang;

        impl Endpoint for Hang {
            impl Request for HangRequest;
            type Response<'de> = Empty;
        }

        /// An endpoint which the server does not support.
        pub type Unhandled;

        impl Endpoint for Unhandled {
            impl Request for Empty;
            type Response<'de> = Empty;
        }

        /// A broadcast which is emitted by the test server on demand.
        pub type Tick;

        impl Broadcast for Tick {
            impl<'de> Event for TickEvent<'de>;
        }
    }
}

/// An event observed by the server side handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    OpenChannel(ChannelId),
    CloseChannel(ChannelId),
}

/// A tick which the test server should broadcast.
#[derive(Debug, Clone, Copy)]
struct Tick {
    tick: u32,
    channel: ChannelId,
}

#[derive(Clone)]
struct TestHandler {
    events: mpsc::UnboundedSender<Event>,
}

impl ws::Handler for TestHandler {
    type Id = api::Request;
    type Response = Result<bool, String>;

    async fn open_channel(&self, channel: ChannelId) {
        _ = self.events.send(Event::OpenChannel(channel));
    }

    async fn close_channel(&self, channel: ChannelId) {
        _ = self.events.send(Event::CloseChannel(channel));
    }

    async fn handle(
        &self,
        id: api::Request,
        incoming: &mut ws::Incoming<'_>,
        outgoing: &mut ws::Outgoing<'_>,
    ) -> Self::Response {
        match id {
            api::Request::Hello => {
                let Some(request) = incoming.read::<api::HelloRequest<'_>>() else {
                    return Ok(false);
                };

                outgoing.write(api::HelloResponse {
                    message: request.message,
                    channel: incoming.channel(),
                });

                Ok(true)
            }
            api::Request::Fail => {
                let Some(request) = incoming.read::<api::FailRequest<'_>>() else {
                    return Ok(false);
                };

                Err(request.reason.to_owned())
            }
            api::Request::Hang => {
                // Never completes, so the request stays pending until the
                // connection is torn down.
                core::future::pending::<()>().await;
                Ok(true)
            }
            // Deliberately reported as unsupported by the server.
            api::Request::Unhandled => Ok(false),
            api::Request::Unknown(..) => Ok(false),
        }
    }
}

#[derive(Clone)]
struct AppState {
    ticks: broadcast::Sender<Tick>,
    shutdown: watch::Receiver<bool>,
    handler: TestHandler,
}

async fn upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| connection(socket, state))
}

async fn connection(socket: WebSocket, mut state: AppState) {
    let mut ticks = state.ticks.subscribe();
    let mut server = musli_web::axum08::server(socket, state.handler.clone());

    if *state.shutdown.borrow_and_update() {
        return;
    }

    loop {
        tokio::select! {
            _ = state.shutdown.changed() => {
                break;
            }
            tick = ticks.recv() => {
                let Ok(Tick { tick, channel }) = tick else {
                    continue;
                };

                let event = api::TickEvent {
                    message: "tick",
                    tick,
                    channel,
                };

                if let Err(error) = server.broadcast_in(event, channel) {
                    panic!("Broadcast failed: {error}");
                }
            }
            result = server.run() => {
                if let Err(error) = result {
                    tracing::error!("Server error: {error}");
                }

                break;
            }
        }
    }
}

/// A test server bound to a local port.
struct TestServer {
    addr: SocketAddr,
    ticks: broadcast::Sender<Tick>,
    events: mpsc::UnboundedReceiver<Event>,
    shutdown: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl TestServer {
    /// Bind a server to a random port on localhost.
    async fn new() -> Self {
        Self::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await
    }

    /// Bind a server to the specified address.
    async fn bind(addr: SocketAddr) -> Self {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        let addr = listener.local_addr().expect("Missing local address");

        let (ticks, _) = broadcast::channel(64);
        let (events_tx, events) = mpsc::unbounded_channel();
        let (shutdown, shutdown_rx) = watch::channel(false);

        let state = AppState {
            ticks: ticks.clone(),
            shutdown: shutdown_rx.clone(),
            handler: TestHandler { events: events_tx },
        };

        let app = Router::new().route("/ws", any(upgrade).with_state(state));

        let task = tokio::spawn(async move {
            let mut shutdown_rx = shutdown_rx;

            let result = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    while !*shutdown_rx.borrow_and_update() {
                        if shutdown_rx.changed().await.is_err() {
                            break;
                        }
                    }
                })
                .await;

            if let Err(error) = result {
                tracing::error!("Serve error: {error}");
            }
        });

        Self {
            addr,
            ticks,
            events,
            shutdown,
            task,
        }
    }

    /// The address the server is bound to.
    fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// The websocket URL of the server.
    fn url(&self) -> String {
        format!("ws://{}/ws", self.addr)
    }

    /// Broadcast a tick to every connected client.
    fn tick(&self, tick: u32) {
        self.tick_in(tick, ChannelId::NONE);
    }

    /// Broadcast a tick over the specified channel.
    fn tick_in(&self, tick: u32, channel: ChannelId) {
        _ = self.ticks.send(Tick { tick, channel });
    }

    /// Receive the next handler event.
    async fn event(&mut self) -> Event {
        timeout!(self.events.recv()).expect("Server handler is gone")
    }

    /// Shut the server down, releasing the port it is bound to.
    async fn shutdown(self) {
        _ = self.shutdown.send(true);
        timeout!(self.task).expect("Server task panicked");
    }
}

/// A test client driving a service in the background.
struct TestClient {
    handle: Handle,
    task: JoinHandle<()>,
}

impl TestClient {
    /// Connect a client to the specified url and drive it in the background.
    fn new(url: &str) -> Self {
        Self::builder(url, true)
    }

    fn builder(url: &str, reconnect: bool) -> Self {
        let mut service = tungstenite029::connect(url)
            .reconnect(reconnect)
            .on_error(|error: client::Error| {
                tracing::debug!("Client error: {error}");
            })
            .build();

        let handle = service.handle().clone();

        let task = tokio::spawn(async move {
            service.run().await.expect("Client service failed");
        });

        Self { handle, task }
    }

    /// Wait until the client is connected.
    async fn open(&self) -> &Handle {
        timeout!(self.handle.wait_until_open()).expect("Failed to open connection");
        &self.handle
    }

    /// Close the client and wait for the service to shut down.
    async fn close(self) {
        self.handle.close();
        timeout!(self.task).expect("Client task panicked");
    }
}

/// Send a hello request and return the echoed message.
async fn hello(handle: &Handle, message: &str) -> (String, ChannelId) {
    let packet = timeout!(handle.request().body(api::HelloRequest { message }).send())
        .expect("Hello request failed");

    let response = packet.decode().expect("Failed to decode response");
    (response.message.to_owned(), response.channel)
}

#[tokio::test]
async fn request_response() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let (message, channel) = hello(handle, "Hello!").await;
    assert_eq!(message, "Hello!");
    assert_eq!(channel, ChannelId::NONE);

    // The same handle can be used for any number of requests.
    let (message, _) = hello(handle, "Again!").await;
    assert_eq!(message, "Again!");

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn packet_is_consumed_once_decoded() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let packet = timeout!(
        handle
            .request()
            .body(api::HelloRequest { message: "Hello!" })
            .send()
    )
    .expect("Hello request failed");

    assert!(!packet.is_empty());
    assert_eq!(packet.id(), <api::Hello as musli_web::api::Endpoint>::ID);
    assert!(packet.remaining() > 0);

    let response = packet.decode().expect("Failed to decode response");
    assert_eq!(response.message, "Hello!");

    // Everything has been consumed, so a second decode has nothing to read.
    assert!(packet.is_empty());
    assert_eq!(packet.remaining(), 0);
    assert!(packet.decode().is_err());

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn raw_packet() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let packet = timeout!(
        handle
            .request()
            .body(api::HelloRequest { message: "Raw!" })
            .send_raw()
    )
    .expect("Hello request failed");

    assert_eq!(packet.channel(), ChannelId::NONE);
    assert!(!packet.as_slice().is_empty());

    let response = packet
        .decode::<api::HelloResponse<'_>>()
        .expect("Failed to decode response");

    assert_eq!(response.message, "Raw!");

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn server_error_is_propagated() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let error = timeout!(
        handle
            .request()
            .body(api::FailRequest {
                reason: "not today"
            })
            .send()
    )
    .expect_err("Expected the request to fail");

    assert_eq!(error.as_server_error(), Some("not today"));

    // The connection is still usable after an error.
    let (message, _) = hello(handle, "Still here").await;
    assert_eq!(message, "Still here");

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn unsupported_endpoint() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let error = timeout!(handle.request().body(api::Empty).send())
        .expect_err("Expected the request to fail");

    let message = error
        .as_server_error()
        .expect("Expected a server reported error");

    assert!(
        message.contains("No support for request"),
        "Unexpected error message: {message}"
    );

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn request_while_not_connected() {
    let server = TestServer::new().await;

    // NB: The service is deliberately never driven, so the connection never
    // opens.
    let service = tungstenite029::connect(server.url()).build();
    let handle = service.handle().clone();

    assert_eq!(handle.state(), ClientState::Closed);
    assert!(!handle.is_open());

    let error = handle
        .request()
        .body(api::HelloRequest { message: "Hello!" })
        .send()
        .await
        .expect_err("Expected the request to fail");

    assert!(error.is_not_connected());

    let error = handle
        .channel()
        .await
        .expect_err("Expected opening a channel to fail");

    assert!(error.is_not_connected());

    // Dropping the service invalidates every handle associated with it.
    drop(service);
    assert!(handle.wait_until_open().await.is_err());

    server.shutdown().await;
}

#[tokio::test]
async fn state_transitions() {
    let server = TestServer::new().await;
    let client = TestClient::builder(&server.url(), false);

    let mut states = client.handle.on_state_change();
    assert_eq!(states.state(), ClientState::Closed);

    assert!(timeout!(states.wait_until(ClientState::Open)));
    assert_eq!(client.handle.state(), ClientState::Open);
    assert!(client.handle.is_open());

    server.shutdown().await;

    assert!(timeout!(states.wait_until(ClientState::Closed)));
    assert!(!client.handle.is_open());

    // Reconnecting is disabled, so losing the connection ends the service.
    timeout!(client.task).expect("Client task panicked");
}

#[tokio::test]
async fn broadcasts() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let mut first = handle.on_broadcast::<api::Tick>();
    let mut second = handle.on_broadcast::<api::Tick>();

    server.tick(1);

    for listener in [&mut first, &mut second] {
        let packet = timeout!(listener.recv())
            .expect("Listener is closed")
            .expect("Broadcast failed");

        let event = packet.decode_event().expect("Failed to decode event");
        assert_eq!(event.tick, 1);
        assert_eq!(event.message, "tick");
        assert_eq!(packet.channel(), ChannelId::NONE);
    }

    // Once cleared a listener no longer observes broadcasts.
    second.clear();
    server.tick(2);

    let packet = timeout!(first.recv())
        .expect("Listener is closed")
        .expect("Broadcast failed");

    assert_eq!(
        packet.decode_event().expect("Failed to decode event").tick,
        2
    );

    assert!(
        timeout!(second.recv()).is_none(),
        "A cleared listener must not observe broadcasts"
    );

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn broadcast_listener_closes_with_service() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let mut listener = handle.on_broadcast::<api::Tick>();

    client.close().await;

    assert!(
        timeout!(listener.recv()).is_none(),
        "Listener must be closed once the service shuts down"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn channels() {
    let mut server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let channel = timeout!(handle.channel()).expect("Failed to open channel");
    assert_ne!(channel.id(), ChannelId::NONE);

    assert_eq!(server.event().await, Event::OpenChannel(channel.id()));

    // Requests over a channel are tagged with the channel on both sides.
    let packet = timeout!(
        channel
            .request()
            .body(api::HelloRequest {
                message: "Over a channel"
            })
            .send()
    )
    .expect("Hello request failed");

    assert_eq!(packet.channel(), channel.id());

    let response = packet.decode().expect("Failed to decode response");
    assert_eq!(response.message, "Over a channel");
    assert_eq!(response.channel, channel.id());

    // A second channel is distinct from the first.
    let other = timeout!(handle.channel()).expect("Failed to open channel");
    assert_ne!(other.id(), channel.id());
    assert_eq!(server.event().await, Event::OpenChannel(other.id()));

    // Requests over the handle itself are not associated with any channel.
    let (_, channel_id) = hello(handle, "No channel").await;
    assert_eq!(channel_id, ChannelId::NONE);

    let id = channel.id();
    drop(channel);
    assert_eq!(server.event().await, Event::CloseChannel(id));

    let other_id = other.id();
    drop(other);
    assert_eq!(server.event().await, Event::CloseChannel(other_id));

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn broadcast_over_channel() {
    let mut server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let channel = timeout!(handle.channel()).expect("Failed to open channel");
    assert_eq!(server.event().await, Event::OpenChannel(channel.id()));

    let mut listener = channel.handle().on_broadcast::<api::Tick>();

    server.tick_in(42, channel.id());

    let packet = timeout!(listener.recv())
        .expect("Listener is closed")
        .expect("Broadcast failed");

    assert_eq!(packet.channel(), channel.id());

    let event = packet.decode_event().expect("Failed to decode event");
    assert_eq!(event.tick, 42);
    assert_eq!(event.channel, channel.id());

    client.close().await;
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_requests() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await.clone();

    let mut tasks = Vec::new();

    for n in 0..64u32 {
        let handle = handle.clone();

        tasks.push(tokio::spawn(async move {
            let message = format!("message-{n}");
            let (echoed, _) = hello(&handle, &message).await;
            assert_eq!(
                echoed, message,
                "Response was correlated with the wrong request"
            );
        }));
    }

    for task in tasks {
        timeout!(task).expect("Request task panicked");
    }

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn reconnects_when_the_server_comes_back() {
    let server = TestServer::new().await;
    let addr = server.addr();

    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    let (message, _) = hello(handle, "Before").await;
    assert_eq!(message, "Before");

    let mut states = handle.on_state_change();

    server.shutdown().await;
    assert!(timeout!(states.wait_until(ClientState::Closed)));

    // Requests performed while disconnected fail immediately.
    let error = handle
        .request()
        .body(api::HelloRequest { message: "Nope" })
        .send()
        .await
        .expect_err("Expected the request to fail while disconnected");

    assert!(error.is_not_connected());

    // Bring the server back up on the same address.
    let server = TestServer::bind(addr).await;
    assert!(timeout!(states.wait_until(ClientState::Open)));

    let (message, _) = hello(handle, "After").await;
    assert_eq!(message, "After");

    client.close().await;
    server.shutdown().await;
}

/// The server pings every 10 seconds and closes any connection which has not
/// answered with a matching pong within 30 seconds. This verifies that the
/// client keeps answering those pings while it is otherwise idle.
///
/// This is deliberately slower than every other test, since the timeouts it
/// exercises are baked into the server.
#[tokio::test]
async fn keepalive_survives_the_server_close_timeout() {
    let server = TestServer::new().await;

    // NB: Reconnecting is disabled so that a dropped connection is observable
    // rather than being silently repaired.
    let client = TestClient::builder(&server.url(), false);
    let handle = client.open().await;

    tokio::time::sleep(Duration::from_secs(35)).await;

    assert_eq!(
        handle.state(),
        ClientState::Open,
        "The connection must have been kept alive by answering pings"
    );

    let (message, _) = hello(handle, "Still alive").await;
    assert_eq!(message, "Still alive");

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn pending_requests_fail_when_the_connection_is_lost() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await.clone();

    let mut states = handle.on_state_change();

    // Keep a request in flight while the server goes away. The handler for
    // `Hang` never completes, so the only way this resolves is by the
    // connection being torn down.
    let request = tokio::spawn(async move {
        handle
            .request()
            .body(api::HangRequest)
            .send()
            .await
            .map(|_| ())
    });

    server.shutdown().await;
    assert!(timeout!(states.wait_until(ClientState::Closed)));

    let error = timeout!(request)
        .expect("Request task panicked")
        .expect_err("Pending request must be failed");

    assert_eq!(
        error.as_server_error(),
        None,
        "The failure must come from the connection being lost, not the server"
    );

    client.close().await;
}

#[tokio::test]
async fn close_stops_the_service() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());

    client.open().await;

    let handle = client.handle.clone();
    handle.close();

    timeout!(client.task).expect("Client task panicked");
    assert!(!handle.is_open());

    server.shutdown().await;
}

#[tokio::test]
async fn multiple_clients() {
    let server = TestServer::new().await;

    let first = TestClient::new(&server.url());
    let second = TestClient::new(&server.url());

    let first_handle = first.open().await;
    let second_handle = second.open().await;

    let mut first_listener = first_handle.on_broadcast::<api::Tick>();
    let mut second_listener = second_handle.on_broadcast::<api::Tick>();

    let (message, _) = hello(first_handle, "first").await;
    assert_eq!(message, "first");

    let (message, _) = hello(second_handle, "second").await;
    assert_eq!(message, "second");

    server.tick(7);

    for listener in [&mut first_listener, &mut second_listener] {
        let packet = timeout!(listener.recv())
            .expect("Listener is closed")
            .expect("Broadcast failed");

        assert_eq!(
            packet.decode_event().expect("Failed to decode event").tick,
            7
        );
    }

    first.close().await;
    second.close().await;
    server.shutdown().await;
}
