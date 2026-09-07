//! End-to-end tests for the `tokio-tungstenite` client.
//!
//! Every test spins up a real [`axum`] server bound to a random port on
//! `127.0.0.1` and talks to it over a real websocket connection.
//!
//! Miri cannot open sockets under isolation, so these are skipped there.

#![cfg(all(feature = "tungstenite029", feature = "axum08", not(miri)))]

use std::net::SocketAddr;
use std::time::Duration;

use axum08 as axum;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::any;
use musli_web::api::{ChannelId, Format, MessageId, Mode};
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
        /// The format the server decoded the request with, so tests can observe
        /// what was actually negotiated from the other end.
        pub format: u8,
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
                    format: incoming.format().to_u8(),
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
    formats: Option<&'static [Format]>,
}

async fn upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| connection(socket, state))
}

async fn connection(socket: WebSocket, mut state: AppState) {
    let mut ticks = state.ticks.subscribe();
    let mut connect = musli_web::axum08::server(socket, state.handler.clone());

    if let Some(formats) = state.formats {
        connect = connect.with_formats(formats);
    }

    if *state.shutdown.borrow_and_update() {
        return;
    }

    // NB: Nothing can be broadcast until this resolves, which is exactly the
    // guarantee the connection step exists to provide.
    let mut server = match connect.connect().await {
        Ok(server) => server,
        Err(error) => {
            tracing::debug!("Failed to negotiate connection: {error}");
            return;
        }
    };

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

    /// Bind a server which only accepts the given formats.
    async fn with_formats(formats: &'static [Format]) -> Self {
        Self::bind_with(SocketAddr::from(([127, 0, 0, 1], 0)), Some(formats)).await
    }

    /// Bind a server to the specified address.
    async fn bind(addr: SocketAddr) -> Self {
        Self::bind_with(addr, None).await
    }

    async fn bind_with(addr: SocketAddr, formats: Option<&'static [Format]>) -> Self {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        let addr = listener.local_addr().expect("Missing local address");

        let (ticks, _) = broadcast::channel(64);
        let (events_tx, events) = mpsc::unbounded_channel();
        let (shutdown, shutdown_rx) = watch::channel(false);

        let state = AppState {
            ticks: ticks.clone(),
            shutdown: shutdown_rx.clone(),
            handler: TestHandler { events: events_tx },
            formats,
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
    errors: mpsc::UnboundedReceiver<String>,
}

impl TestClient {
    /// Connect a client to the specified url and drive it in the background.
    fn new(url: &str) -> Self {
        Self::builder(url, true, Format::DEFAULT, Mode::DEFAULT)
    }

    /// Connect a client which asks for the specified format.
    fn with_format(url: &str, format: Format) -> Self {
        Self::builder(url, true, format, Mode::DEFAULT)
    }

    /// Connect a client which asks for the specified format and mode.
    fn with_mode(url: &str, format: Format, mode: Mode) -> Self {
        Self::builder(url, true, format, mode)
    }

    fn builder(url: &str, reconnect: bool, format: Format, mode: Mode) -> Self {
        let (errors_tx, errors) = mpsc::unbounded_channel();

        let mut service = tungstenite029::connect(url)
            .reconnect(reconnect)
            .format(format)
            .mode(mode)
            .on_error(move |error: client::Error| {
                tracing::debug!("Client error: {error}");
                _ = errors_tx.send(error.to_string());
            })
            .build();

        let handle = service.handle().clone();

        let task = tokio::spawn(async move {
            service.run().await.expect("Client service failed");
        });

        Self {
            handle,
            task,
            errors,
        }
    }

    /// Receive the next error reported by the client service.
    async fn error(&mut self) -> String {
        timeout!(self.errors.recv()).expect("Client service is gone")
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
    let (message, channel, _) = hello_full(handle, message).await;
    (message, channel)
}

/// Send a hello request, additionally reporting the format the server decoded
/// it with.
async fn hello_full(handle: &Handle, message: &str) -> (String, ChannelId, Format) {
    let packet = timeout!(handle.request().body(api::HelloRequest { message }).send())
        .expect("Hello request failed");

    let response = packet.decode().expect("Failed to decode response");

    let format = Format::from_u8(response.format).expect("Server reported an unknown format");

    // The response body must be encoded with the same format as the request.
    assert_eq!(packet.format(), format);

    (response.message.to_owned(), response.channel, format)
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
    let client = TestClient::builder(&server.url(), false, Format::DEFAULT, Mode::DEFAULT);

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
    let client = TestClient::builder(&server.url(), false, Format::DEFAULT, Mode::DEFAULT);
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

/// Exercise the paths that carry a body end to end using `format`.
///
/// This covers requests, responses, server errors and broadcasts, each of which
/// encodes a body and therefore depends on the negotiated format.
async fn exercise(format: Format, mode: Mode) {
    let mut server = TestServer::new().await;
    let client = TestClient::with_mode(&server.url(), format, mode);
    let handle = client.open().await;

    // The client reports the format and mode the server agreed to.
    assert_eq!(
        handle.format(),
        format,
        "client did not settle on `{format}`"
    );

    assert_eq!(handle.mode(), mode, "client did not settle on `{mode}`");

    // Requests and responses round-trip, and the server decoded the request
    // with the format the client asked for.
    let (message, channel, server_format) = hello_full(handle, "Hello!").await;
    assert_eq!(message, "Hello!");
    assert_eq!(channel, ChannelId::NONE);
    assert_eq!(server_format, format, "server used the wrong format");

    // Borrowed strings survive the round trip in every format.
    let (message, ..) = hello_full(handle, "a string with spaces").await;
    assert_eq!(message, "a string with spaces");

    // Error bodies are encoded with a format the client can read.
    let error = timeout!(
        handle
            .request()
            .body(api::FailRequest { reason: "nope" })
            .send()
    )
    .expect_err("Expected the request to fail");

    assert_eq!(error.as_server_error(), Some("nope"));

    // Broadcasts are encoded with the negotiated format, since the server
    // originates them without a request to take the format from.
    let mut listener = handle.on_broadcast::<api::Tick>();
    server.tick(11);

    let packet = timeout!(listener.recv())
        .expect("Listener is closed")
        .expect("Broadcast failed");

    assert_eq!(packet.format(), format, "broadcast used the wrong format");

    let event = packet.decode_event().expect("Failed to decode event");
    assert_eq!(event.tick, 11);
    assert_eq!(event.message, "tick");

    // Channels work the same regardless of format.
    let channel = timeout!(handle.channel()).expect("Failed to open channel");
    assert_eq!(server.event().await, Event::OpenChannel(channel.id()));

    let packet = timeout!(
        channel
            .request()
            .body(api::HelloRequest {
                message: "over a channel"
            })
            .send()
    )
    .expect("Hello request failed");

    assert_eq!(packet.channel(), channel.id());
    assert_eq!(packet.format(), format);

    let response = packet.decode().expect("Failed to decode response");
    assert_eq!(response.message, "over a channel");

    drop(channel);

    client.close().await;
    server.shutdown().await;
}

macro_rules! format_tests {
    ($($name:ident => $format:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let format = $format;

                assert!(
                    format.is_supported(),
                    "`{format}` must be built in for this test",
                );

                exercise(format, Mode::DEFAULT).await;
            }
        )*
    };
}

format_tests! {
    format_packed => Format::Packed,
    format_storage => Format::Storage,
    format_wire => Format::Wire,
    format_descriptive => Format::Descriptive,
    format_json => Format::Json,
}

/// Text mode has to carry everything binary mode does, since the only thing it
/// changes is how a frame is spelled.
#[tokio::test]
async fn text_mode_round_trip() {
    exercise(Format::Json, Mode::Text).await;
}

/// A text frame has to be valid UTF-8 in its entirety, so a format which is not
/// human readable cannot be carried in one and the connection settles on the
/// defaults instead.
#[tokio::test]
async fn text_mode_requires_a_human_readable_format() {
    let server = TestServer::new().await;
    let mut client = TestClient::with_mode(&server.url(), Format::Wire, Mode::Text);

    let error = client.error().await;

    assert!(
        error.contains("rejected format `wire` in `text` mode"),
        "Unexpected error: {error}"
    );

    // Falling back leaves the connection fully usable, in binary mode.
    let handle = client.open().await;
    assert_eq!(handle.mode(), Mode::Binary);
    assert_eq!(handle.format(), Format::Wire);

    let (message, _, server_format) = hello_full(handle, "after fallback").await;
    assert_eq!(message, "after fallback");
    assert_eq!(server_format, Format::Wire);

    client.close().await;
    server.shutdown().await;
}

/// The mode is settled per connection, so a reconnect has to ask for it again
/// rather than assume the replacement server speaks it.
#[tokio::test]
async fn text_mode_survives_a_reconnect() {
    let server = TestServer::new().await;
    let addr = server.addr();

    let client = TestClient::with_mode(&server.url(), Format::Json, Mode::Text);
    let handle = client.open().await;

    assert_eq!(handle.mode(), Mode::Text);
    let (message, ..) = hello_full(handle, "before").await;
    assert_eq!(message, "before");

    let mut states = handle.on_state_change();

    server.shutdown().await;
    assert!(timeout!(states.wait_until(ClientState::Closed)));

    let server = TestServer::bind(addr).await;
    assert!(timeout!(states.wait_until(ClientState::Open)));

    assert_eq!(handle.mode(), Mode::Text);
    assert_eq!(handle.format(), Format::Json);

    let (message, ..) = hello_full(handle, "after").await;
    assert_eq!(message, "after");

    client.close().await;
    server.shutdown().await;
}

/// Two clients on one server may run in different modes, since the mode is a
/// property of the connection rather than of the server.
#[tokio::test]
async fn clients_may_use_different_modes_on_one_server() {
    let server = TestServer::new().await;

    let binary = TestClient::with_format(&server.url(), Format::Json);
    let text = TestClient::with_mode(&server.url(), Format::Json, Mode::Text);

    let binary_handle = binary.open().await;
    let text_handle = text.open().await;

    assert_eq!(binary_handle.mode(), Mode::Binary);
    assert_eq!(text_handle.mode(), Mode::Text);

    let (message, ..) = hello_full(binary_handle, "binary").await;
    assert_eq!(message, "binary");

    let (message, ..) = hello_full(text_handle, "text").await;
    assert_eq!(message, "text");

    binary.close().await;
    text.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn default_format_is_wire() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url());
    let handle = client.open().await;

    assert_eq!(handle.format(), Format::Wire);

    let (_, _, server_format) = hello_full(handle, "Hello!").await;
    assert_eq!(server_format, Format::Wire);

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn unsupported_format_falls_back_to_default() {
    // A server which refuses everything except the default.
    let server = TestServer::with_formats(&[Format::Wire]).await;
    let mut client = TestClient::with_format(&server.url(), Format::Json);

    let error = client.error().await;

    assert!(
        error.contains("rejected format `json`") && error.contains("falling back to `wire`"),
        "Unexpected error: {error}"
    );

    // Falling back leaves the connection fully usable.
    let handle = client.open().await;
    assert_eq!(handle.format(), Format::Wire);

    let (message, _, server_format) = hello_full(handle, "after fallback").await;
    assert_eq!(message, "after fallback");
    assert_eq!(server_format, Format::Wire);

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn server_reports_which_formats_it_accepts() {
    let server = TestServer::with_formats(&[Format::Wire, Format::Storage]).await;
    let mut client = TestClient::with_format(&server.url(), Format::Descriptive);

    let error = client.error().await;

    assert!(
        error.contains("`wire`") && error.contains("`storage`"),
        "The rejection must list what the server does accept: {error}"
    );

    assert!(
        !error.contains("`json`"),
        "The rejection must not list formats the server refuses: {error}"
    );

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn negotiation_survives_a_reconnect() {
    let server = TestServer::new().await;
    let addr = server.addr();

    let client = TestClient::with_format(&server.url(), Format::Json);
    let handle = client.open().await;

    let (_, _, server_format) = hello_full(handle, "before").await;
    assert_eq!(server_format, Format::Json);

    let mut states = handle.on_state_change();

    server.shutdown().await;
    assert!(timeout!(states.wait_until(ClientState::Closed)));

    // The replacement server is a fresh connection, so the format has to be
    // negotiated again rather than assumed.
    let server = TestServer::bind(addr).await;
    assert!(timeout!(states.wait_until(ClientState::Open)));

    assert_eq!(handle.format(), Format::Json);

    let (message, _, server_format) = hello_full(handle, "after").await;
    assert_eq!(message, "after");
    assert_eq!(server_format, Format::Json);

    client.close().await;
    server.shutdown().await;
}

#[tokio::test]
async fn clients_may_use_different_formats_on_one_server() {
    let server = TestServer::new().await;

    let json = TestClient::with_format(&server.url(), Format::Json);
    let packed = TestClient::with_format(&server.url(), Format::Packed);

    let json_handle = json.open().await;
    let packed_handle = packed.open().await;

    let (message, _, format) = hello_full(json_handle, "json client").await;
    assert_eq!(message, "json client");
    assert_eq!(format, Format::Json);

    let (message, _, format) = hello_full(packed_handle, "packed client").await;
    assert_eq!(message, "packed client");
    assert_eq!(format, Format::Packed);

    // Each connection broadcasts in its own negotiated format.
    let mut json_listener = json_handle.on_broadcast::<api::Tick>();
    let mut packed_listener = packed_handle.on_broadcast::<api::Tick>();

    server.tick(5);

    let packet = timeout!(json_listener.recv())
        .expect("Listener is closed")
        .expect("Broadcast failed");

    assert_eq!(packet.format(), Format::Json);
    assert_eq!(packet.decode_event().unwrap().tick, 5);

    let packet = timeout!(packed_listener.recv())
        .expect("Listener is closed")
        .expect("Broadcast failed");

    assert_eq!(packet.format(), Format::Packed);
    assert_eq!(packet.decode_event().unwrap().tick, 5);

    json.close().await;
    packed.close().await;
    server.shutdown().await;
}

/// A websocket client which speaks the wire protocol by hand.
///
/// The real client always negotiates before it does anything else, which is
/// precisely the behaviour the tests below need to violate in order to show
/// that the server refuses to play along.
mod raw {
    use core::future::poll_fn;
    use core::pin::Pin;
    use core::time::Duration;

    use bytes::Bytes;
    use futures_core03::Stream;
    use futures_sink03::Sink;
    use musli::reader::SliceReader;
    use musli_web::api::{ChannelId, Format, MessageId, RequestHeader, ResponseHeader};
    use tokio::net::TcpStream;

    use crate::TIMEOUT;
    use tokio_tungstenite029::tungstenite::{Message as WsMessage, Utf8Bytes};
    use tokio_tungstenite029::{MaybeTlsStream, WebSocketStream, connect_async};

    /// One binary frame received from the server, split into its fixed envelope
    /// and whatever body followed it.
    pub(crate) struct Frame {
        pub(crate) header: ResponseHeader,
        bytes: Bytes,
        at: usize,
    }

    impl Frame {
        /// The body of the frame, which is encoded with the format the envelope
        /// declares.
        pub(crate) fn body(&self) -> &[u8] {
            &self.bytes[self.at..]
        }

        /// The [`Format`] the body is encoded with.
        pub(crate) fn format(&self) -> Option<Format> {
            Format::from_u8(self.header.format)
        }
    }

    pub(crate) struct RawClient {
        socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    }

    impl RawClient {
        /// Open a websocket connection without saying anything over it.
        pub(crate) async fn connect(url: &str) -> Self {
            let (socket, _) = connect_async(url).await.expect("Failed to connect");
            Self { socket }
        }

        /// Send an envelope with no body.
        pub(crate) async fn send(&mut self, header: RequestHeader) {
            let mut data = Vec::new();
            musli::packed::encode(&mut data, &header).expect("Failed to encode envelope");
            self.send_raw(data).await;
        }

        /// Ask the server to use `format` for the rest of the connection.
        pub(crate) async fn negotiate(&mut self, serial: u32, format: Format) {
            self.send(RequestHeader {
                serial,
                id: MessageId::NEGOTIATE.get(),
                format: format.to_u8(),
                channel: ChannelId::NONE,
            })
            .await;
        }

        /// Ask the server to use `format` in text mode, by hand.
        ///
        /// Spelling the envelope out here rather than going through the crate
        /// is the point: it is what pins the shape a hand-written client has to
        /// produce.
        pub(crate) async fn negotiate_text(&mut self, serial: u32, format: Format) {
            let text = format!(
                "serial: {serial}\nid: {}\nformat: {}\nchannel: 0\n\n",
                MessageId::NEGOTIATE.get(),
                format.to_u8()
            );

            let message = WsMessage::Text(Utf8Bytes::from(&*text));

            poll_fn(|cx| Pin::new(&mut self.socket).poll_ready(cx))
                .await
                .expect("Socket is not ready");

            Pin::new(&mut self.socket)
                .start_send(message)
                .expect("Failed to send");

            poll_fn(|cx| Pin::new(&mut self.socket).poll_flush(cx))
                .await
                .expect("Failed to flush");
        }

        /// Receive the next text frame verbatim, ignoring keepalive traffic.
        pub(crate) async fn text_frame(&mut self) -> String {
            loop {
                let message = timeout!(self.next()).expect("Connection closed unexpectedly");

                match message {
                    WsMessage::Text(text) => return text.as_str().to_owned(),
                    WsMessage::Ping(..) | WsMessage::Pong(..) => continue,
                    message => panic!("Expected a text frame, got {message:?}"),
                }
            }
        }

        async fn send_raw(&mut self, data: Vec<u8>) {
            let message = WsMessage::Binary(Bytes::from(data));

            poll_fn(|cx| Pin::new(&mut self.socket).poll_ready(cx))
                .await
                .expect("Socket is not ready");

            Pin::new(&mut self.socket)
                .start_send(message)
                .expect("Failed to send");

            poll_fn(|cx| Pin::new(&mut self.socket).poll_flush(cx))
                .await
                .expect("Failed to flush");
        }

        /// Receive the next message, whatever it is.
        async fn next(&mut self) -> Option<WsMessage> {
            let message = poll_fn(|cx| Pin::new(&mut self.socket).poll_next(cx)).await?;
            Some(message.expect("Socket error"))
        }

        /// Receive the next frame, ignoring keepalive traffic.
        pub(crate) async fn frame(&mut self) -> Frame {
            loop {
                let message = timeout!(self.next()).expect("Connection closed unexpectedly");

                let bytes = match message {
                    WsMessage::Binary(bytes) => bytes,
                    WsMessage::Ping(..) | WsMessage::Pong(..) => continue,
                    message => panic!("Unexpected message: {message:?}"),
                };

                let mut reader = SliceReader::new(&bytes[..]);

                let header: ResponseHeader =
                    musli::packed::decode(&mut reader).expect("Failed to decode envelope");

                let at = bytes.len() - reader.remaining();
                return Frame { header, bytes, at };
            }
        }

        /// Assert that the server sends no frame for the given duration.
        ///
        /// Keepalive traffic is allowed, since it carries nothing that depends
        /// on the format.
        pub(crate) async fn expect_silence(&mut self, duration: Duration) {
            let deadline = tokio::time::Instant::now() + duration;

            loop {
                let message = match tokio::time::timeout_at(deadline, self.next()).await {
                    Ok(message) => message.expect("Connection closed unexpectedly"),
                    // NB: The timeout elapsing is the outcome being asserted.
                    Err(..) => return,
                };

                match message {
                    WsMessage::Ping(..) | WsMessage::Pong(..) => continue,
                    message => panic!("The server sent {message:?} before the format was agreed"),
                }
            }
        }

        /// Wait for the server to close the connection and return the close
        /// code it used.
        pub(crate) async fn expect_close(mut self) -> Option<u16> {
            loop {
                let message = timeout!(self.next())?;

                match message {
                    WsMessage::Close(frame) => return frame.map(|f| u16::from(f.code)),
                    WsMessage::Ping(..) | WsMessage::Pong(..) => continue,
                    message => panic!("Expected a close, got {message:?}"),
                }
            }
        }
    }
}

/// The close code a websocket peer uses to report a protocol violation.
const CLOSE_PROTOCOL_ERROR: u16 = 1002;

/// Nothing at all may reach a client which has not agreed on a format, not even
/// a broadcast the application asked for while the handshake was in flight.
#[tokio::test]
async fn nothing_is_sent_before_the_format_is_negotiated() {
    use musli_web::api::Broadcast;

    let server = TestServer::new().await;
    let mut raw = raw::RawClient::connect(&server.url()).await;

    // The hello is the one thing the server may send unprompted, and it
    // deliberately carries no body so no format applies to it.
    let frame = raw.frame().await;
    assert_eq!(frame.header.broadcast, MessageId::SERVER_HELLO.get());
    assert_eq!(frame.header.format, 0);
    assert!(frame.body().is_empty());

    // Receiving the hello proves the connection task is subscribed, so these
    // ticks are seen by a connection which has not negotiated yet.
    for tick in 0..4 {
        server.tick(tick);
    }

    raw.expect_silence(Duration::from_millis(500)).await;

    // Negotiating releases them, and every one arrives in the format that was
    // just agreed rather than the default.
    raw.negotiate(7, Format::Json).await;

    let frame = raw.frame().await;
    assert_eq!(frame.header.serial, 7);
    assert_eq!(frame.header.broadcast, 0);
    assert_eq!(frame.header.error, 0);
    assert_eq!(frame.format(), Some(Format::Json));

    for tick in 0..4 {
        let frame = raw.frame().await;

        assert_eq!(
            frame.header.broadcast,
            <api::Tick as Broadcast>::ID.get(),
            "Expected the broadcast for tick {tick}"
        );

        assert_eq!(
            frame.format(),
            Some(Format::Json),
            "A broadcast must use the negotiated format"
        );
    }

    server.shutdown().await;
}

/// Every message other than a negotiation is a protocol violation while the
/// format is still undecided.
async fn first_message_is_rejected(id: MessageId) {
    let server = TestServer::new().await;
    let mut raw = raw::RawClient::connect(&server.url()).await;

    let frame = raw.frame().await;
    assert_eq!(frame.header.broadcast, MessageId::SERVER_HELLO.get());

    raw.send(musli_web::api::RequestHeader {
        serial: 1,
        id: id.get(),
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
    })
    .await;

    assert_eq!(
        raw.expect_close().await,
        Some(CLOSE_PROTOCOL_ERROR),
        "Message id {id} must not be accepted before negotiating"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn a_request_before_negotiating_is_rejected() {
    use musli_web::api::Endpoint;

    first_message_is_rejected(<api::Hello as Endpoint>::ID).await;
}

#[tokio::test]
async fn opening_a_channel_before_negotiating_is_rejected() {
    first_message_is_rejected(MessageId::CONNECT).await;
}

/// The format is settled once and for all by the connection step, so a second
/// negotiation is refused rather than silently moving it.
#[tokio::test]
async fn the_format_cannot_be_renegotiated() {
    use musli_web::api::{Broadcast, ErrorMessage};

    let server = TestServer::new().await;
    let mut raw = raw::RawClient::connect(&server.url()).await;

    let frame = raw.frame().await;
    assert_eq!(frame.header.broadcast, MessageId::SERVER_HELLO.get());

    raw.negotiate(1, Format::Wire).await;

    let frame = raw.frame().await;
    assert_eq!(frame.header.serial, 1);
    assert_eq!(frame.format(), Some(Format::Wire));

    raw.negotiate(2, Format::Json).await;

    let frame = raw.frame().await;
    assert_eq!(frame.header.serial, 2);
    assert_eq!(frame.header.error, MessageId::ERROR_MESSAGE.get());

    let error: ErrorMessage<'_> = musli::wire::from_slice(frame.body()).expect("Failed to decode");

    assert!(
        error.message.contains("already been negotiated"),
        "Unexpected error: {}",
        error.message
    );

    // The connection is still usable, and still on the format it first agreed
    // to rather than the one it was just refused.
    server.tick(1);

    let frame = raw.frame().await;
    assert_eq!(frame.header.broadcast, <api::Tick as Broadcast>::ID.get());
    assert_eq!(frame.format(), Some(Format::Wire));

    server.shutdown().await;
}

/// A text mode connection has to be readable by hand from end to end, which is
/// the whole reason the mode exists.
#[tokio::test]
async fn text_frames_are_readable_by_hand() {
    use musli_web::api::Broadcast;

    let server = TestServer::new().await;
    let mut raw = raw::RawClient::connect(&server.url()).await;

    // The hello precedes the negotiation, so it is still a binary frame.
    let frame = raw.frame().await;
    assert_eq!(frame.header.broadcast, MessageId::SERVER_HELLO.get());

    raw.negotiate_text(1, Format::Json).await;

    // The acknowledgement comes back as text, which is what tells a client the
    // server settled on the mode it asked for.
    let text = raw.text_frame().await;

    assert_eq!(
        text,
        format!(
            "serial: 1\nbroadcast: 0\nerror: 0\nformat: {}\nchannel: 0\n\n",
            Format::Json.to_u8()
        )
    );

    // A broadcast carries its body in the same frame, after the empty line.
    server.tick(7);

    let text = raw.text_frame().await;

    let (envelope, body) = text
        .split_once("\n\n")
        .expect("The envelope must be terminated by an empty line");

    let fields = envelope
        .lines()
        .map(|line| {
            line.split_once(": ")
                .expect("Every line is `<key>: <value>`")
        })
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(
        fields.get("broadcast").copied(),
        Some(<api::Tick as Broadcast>::ID.get().to_string()).as_deref()
    );

    assert_eq!(
        fields.get("format").copied(),
        Some(Format::Json.to_u8().to_string()).as_deref()
    );

    assert_eq!(body, r#"{"message":"tick","tick":7,"channel":0}"#);

    server.shutdown().await;
}

/// Once a mode is settled, a frame of the other type carries an envelope the
/// connection has no way to read, so it is a protocol violation.
#[tokio::test]
async fn a_frame_of_the_wrong_type_is_refused() {
    let server = TestServer::new().await;
    let mut raw = raw::RawClient::connect(&server.url()).await;

    let frame = raw.frame().await;
    assert_eq!(frame.header.broadcast, MessageId::SERVER_HELLO.get());

    raw.negotiate_text(1, Format::Json).await;
    _ = raw.text_frame().await;

    // Binary is what this connection is not speaking any more.
    raw.send(musli_web::api::RequestHeader {
        serial: 2,
        id: MessageId::CONNECT.get(),
        format: 0,
        channel: ChannelId::NONE,
    })
    .await;

    assert_eq!(raw.expect_close().await, Some(CLOSE_PROTOCOL_ERROR));

    server.shutdown().await;
}
