//! Tests for the generic web client, driven through a mock [`WebImpl`] so that
//! they run natively rather than in a browser.

use super::*;

use alloc::vec;
use std::thread_local;

use crate::api::Broadcast;

/// Something the client did to the outside world.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Op {
    /// A socket was opened to the given URL.
    Connect(String),
    /// A frame was sent.
    Send(Mode, Vec<u8>),
    /// A socket was closed.
    Close,
}

thread_local! {
    static OPS: RefCell<Vec<Op>> = const { RefCell::new(Vec::new()) };
    static TIMERS: RefCell<Vec<Option<Rc<dyn Fn()>>>> = const { RefCell::new(Vec::new()) };
    static UNLOAD: RefCell<Option<Rc<dyn Fn()>>> = const { RefCell::new(None) };
    /// The protocol of the page, or `None` if looking up the location fails.
    static PROTOCOL: RefCell<Option<&'static str>> = const { RefCell::new(Some("http:")) };
    /// Whether opening a socket fails.
    static FAIL_SOCKET: Cell<bool> = const { Cell::new(false) };
    /// Called instead of sending a frame, if set.
    static SEND: RefCell<Option<Rc<dyn Fn() -> Result<(), Error>>>> = const { RefCell::new(None) };
}

/// Make every following send call `send` instead, which decides its outcome.
fn on_send(send: impl Fn() -> Result<(), Error> + 'static) {
    SEND.with(|s| *s.borrow_mut() = Some(Rc::new(send)));
}

fn push_op(op: Op) {
    OPS.with(|ops| ops.borrow_mut().push(op));
}

/// Take every operation recorded so far.
fn take_ops() -> Vec<Op> {
    OPS.with(|ops| mem::take(&mut *ops.borrow_mut()))
}

/// Run every timer which has been set and not cancelled, returning how many
/// fired.
fn fire_timers() -> usize {
    let timers = TIMERS.with(|timers| {
        timers
            .borrow_mut()
            .iter_mut()
            .filter_map(Option::take)
            .collect::<Vec<_>>()
    });

    for timer in &timers {
        timer();
    }

    timers.len()
}

/// The number of timers which have been set and not cancelled.
fn pending_timers() -> usize {
    TIMERS.with(|timers| timers.borrow().iter().filter(|t| t.is_some()).count())
}

/// Fire the `beforeunload` handler, as a browser would on navigation.
fn fire_unload() {
    let callback = UNLOAD.with(|unload| unload.borrow().clone());
    let callback = callback.expect("No beforeunload handler installed");
    callback();
}

#[derive(Clone, Copy)]
enum MockImpl {}

struct MockWindow;

struct MockSocket;

/// A timer which is cancelled when dropped, like the web-sys one.
struct MockTimeout(usize);

impl Drop for MockTimeout {
    fn drop(&mut self) {
        TIMERS.with(|timers| {
            if let Some(slot) = timers.borrow_mut().get_mut(self.0) {
                *slot = None;
            }
        });
    }
}

impl sealed_socket::Sealed for MockSocket {}

impl SocketImpl for MockSocket {
    type Handles = ();

    fn new(url: &str, _: &Self::Handles) -> Result<Self, Error> {
        if FAIL_SOCKET.with(Cell::get) {
            return Err(Error::message("Socket failed"));
        }

        push_op(Op::Connect(url.to_string()));
        Ok(MockSocket)
    }

    fn send(&self, mode: Mode, data: &[u8]) -> Result<(), Error> {
        let send = SEND.with(|s| s.borrow().clone());

        if let Some(send) = send {
            return send();
        }

        push_op(Op::Send(mode, data.to_vec()));
        Ok(())
    }

    fn close(self) -> Result<(), Error> {
        push_op(Op::Close);
        Ok(())
    }
}

impl sealed_window::Sealed for MockWindow {}

impl WindowImpl for MockWindow {
    type Timeout = MockTimeout;
    type OnBeforeUnload = ();

    fn new() -> Result<Self, Error> {
        Ok(MockWindow)
    }

    fn location(&self) -> Result<Location, Error> {
        let Some(protocol) = PROTOCOL.with(|p| *p.borrow()) else {
            return Err(Error::message("No location"));
        };

        Ok(Location {
            protocol: String::from(protocol),
            host: String::from("localhost"),
            port: String::from("8080"),
        })
    }

    fn set_timeout(&self, _: u32, callback: impl Fn() + 'static) -> Result<Self::Timeout, Error> {
        let id = TIMERS.with(|timers| {
            let mut timers = timers.borrow_mut();
            timers.push(Some(Rc::new(callback)));
            timers.len() - 1
        });

        Ok(MockTimeout(id))
    }

    fn onbeforeunload(&self, callback: impl Fn() + 'static) -> Result<Self::OnBeforeUnload, Error> {
        UNLOAD.with(|unload| *unload.borrow_mut() = Some(Rc::new(callback)));
        Ok(())
    }
}

impl sealed_web::Sealed for MockImpl {}

impl WebImpl for MockImpl {
    type Window = MockWindow;
    type Handles = ();
    type Socket = MockSocket;

    #[allow(private_interfaces)]
    fn handles(_: &Weak<Shared<Self>>) -> Self::Handles {}

    fn random(_: u32) -> u32 {
        0
    }
}

/// A broadcast used to observe how frames are decoded.
struct Ping;

impl Broadcast for Ping {
    const ID: MessageId = match MessageId::new(1) {
        Some(id) => id,
        None => panic!("Invalid message id"),
    };

    fn __do_not_implement_broadcast() {}
}

/// A request with an empty body.
#[derive(musli::Encode, musli::Decode)]
struct Hello;

enum HelloEndpoint {}

impl api::Decodable for HelloEndpoint {
    type Type<'de> = Hello;

    fn __do_not_implement_decodable() {}
}

impl api::Endpoint for HelloEndpoint {
    const ID: MessageId = match MessageId::new(2) {
        Some(id) => id,
        None => panic!("Invalid message id"),
    };

    type Response<'de> = Hello;

    fn __do_not_implement_endpoint() {}
}

impl api::Request for Hello {
    type Endpoint = HelloEndpoint;

    fn __do_not_implement_request() {}
}

const URL: &str = "ws://example.com/ws";

type Errors = Rc<RefCell<Vec<String>>>;

fn builder(connect: Connect) -> (ServiceBuilder<MockImpl, impl Callback<Error>>, Errors) {
    let errors = Errors::default();

    let builder = super::connect::<MockImpl>(connect).on_error({
        let errors = errors.clone();
        move |error: Error| errors.borrow_mut().push(error.to_string())
    });

    (builder, errors)
}

/// Deliver a frame to the client the way the socket would.
fn deliver(service: &Service<MockImpl>, mode: Mode, header: &api::ResponseHeader, body: &[u8]) {
    let mut out = Vec::new();
    format::encode_envelope(mode, &mut out, header).unwrap();
    out.extend_from_slice(body);

    let result = match mode {
        Mode::Text => service
            .shared
            .text_message(core::str::from_utf8(&out).unwrap()),
        Mode::Binary => {
            let mut buf = service.shared.next_buffer(out.len());
            buf.data.extend_from_slice(&out);
            service.shared.message(mode, buf)
        }
    };

    result.unwrap();
}

fn broadcast_header(id: MessageId) -> api::ResponseHeader {
    api::ResponseHeader {
        version: api::VERSION,
        serial: 0,
        broadcast: id.get(),
        error: 0,
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    }
}

/// Collect the body of every [`Ping`] broadcast.
fn on_ping(service: &Service<MockImpl>) -> (Rc<RefCell<Vec<Vec<u8>>>>, Listener) {
    let received = Rc::new(RefCell::new(Vec::new()));

    let listener = service.handle().on_raw_broadcast::<Ping>({
        let received = received.clone();

        move |packet: Result<RawPacket>| {
            let packet = packet.unwrap();
            let body = &packet.as_slice()[packet.at.get()..];
            received.borrow_mut().push(body.to_vec());
        }
    });

    (received, listener)
}

/// A recycled buffer must not carry the previous message into the next one,
/// which is what text frames used to do since they append to the buffer.
#[test]
fn text_frames_decode_their_own_payload() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    let (received, _listener) = on_ping(&service);

    let header = broadcast_header(Ping::ID);
    deliver(&service, Mode::Text, &header, b"first message");
    deliver(&service, Mode::Text, &header, b"second");
    deliver(&service, Mode::Text, &header, b"third and longest message");

    assert_eq!(
        *received.borrow(),
        vec![
            b"first message".to_vec(),
            b"second".to_vec(),
            b"third and longest message".to_vec(),
        ]
    );

    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Building a service opens a socket straight away.
#[test]
fn build_connects() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let _service = builder.build();
    assert_eq!(take_ops(), vec![Op::Connect(String::from(URL))]);
    assert_eq!(fire_timers(), 0);
}

/// The state of the connection as seen by the service.
fn state(service: &Service<MockImpl>) -> State {
    service.handle().on_state_change(|_: State| {}).0
}

/// Take the connection through the server hello and format negotiation.
fn open_session(service: &Service<MockImpl>) {
    deliver(
        service,
        Mode::Binary,
        &broadcast_header(MessageId::SERVER_HELLO),
        b"",
    );

    let Some(Op::Send(mode, bytes)) = take_ops().pop() else {
        panic!("Expected a negotiation request");
    };

    let mut at = 0;
    let request: api::RequestHeader = format::decode_envelope(mode, &bytes, &mut at).unwrap();
    assert_eq!(request.id, MessageId::NEGOTIATE.get());

    let response = api::ResponseHeader {
        version: api::VERSION,
        serial: request.serial,
        broadcast: 0,
        error: 0,
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    };

    deliver(service, mode, &response, b"");
    assert_eq!(state(service), State::Open);
}

/// Closing an open connection closes its socket and reports it as closed.
#[test]
fn close_closes_the_socket() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);
    take_ops();

    service.close();

    assert_eq!(take_ops(), vec![Op::Close]);
    assert_eq!(state(&service), State::Closed);

    // Nothing brings it back on its own.
    assert_eq!(fire_timers(), 0);
    assert!(take_ops().is_empty());
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Closing while a reconnect is scheduled cancels the reconnect.
#[test]
fn close_cancels_pending_reconnect() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    // What a socket error or close event does.
    service.shared.close_and_reconnect().unwrap();
    assert_eq!(take_ops(), vec![Op::Close]);
    assert_eq!(pending_timers(), 1);

    service.close();

    assert_eq!(pending_timers(), 0);
    assert!(take_ops().is_empty());
}

/// A service which has been closed can be opened again.
#[test]
fn open_after_close_reconnects() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    assert_eq!(take_ops(), vec![Op::Connect(String::from(URL))]);

    service.close();
    assert_eq!(take_ops(), vec![Op::Close]);

    service.open();
    assert_eq!(take_ops(), vec![Op::Connect(String::from(URL))]);
}

/// `close_before_unload` closes the socket when the page unloads.
#[test]
fn close_before_unload_closes_the_socket() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.close_before_unload().build();
    open_session(&service);
    take_ops();

    fire_unload();

    assert_eq!(take_ops(), vec![Op::Close]);
    assert_eq!(state(&service), State::Closed);
}

/// A failed attempt to connect is reported, and does not stop the service
/// from being opened again.
fn failed_connect_can_be_retried(connect: Connect, fail: impl FnOnce(), recover: impl FnOnce()) {
    fail();

    let (builder, errors) = builder(connect);
    let service = builder.build();

    assert!(take_ops().is_empty());
    assert_eq!(errors.borrow().len(), 1, "{:?}", errors.borrow());

    // NB: A failure is not retried on its own, since there is no reason to
    // believe that it is transient.
    assert_eq!(fire_timers(), 0);

    recover();
    service.open();

    assert_eq!(
        take_ops(),
        vec![Op::Connect(String::from("ws://localhost:8080/ws"))]
    );

    assert_eq!(errors.borrow().len(), 1, "{:?}", errors.borrow());
}

#[test]
fn failed_location_lookup_can_be_retried() {
    failed_connect_can_be_retried(
        Connect::location("ws"),
        || PROTOCOL.with(|p| *p.borrow_mut() = None),
        || PROTOCOL.with(|p| *p.borrow_mut() = Some("http:")),
    );
}

#[test]
fn unsupported_protocol_can_be_retried() {
    failed_connect_can_be_retried(
        Connect::location("ws"),
        || PROTOCOL.with(|p| *p.borrow_mut() = Some("file:")),
        || PROTOCOL.with(|p| *p.borrow_mut() = Some("http:")),
    );
}

#[test]
fn failed_socket_can_be_retried() {
    failed_connect_can_be_retried(
        Connect::location("ws"),
        || FAIL_SOCKET.with(|f| f.set(true)),
        || FAIL_SOCKET.with(|f| f.set(false)),
    );
}

/// The outcomes a request or channel callback has been called with, as the
/// error message or `None` for a success.
type Outcomes = Rc<RefCell<Vec<Option<String>>>>;

fn outcome<T>(result: Result<T>) -> Option<String> {
    result.err().map(|error| error.to_string())
}

/// Send a [`Hello`] request, recording what its callback is called with.
fn send_hello(service: &Service<MockImpl>) -> (Outcomes, Request) {
    let outcomes = Outcomes::default();

    let request = service
        .handle()
        .request()
        .body(Hello)
        .on_raw_packet({
            let outcomes = outcomes.clone();
            move |result: Result<RawPacket>| outcomes.borrow_mut().push(outcome(result))
        })
        .send();

    (outcomes, request)
}

/// Every event a channel has reported, spelled out.
type Events = Rc<RefCell<Vec<String>>>;

/// Open a channel, recording every event it reports.
fn open_channel(service: &Service<MockImpl>) -> (Events, Channel<MockImpl>) {
    let events = Events::default();

    let channel = service
        .handle()
        .channel()
        .on_change({
            let events = events.clone();

            move |event: ChannelEvent| {
                let event = match event {
                    ChannelEvent::Opened(id) => format!("opened {}", id.raw()),
                    ChannelEvent::Closed => String::from("closed"),
                    ChannelEvent::Failed(error) => format!("failed: {error}"),
                };

                events.borrow_mut().push(event);
            }
        })
        .open();

    (events, channel)
}

/// Take every event recorded so far.
fn take_events(events: &Events) -> Vec<String> {
    mem::take(&mut *events.borrow_mut())
}

/// Whether any request is still waiting for a response.
fn has_pending(service: &Service<MockImpl>) -> bool {
    !service.shared.g.requests.borrow().is_empty()
}

/// A request which could not be sent is failed through its own callback,
/// rather than only through the connection-wide error handler.
#[test]
fn send_error_fails_the_request() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    on_send(|| Err(Error::message("Send failed")));

    let (outcomes, _request) = send_hello(&service);

    assert_eq!(*outcomes.borrow(), vec![Some(String::from("Send failed"))]);
    assert!(!has_pending(&service));
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A channel which could not be requested is failed through its own callback.
#[test]
fn send_error_fails_the_channel() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    on_send(|| Err(Error::message("Send failed")));

    let (events, channel) = open_channel(&service);

    assert_eq!(
        take_events(&events),
        vec![String::from("failed: Send failed")]
    );
    assert!(!channel.is_open());
    assert!(!has_pending(&service));
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Make sending fail every pending request before it returns an error, the
/// way the close handler does when sending observes that the peer is gone.
fn fail_pending_on_send(service: &Service<MockImpl>) {
    on_send({
        let shared = Rc::downgrade(&service.shared);

        move || {
            if let Some(shared) = shared.upgrade() {
                shared.close_pending();
            }

            Err(Error::message("Send failed"))
        }
    });
}

/// Sending can observe that the peer is gone and fail every pending request
/// before it returns. The request being sent must be among them, and must only
/// be failed once.
#[test]
fn send_error_after_pending_requests_were_failed() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);
    fail_pending_on_send(&service);

    let (outcomes, _request) = send_hello(&service);

    assert_eq!(
        *outcomes.borrow(),
        vec![Some(String::from("Connection closed"))]
    );
    assert!(!has_pending(&service));
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Like [`send_error_after_pending_requests_were_failed`], but for opening a
/// channel. A channel lost with the connection is not a failure, since it is
/// opened again once the connection is back, so nothing is reported.
#[test]
fn channel_send_error_after_pending_requests_were_failed() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);
    fail_pending_on_send(&service);

    let (events, channel) = open_channel(&service);

    assert!(take_events(&events).is_empty());
    assert!(!channel.is_open());
    assert!(!has_pending(&service));
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Every error a callback has been called with.
type Failures = Rc<RefCell<Vec<Error>>>;

/// Send a [`Hello`] request, recording every error its callback gets.
fn send_hello_failures(service: &Service<MockImpl>) -> (Failures, Request) {
    let failures = Failures::default();

    let request = service
        .handle()
        .request()
        .body(Hello)
        .on_raw_packet({
            let failures = failures.clone();

            move |result: Result<RawPacket>| {
                if let Err(error) = result {
                    failures.borrow_mut().push(error);
                }
            }
        })
        .send();

    (failures, request)
}

/// Take the single error recorded in `failures`.
fn single(failures: &Failures) -> Error {
    let mut failures = failures.borrow_mut();
    assert_eq!(failures.len(), 1, "{failures:?}");
    failures.pop().unwrap()
}

/// The serial of the last request sent.
fn last_serial() -> u32 {
    let Some(Op::Send(mode, bytes)) = take_ops().pop() else {
        panic!("Expected a request to be sent");
    };

    let mut at = 0;
    let request: api::RequestHeader = format::decode_envelope(mode, &bytes, &mut at).unwrap();
    request.serial
}

/// The body of an error response carrying `message`.
fn error_body(message: &str) -> Vec<u8> {
    let mut body = Vec::new();

    Format::DEFAULT
        .encode(&mut body, &api::ErrorMessage { message })
        .unwrap();

    body
}

/// Check every accessor at once, so that each error kind is shown to match
/// exactly one of them.
#[track_caller]
fn assert_kind(error: &Error, expected: &str) {
    let actual = [
        ("not_connected", error.is_not_connected()),
        ("connection_lost", error.is_connection_lost()),
        ("cancelled", error.is_cancelled()),
        ("server", error.as_server_error().is_some()),
        ("decode", error.is_decode()),
    ]
    .into_iter()
    .filter(|(_, is)| *is)
    .map(|(name, _)| name)
    .collect::<Vec<_>>();

    let expected = if expected.is_empty() {
        vec![]
    } else {
        vec![expected]
    };

    assert_eq!(actual, expected, "{error}");
}

/// A request sent before the connection is open is never sent, and says so.
#[test]
fn request_before_open_is_not_connected() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let (failures, _request) = send_hello_failures(&service);

    let error = single(&failures);
    assert_kind(&error, "not_connected");
    assert_eq!(error.to_string(), "WebSocket is not connected");
    assert!(take_ops().is_empty());
}

/// A request over a channel which is not open is never sent.
#[test]
fn request_over_closed_channel_is_not_connected() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);
    take_ops();

    let failures = Failures::default();

    // NB: Not answered, so the channel never opens.
    let channel = service.handle().channel().open();
    take_ops();

    let _request = channel
        .request()
        .body(Hello)
        .on_raw_packet({
            let failures = failures.clone();

            move |result: Result<RawPacket>| {
                if let Err(error) = result {
                    failures.borrow_mut().push(error);
                }
            }
        })
        .send();

    let error = single(&failures);
    assert_kind(&error, "not_connected");
    assert_eq!(error.to_string(), "WebSocket request over closed channel");
    assert!(take_ops().is_empty());
}

/// A request which could not be sent since the socket is gone is not
/// connected, and is not reported as lost.
#[test]
fn send_without_socket_is_not_connected() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    // The state is still open, but the socket has gone away.
    service.shared.socket.borrow_mut().take();

    let (failures, _request) = send_hello_failures(&service);

    let error = single(&failures);
    assert_kind(&error, "not_connected");
    assert_eq!(error.to_string(), "Socket is not connected");
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A request which was sent but not answered before the connection closed
/// is lost, which is distinct from never having been sent.
#[test]
fn pending_request_is_lost_on_close() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (failures, _request) = send_hello_failures(&service);
    assert!(failures.borrow().is_empty());

    service.shared.close_pending();

    let error = single(&failures);
    assert_kind(&error, "connection_lost");
    assert_eq!(error.to_string(), "Connection closed");
}

/// Dropping the service fails pending requests as lost.
#[test]
fn pending_request_is_lost_on_drop() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (failures, _request) = send_hello_failures(&service);
    drop(service);

    let error = single(&failures);
    assert_kind(&error, "connection_lost");
    assert_eq!(error.to_string(), "Websocket service closed");
}

/// A request replaced by another under the same serial is cancelled.
#[test]
fn replaced_request_is_cancelled() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (first, _first) = send_hello_failures(&service);
    let serial = last_serial();

    // NB: Serials wrap around, so a request which is outstanding for long
    // enough is replaced by a later one.
    service.shared.serial.set(serial);
    let (second, _second) = send_hello_failures(&service);

    let error = single(&first);
    assert_kind(&error, "cancelled");
    assert_eq!(error.to_string(), "Request cancelled");
    assert!(second.borrow().is_empty());
}

/// An error response exposes the server's message without the prefix used
/// when displaying it.
#[test]
fn error_response_is_server_error() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (failures, _request) = send_hello_failures(&service);
    let serial = last_serial();

    let header = api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: MessageId::ERROR_MESSAGE.get(),
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    };

    deliver(&service, Mode::Binary, &header, &error_body("No such user"));

    let error = single(&failures);
    assert_kind(&error, "server");
    assert_eq!(error.as_server_error(), Some("No such user"));
    assert_eq!(error.to_string(), "Server error: No such user");
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A request the server does not support is a server error.
#[test]
fn unsupported_request_is_server_error() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (failures, _request) = send_hello_failures(&service);
    let serial = last_serial();

    let header = api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        // Any error id other than an error message.
        error: MessageId::CONNECT.get(),
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    };

    deliver(&service, Mode::Binary, &header, b"");

    let error = single(&failures);
    assert_kind(&error, "server");
    assert_eq!(error.as_server_error(), Some("Unsupported request"));
}

/// An error broadcast is delivered to listeners as a server error.
#[test]
fn error_broadcast_is_server_error() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();

    let failures = Failures::default();

    let _listener = service.handle().on_raw_broadcast::<Ping>({
        let failures = failures.clone();

        move |result: Result<RawPacket>| {
            if let Err(error) = result {
                failures.borrow_mut().push(error);
            }
        }
    });

    let header = api::ResponseHeader {
        error: MessageId::ERROR_MESSAGE.get(),
        ..broadcast_header(Ping::ID)
    };

    deliver(&service, Mode::Binary, &header, &error_body("Overloaded"));

    let error = single(&failures);
    assert_kind(&error, "server");
    assert_eq!(error.as_server_error(), Some("Overloaded"));
}

/// A typed error, the way an application would define one in its API crate.
#[derive(Debug, PartialEq, musli::Encode, musli::Decode)]
enum Refusal {
    NotFound,
    Forbidden { reason: String },
}

/// A type which is not what the server sent, to decode a payload as.
#[derive(Debug, PartialEq, musli::Encode, musli::Decode)]
struct Unrelated {
    a: u64,
    b: u64,
    c: u64,
}

/// The body of a typed error response carrying `message` followed by an
/// already encoded `payload`.
fn typed_error_body(format: Format, message: &str, payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    format
        .encode(&mut body, &api::ErrorMessage { message })
        .unwrap();
    body.extend_from_slice(payload);
    body
}

fn encode_payload(format: Format, value: &impl api::EncodeBody) -> Vec<u8> {
    let mut out = Vec::new();
    format.encode(&mut out, value).unwrap();
    out
}

/// The header of a response to `serial` which carries `error`.
fn error_header(serial: u32, error: MessageId, format: Format) -> api::ResponseHeader {
    api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: error.get(),
        format: format.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    }
}

/// Fail a fresh request with a typed error response whose body is `body`, and
/// return the error the request saw.
fn fail_typed(mode: Mode, format: Format, body: &[u8]) -> Error {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (failures, _request) = send_hello_failures(&service);
    let serial = last_serial();

    let header = error_header(serial, MessageId::TYPED_ERROR_MESSAGE, format);
    deliver(&service, mode, &header, body);

    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
    single(&failures)
}

/// A typed error response is a server error which decodes back into the type
/// the server sent, and still has its message.
#[test]
fn typed_error_response_is_decoded() {
    let refusal = Refusal::Forbidden {
        reason: String::from("Read only"),
    };

    let format = Format::DEFAULT;
    let payload = encode_payload(format, &refusal);
    let error = fail_typed(
        Mode::Binary,
        format,
        &typed_error_body(format, "Forbidden: Read only", &payload),
    );

    assert_kind(&error, "server");
    assert_eq!(error.as_server_error(), Some("Forbidden: Read only"));
    assert_eq!(error.to_string(), "Server error: Forbidden: Read only");
    assert_eq!(error.server_error::<Refusal>().unwrap(), Some(refusal));

    // NB: Decoding does not consume the payload.
    assert!(error.server_error::<Refusal>().unwrap().is_some());
}

/// A typed error is decoded with the format its envelope declares, which is
/// how a text mode connection carries one.
#[test]
fn typed_error_response_in_text_mode() {
    let format = Format::Json;
    let payload = encode_payload(format, &Refusal::NotFound);
    let body = typed_error_body(format, "Not found", &payload);

    let error = fail_typed(Mode::Text, format, &body);

    assert_kind(&error, "server");
    assert_eq!(error.as_server_error(), Some("Not found"));
    assert_eq!(
        error.server_error::<Refusal>().unwrap(),
        Some(Refusal::NotFound)
    );
}

/// A plain error message carries no typed error.
#[test]
fn error_message_has_no_typed_error() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (failures, _request) = send_hello_failures(&service);
    let serial = last_serial();

    let header = error_header(serial, MessageId::ERROR_MESSAGE, Format::DEFAULT);
    deliver(&service, Mode::Binary, &header, &error_body("No such user"));

    let error = single(&failures);
    assert_eq!(error.server_error::<Refusal>().unwrap(), None);

    // Neither does anything which is not a server error.
    assert_eq!(
        Error::message("Something else")
            .server_error::<Refusal>()
            .unwrap(),
        None
    );
}

/// A typed error which does not decode as the requested type is a decode
/// error, while the message is still available.
#[test]
fn typed_error_of_another_type_is_decode() {
    let format = Format::DEFAULT;
    let payload = encode_payload(format, &Refusal::NotFound);
    let error = fail_typed(
        Mode::Binary,
        format,
        &typed_error_body(format, "Not found", &payload),
    );

    let decode = error.server_error::<Unrelated>().unwrap_err();
    assert_kind(&decode, "decode");
    assert_eq!(error.as_server_error(), Some("Not found"));
}

/// A typed error whose payload is garbage, or missing, is a decode error when
/// it is decoded.
#[test]
fn malformed_typed_error_payload_is_decode() {
    let format = Format::DEFAULT;

    for payload in [&[0xff; 4][..], &[][..]] {
        let error = fail_typed(
            Mode::Binary,
            format,
            &typed_error_body(format, "Broken", payload),
        );

        assert_kind(&error, "server");
        assert_eq!(error.as_server_error(), Some("Broken"));

        let decode = error.server_error::<Refusal>().unwrap_err();
        assert_kind(&decode, "decode");
    }
}

/// A typed error whose message cannot be decoded fails the request with a
/// decode error, rather than leaving it waiting.
#[test]
fn malformed_typed_error_message_fails_the_request() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let (failures, _request) = send_hello_failures(&service);
    let serial = last_serial();

    let header = error_header(serial, MessageId::TYPED_ERROR_MESSAGE, Format::DEFAULT);
    deliver(&service, Mode::Binary, &header, &[0xff; 4]);

    let error = single(&failures);
    assert_kind(&error, "decode");
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
    assert!(!has_pending(&service));
}

/// A typed error broadcast reaches every listener with its payload.
#[test]
fn typed_error_broadcast_is_decoded() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();

    let failures = Failures::default();

    let listener = |failures: &Failures| {
        let failures = failures.clone();

        move |result: Result<RawPacket>| {
            if let Err(error) = result {
                failures.borrow_mut().push(error);
            }
        }
    };

    let _first = service
        .handle()
        .on_raw_broadcast::<Ping>(listener(&failures));
    let _second = service
        .handle()
        .on_raw_broadcast::<Ping>(listener(&failures));

    let header = api::ResponseHeader {
        error: MessageId::TYPED_ERROR_MESSAGE.get(),
        ..broadcast_header(Ping::ID)
    };

    let format = Format::DEFAULT;
    let payload = encode_payload(format, &Refusal::NotFound);
    deliver(
        &service,
        Mode::Binary,
        &header,
        &typed_error_body(format, "Not found", &payload),
    );

    let failures = failures.borrow();
    assert_eq!(failures.len(), 2);

    for error in failures.iter() {
        assert_kind(error, "server");
        assert_eq!(
            error.server_error::<Refusal>().unwrap(),
            Some(Refusal::NotFound)
        );
    }
}

/// A response which does not decode as the requested type is a decode error.
#[test]
fn undecodable_packet_is_decode() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();

    let packets = Rc::new(RefCell::new(Vec::new()));

    let _listener = service.handle().on_raw_broadcast::<Ping>({
        let packets = packets.clone();
        move |result: Result<RawPacket>| packets.borrow_mut().push(result.unwrap())
    });

    deliver(
        &service,
        Mode::Binary,
        &broadcast_header(Ping::ID),
        &[0xff; 4],
    );

    let packet = packets.borrow_mut().pop().unwrap();
    let error = packet.decode::<String>().unwrap_err();
    assert_kind(&error, "decode");
    assert!(!error.is_empty_packet());

    let error = RawPacket::empty().decode::<String>().unwrap_err();
    assert_kind(&error, "decode");
    assert!(error.is_empty_packet());
}

/// A frame using a format the client does not know is a decode error.
#[test]
fn unknown_format_is_decode() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    let (_received, _listener) = on_ping(&service);

    let header = api::ResponseHeader {
        format: u8::MAX,
        ..broadcast_header(Ping::ID)
    };

    let mut out = Vec::new();
    format::encode_envelope(Mode::Binary, &mut out, &header).unwrap();
    let mut buf = service.shared.next_buffer(out.len());
    buf.data.extend_from_slice(&out);

    let error = service.shared.message(Mode::Binary, buf).unwrap_err();
    assert_kind(&error, "decode");
}

/// A plain message matches none of the accessors.
#[test]
fn message_matches_no_accessor() {
    let error = Error::message("Something else");
    assert_kind(&error, "");
    assert!(!error.is_empty_packet());
}

/// Lose the connection the way a socket error or close event does, and take
/// the reconnect through to an open session again.
fn reconnect(service: &Service<MockImpl>) {
    service.shared.close_and_reconnect().unwrap();
    assert_eq!(state(service), State::Closed);
    assert_eq!(take_ops(), vec![Op::Close]);
    assert_eq!(fire_timers(), 1);
    assert_eq!(take_ops(), vec![Op::Connect(String::from(URL))]);
    open_session(service);
}

/// The generation counts how many times the connection has opened.
#[test]
fn generation_counts_opens() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    let handle = service.handle().clone();
    take_ops();

    assert_eq!(handle.generation(), 0);

    open_session(&service);
    assert_eq!(handle.generation(), 1);

    // Closing does not move it, only opening again does.
    service.shared.close_and_reconnect().unwrap();
    assert_eq!(handle.generation(), 1);
    assert_eq!(fire_timers(), 1);
    take_ops();
    assert_eq!(handle.generation(), 1);
    open_session(&service);
    assert_eq!(handle.generation(), 2);

    // Neither does an explicit close and open, until it opens.
    service.close();
    service.open();
    take_ops();
    assert_eq!(handle.generation(), 2);
    open_session(&service);
    assert_eq!(handle.generation(), 3);

    drop(service);
    assert_eq!(handle.generation(), 0);
    assert_eq!(Handle::<MockImpl>::default().generation(), 0);
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A server which rejects the requested format still opens the connection,
/// and that counts as a new generation like any other open.
#[test]
fn generation_counts_open_after_rejected_format() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    deliver(
        &service,
        Mode::Binary,
        &broadcast_header(MessageId::SERVER_HELLO),
        b"",
    );

    let serial = last_serial();

    let response = api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: MessageId::ERROR_MESSAGE.get(),
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    };

    deliver(
        &service,
        Mode::Binary,
        &response,
        &error_body("Unsupported format"),
    );

    assert_eq!(state(&service), State::Open);
    assert_eq!(service.handle().generation(), 1);
}

/// Listeners see the generation of the connection which just opened.
#[test]
fn generation_is_updated_before_listeners() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let seen = Rc::new(RefCell::new(Vec::new()));

    let (_, _listener) = service.handle().on_state_change({
        let seen = seen.clone();
        let handle = service.handle().clone();
        move |state: State| seen.borrow_mut().push((state, handle.generation()))
    });

    open_session(&service);
    reconnect(&service);

    assert_eq!(
        *seen.borrow(),
        vec![(State::Open, 1), (State::Closed, 1), (State::Open, 2)]
    );
}

/// `on_open` is called with the generation each time the connection opens,
/// and never when it closes.
#[test]
fn on_open_reports_each_open() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let opened = Rc::new(RefCell::new(Vec::new()));

    let on_open = |service: &Service<MockImpl>| {
        let opened = opened.clone();

        service
            .handle()
            .on_open(move |generation: u64| opened.borrow_mut().push(generation))
    };

    let (current, listener) = on_open(&service);
    assert_eq!(current, None);
    assert!(opened.borrow().is_empty());

    open_session(&service);
    assert_eq!(*opened.borrow(), vec![1]);

    reconnect(&service);
    assert_eq!(*opened.borrow(), vec![1, 2]);

    service.close();
    assert_eq!(*opened.borrow(), vec![1, 2]);

    // Dropping the listener stops it.
    drop(listener);
    service.open();
    take_ops();
    open_session(&service);
    assert_eq!(*opened.borrow(), vec![1, 2]);

    // Listening while open reports the current generation straight away
    // instead of calling the callback.
    let (current, _listener) = on_open(&service);
    assert_eq!(current, Some(3));
    assert_eq!(*opened.borrow(), vec![1, 2]);

    reconnect(&service);
    assert_eq!(*opened.borrow(), vec![1, 2, 4]);

    let (current, _listener) = Handle::<MockImpl>::default().on_open(|_: u64| {});
    assert_eq!(current, None);
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Take the request to open a channel, which must be the last frame sent,
/// returning its serial.
fn take_connect() -> u32 {
    let Some(Op::Send(mode, bytes)) = take_ops().pop() else {
        panic!("Expected a request to open a channel");
    };

    let mut at = 0;
    let request: api::RequestHeader = format::decode_envelope(mode, &bytes, &mut at).unwrap();
    assert_eq!(request.id, MessageId::CONNECT.get());
    request.serial
}

/// Answer the request with `serial` to open a channel with `id`.
fn answer_connect(service: &Service<MockImpl>, serial: u32, id: u16) {
    let response = api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: 0,
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::from_u16(id),
        app: 0,
    };

    deliver(service, Mode::Binary, &response, b"");
}

/// Answer the request to open a channel which was sent last with `id`.
fn accept_channel(service: &Service<MockImpl>, id: u16) {
    let serial = take_connect();
    answer_connect(service, serial, id);
}

/// Whether `ops` holds a request to open a channel.
fn connects(ops: &[Op]) -> bool {
    ops.iter().any(|op| {
        let Op::Send(mode, bytes) = op else {
            return false;
        };

        let mut at = 0;
        let request: api::RequestHeader = format::decode_envelope(*mode, bytes, &mut at).unwrap();
        request.id == MessageId::CONNECT.get()
    })
}

/// Whether `ops` holds a disconnect of `channel`.
fn disconnects(ops: &[Op], channel: u16) -> bool {
    ops.iter().any(|op| {
        let Op::Send(mode, bytes) = op else {
            return false;
        };

        let mut at = 0;
        let request: api::RequestHeader = format::decode_envelope(*mode, bytes, &mut at).unwrap();
        request.id == MessageId::DISCONNECT.get() && request.channel == ChannelId::from_u16(channel)
    })
}

/// Send a [`Hello`] over `channel`, recording every error its callback gets.
fn send_hello_over(channel: &Channel<MockImpl>) -> (Failures, Request) {
    let failures = Failures::default();

    let request = channel
        .request()
        .body(Hello)
        .on_raw_packet({
            let failures = failures.clone();

            move |result: Result<RawPacket>| {
                if let Err(error) = result {
                    failures.borrow_mut().push(error);
                }
            }
        })
        .send();

    (failures, request)
}

/// The channel of the request which was sent last.
fn last_channel() -> ChannelId {
    let Some(Op::Send(mode, bytes)) = take_ops().pop() else {
        panic!("Expected a request to be sent");
    };

    let mut at = 0;
    let request: api::RequestHeader = format::decode_envelope(mode, &bytes, &mut at).unwrap();
    request.channel
}

/// A channel set up before the connection opens is opened once it does, is
/// reported closed when the connection is lost, and is opened again by itself
/// once the connection is back.
#[test]
fn channel_opens_closes_and_reopens() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let (events, channel) = open_channel(&service);

    // Nothing is asked for before the connection is open.
    assert!(take_ops().is_empty());
    assert!(take_events(&events).is_empty());
    assert!(!channel.is_open());
    assert_eq!(channel.id(), ChannelId::NONE);
    assert_eq!(channel.generation(), 0);

    open_session(&service);
    accept_channel(&service, 7);

    assert_eq!(take_events(&events), vec![String::from("opened 7")]);
    assert!(channel.is_open());
    assert_eq!(channel.id(), ChannelId::from_u16(7));
    assert_eq!(channel.generation(), 1);

    let (failures, _request) = send_hello_over(&channel);
    assert!(failures.borrow().is_empty());
    assert_eq!(last_channel(), ChannelId::from_u16(7));

    // Losing the connection closes the channel.
    service.shared.close_and_reconnect().unwrap();
    assert_eq!(take_events(&events), vec![String::from("closed")]);
    assert!(!channel.is_open());
    assert_eq!(channel.id(), ChannelId::NONE);
    assert_eq!(channel.generation(), 1);

    // And reconnecting opens it again, with whatever id the server hands out.
    assert_eq!(fire_timers(), 1);
    take_ops();
    open_session(&service);
    assert!(!channel.is_open());
    accept_channel(&service, 9);

    assert_eq!(take_events(&events), vec![String::from("opened 9")]);
    assert!(channel.is_open());
    assert_eq!(channel.id(), ChannelId::from_u16(9));
    assert_eq!(channel.generation(), 2);

    let (failures, _request) = send_hello_over(&channel);
    assert!(failures.borrow().is_empty());
    assert_eq!(last_channel(), ChannelId::from_u16(9));

    // An explicit close and open does the same.
    service.close();
    assert_eq!(take_events(&events), vec![String::from("closed")]);
    service.open();
    take_ops();
    open_session(&service);
    accept_channel(&service, 3);
    assert_eq!(take_events(&events), vec![String::from("opened 3")]);
    assert_eq!(channel.generation(), 3);

    // Dropping the channel disconnects the current id only, and stops it from
    // being opened again.
    drop(channel);
    let ops = take_ops();
    assert!(disconnects(&ops, 3));
    assert!(!disconnects(&ops, 7));
    assert!(!disconnects(&ops, 9));

    reconnect(&service);
    assert!(!connects(&take_ops()));
    assert!(take_events(&events).is_empty());
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A channel set up while the connection is open is opened straight away.
#[test]
fn channel_opens_straight_away_when_connected() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();
    open_session(&service);

    let (events, channel) = open_channel(&service);
    accept_channel(&service, 4);

    assert_eq!(take_events(&events), vec![String::from("opened 4")]);
    assert!(channel.is_open());
    assert_eq!(channel.generation(), 1);
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A request over a channel which is closed, or not open yet, fails as not
/// connected without being sent, and works once it has opened again.
#[test]
fn channel_request_while_closed_is_not_connected() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let (_events, channel) = open_channel(&service);

    // Before the connection is open.
    let (failures, _request) = send_hello_over(&channel);
    assert_kind(&single(&failures), "not_connected");
    assert!(take_ops().is_empty());

    // While the server has not answered.
    open_session(&service);
    let serial = take_connect();
    let (failures, _request) = send_hello_over(&channel);
    assert_kind(&single(&failures), "not_connected");
    assert!(take_ops().is_empty());

    answer_connect(&service, serial, 5);

    // After the connection was lost.
    service.shared.close_and_reconnect().unwrap();
    take_ops();
    let (failures, _request) = send_hello_over(&channel);
    assert_kind(&single(&failures), "not_connected");
    assert!(take_ops().is_empty());

    assert_eq!(fire_timers(), 1);
    take_ops();
    open_session(&service);
    accept_channel(&service, 6);

    let (failures, _request) = send_hello_over(&channel);
    assert!(failures.borrow().is_empty());
    assert_eq!(last_channel(), ChannelId::from_u16(6));
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// `when_open` runs straight away on an open channel, and otherwise waits for
/// it to open, in order, before the channel reports it opened.
#[test]
fn channel_when_open_waits_for_the_channel() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let order = Rc::new(RefCell::new(Vec::new()));

    let channel = service
        .handle()
        .channel()
        .on_change({
            let order = order.clone();

            move |event: ChannelEvent| {
                if let ChannelEvent::Opened(..) = event {
                    order.borrow_mut().push(String::from("opened"));
                }
            }
        })
        .open();

    let queue = |name: &'static str| {
        let order = order.clone();

        channel.when_open(move |channel: &Channel<MockImpl>| {
            assert!(channel.is_open());
            order
                .borrow_mut()
                .push(format!("{name} {}", channel.id().raw()));
        });
    };

    queue("first");
    queue("second");
    assert!(order.borrow().is_empty());

    open_session(&service);
    assert!(order.borrow().is_empty());
    accept_channel(&service, 8);

    assert_eq!(
        mem::take(&mut *order.borrow_mut()),
        vec![
            String::from("first 8"),
            String::from("second 8"),
            String::from("opened"),
        ]
    );

    // Straight away while open.
    queue("now");
    assert_eq!(
        mem::take(&mut *order.borrow_mut()),
        vec![String::from("now 8")]
    );

    // A request made while the connection is down goes out once the channel
    // is open again.
    service.shared.close_and_reconnect().unwrap();
    take_ops();

    let failures = Failures::default();
    let sent = Rc::new(RefCell::new(Request::new()));

    channel.when_open({
        let failures = failures.clone();
        let sent = sent.clone();

        move |channel: &Channel<MockImpl>| {
            *sent.borrow_mut() = channel
                .request()
                .body(Hello)
                .on_raw_packet(move |result: Result<RawPacket>| {
                    if let Err(error) = result {
                        failures.borrow_mut().push(error);
                    }
                })
                .send();
        }
    });

    assert_eq!(fire_timers(), 1);
    take_ops();
    open_session(&service);
    accept_channel(&service, 2);

    assert_eq!(last_channel(), ChannelId::from_u16(2));
    assert!(failures.borrow().is_empty());
    assert!(sent.borrow().is_pending());

    // Queued calls are dropped with the channel.
    service.shared.close_and_reconnect().unwrap();
    let dropped = Rc::new(Cell::new(false));

    struct SetOnDrop(Rc<Cell<bool>>);

    impl Drop for SetOnDrop {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    channel.when_open({
        let guard = SetOnDrop(dropped.clone());

        move |_: &Channel<MockImpl>| {
            let _guard = guard;
            panic!("Should never be called");
        }
    });

    assert!(!dropped.get());
    drop(channel);
    assert!(dropped.get());
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Losing the connection while the channel is being opened reports nothing
/// but the channel opening once the connection is back.
#[test]
fn channel_lost_while_opening_is_opened_again() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let (events, channel) = open_channel(&service);
    open_session(&service);
    let serial = take_connect();

    service.shared.close_and_reconnect().unwrap();
    assert!(take_events(&events).is_empty());
    assert!(!has_pending(&service));

    // A late answer for the lost connection is ignored.
    answer_connect(&service, serial, 1);
    assert!(take_events(&events).is_empty());
    assert!(!channel.is_open());

    assert_eq!(fire_timers(), 1);
    take_ops();
    open_session(&service);
    accept_channel(&service, 2);
    assert_eq!(take_events(&events), vec![String::from("opened 2")]);
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A server which refuses to open the channel is reported as a failure, and
/// the channel is tried again when the connection reopens.
#[test]
fn channel_refused_by_server_is_failed() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let (events, channel) = open_channel(&service);
    open_session(&service);
    let serial = take_connect();

    let response = api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: MessageId::ERROR_MESSAGE.get(),
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    };

    deliver(
        &service,
        Mode::Binary,
        &response,
        &error_body("Out of channels"),
    );

    assert_eq!(
        take_events(&events),
        vec![String::from("failed: Server error: Out of channels")]
    );
    assert!(!channel.is_open());

    reconnect(&service);
    accept_channel(&service, 1);
    assert_eq!(take_events(&events), vec![String::from("opened 1")]);
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Clones refer to the same channel, which is disconnected once the last of
/// them is dropped.
#[test]
fn channel_clones_share_the_channel() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();
    open_session(&service);

    let (_events, channel) = open_channel(&service);
    let clone = channel.clone();
    accept_channel(&service, 3);

    assert!(clone.is_open());
    assert_eq!(clone.id(), ChannelId::from_u16(3));
    assert!(channel == clone);

    drop(channel);
    assert!(!disconnects(&take_ops(), 3));
    assert!(clone.is_open());

    drop(clone);
    assert!(disconnects(&take_ops(), 3));
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A channel dropped while the connection is down has nothing to disconnect,
/// and sends nothing.
#[test]
fn channel_dropped_while_closed_sends_nothing() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();
    open_session(&service);

    let (_events, channel) = open_channel(&service);
    accept_channel(&service, 3);

    service.shared.close_and_reconnect().unwrap();
    take_ops();
    drop(channel);

    assert!(take_ops().is_empty());
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// Dropping the service closes an open channel and cancels one which is
/// still being opened, without touching the request table while it is in
/// use.
#[test]
fn channel_outlives_the_service() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();
    open_session(&service);

    let (open_events, open) = open_channel(&service);
    accept_channel(&service, 3);
    take_events(&open_events);

    let (opening_events, opening) = open_channel(&service);
    take_connect();

    drop(service);

    assert_eq!(take_events(&open_events), vec![String::from("closed")]);
    assert!(take_events(&opening_events).is_empty());
    assert!(!open.is_open());
    assert!(!opening.is_open());

    // Neither has a service to send anything to.
    drop(open);
    drop(opening);
}

/// A default channel is not connected to anything and never opens.
#[test]
fn default_channel_never_opens() {
    let channel = Channel::<MockImpl>::default();
    assert!(!channel.is_open());
    assert_eq!(channel.id(), ChannelId::NONE);
    assert_eq!(channel.generation(), 0);

    let (failures, _request) = send_hello_over(&channel);
    assert_kind(&single(&failures), "not_connected");
    assert_eq!(channel, Channel::default());
}

/// Answer the request with `serial` with an empty response.
fn respond(service: &Service<MockImpl>, serial: u32) {
    let header = api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: 0,
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    };

    deliver(service, Mode::Binary, &header, b"");
}

/// Whether the request with `serial` is still waiting for a response.
fn is_waiting(service: &Service<MockImpl>, serial: u32) -> bool {
    service.shared.g.requests.borrow().contains_key(&serial)
}

/// Send a hello, returning what its callback is told, the request and its
/// serial.
fn send_hello_serial(service: &Service<MockImpl>) -> (Outcomes, Request, u32) {
    let (outcomes, request) = send_hello(service);
    (outcomes, request, last_serial())
}

/// Pushing a request leaves the requests pushed before it in flight, and a
/// completed request is no longer counted.
#[test]
fn requests_keep_many_in_flight_and_forget_completed() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut requests = Requests::new();
    assert!(requests.is_empty());

    let (first, request, a) = send_hello_serial(&service);
    requests.push(request);
    let (second, request, b) = send_hello_serial(&service);
    requests.push(request);

    assert_eq!(requests.len(), 2);
    assert!(is_waiting(&service, a));
    assert!(is_waiting(&service, b));

    respond(&service, a);

    assert_eq!(*first.borrow(), [None]);
    assert!(second.borrow().is_empty());
    assert_eq!(requests.len(), 1);
    assert!(!requests.is_empty());
    assert!(is_waiting(&service, b));

    respond(&service, b);

    assert_eq!(*second.borrow(), [None]);
    assert_eq!(requests.len(), 0);
    assert!(requests.is_empty());
}

/// A request which could not be sent is not kept.
#[test]
fn requests_drop_requests_which_are_not_pending() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let mut requests = Requests::new();
    let (failures, request) = send_hello_failures(&service);
    assert_kind(&single(&failures), "not_connected");

    requests.push(request);
    requests.push(Request::new());

    assert!(requests.is_empty());
    assert!(requests.requests.is_empty());
}

/// Completed requests are forgotten instead of growing the set.
#[test]
fn requests_prune_completed_before_growing() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut requests = Requests::new();

    for _ in 0..64 {
        let (_, request, serial) = send_hello_serial(&service);
        requests.push(request);
        respond(&service, serial);
    }

    assert!(requests.is_empty());
    assert!(
        requests.requests.capacity() <= 8,
        "{}",
        requests.requests.capacity()
    );

    // NB: One request stays in flight throughout, and is never pruned.
    let (_, request, kept) = send_hello_serial(&service);
    requests.push(request);

    for _ in 0..64 {
        let (_, request, serial) = send_hello_serial(&service);
        requests.push(request);
        respond(&service, serial);
    }

    assert_eq!(requests.len(), 1);
    assert!(is_waiting(&service, kept));
    assert!(requests.requests.len() <= 8, "{}", requests.requests.len());
}

/// Dropping or clearing the set cancels every request still in flight,
/// without calling their callbacks.
#[test]
fn requests_drop_and_clear_cancel_all() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut requests = Requests::new();
    let mut outcomes = Vec::new();

    for _ in 0..3 {
        let (o, request, _) = send_hello_serial(&service);
        requests.push(request);
        outcomes.push(o);
    }

    assert_eq!(requests.len(), 3);
    drop(requests);
    assert!(!has_pending(&service));

    let mut requests = Requests::new();

    for _ in 0..3 {
        let (o, request, _) = send_hello_serial(&service);
        requests.push(request);
        outcomes.push(o);
    }

    requests.clear();
    assert!(requests.is_empty());
    assert!(!has_pending(&service));

    for o in outcomes {
        assert!(o.borrow().is_empty(), "{:?}", o.borrow());
    }
}

/// Inserting under a key cancels the request stored under it before, and
/// leaves other keys alone.
#[test]
fn keyed_requests_insert_replaces_by_key() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut requests = KeyedRequests::new();

    let (_, request, a) = send_hello_serial(&service);
    requests.insert("a", request);
    let (_, request, b) = send_hello_serial(&service);
    requests.insert("b", request);

    assert_eq!(requests.len(), 2);

    let (_, request, a2) = send_hello_serial(&service);
    requests.insert("a", request);

    assert_eq!(requests.len(), 2);
    assert!(!is_waiting(&service, a));
    assert!(is_waiting(&service, a2));
    assert!(is_waiting(&service, b));
    assert!(requests.is_pending("a"));
    assert!(requests.is_pending("b"));

    // A request which could not be sent still cancels the one before it.
    requests.insert("b", Request::new());
    assert!(!is_waiting(&service, b));
    assert!(!requests.is_pending("b"));
    assert_eq!(requests.len(), 1);

    assert!(requests.remove("a"));
    assert!(!is_waiting(&service, a2));
    assert!(!requests.remove("a"));
    assert!(requests.is_empty());
}

/// A key is forgotten once its request completes.
#[test]
fn keyed_requests_forget_completed() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut requests = KeyedRequests::new();

    let (outcomes, request, a) = send_hello_serial(&service);
    requests.insert(1u32, request);
    let (_, request, _) = send_hello_serial(&service);
    requests.insert(2u32, request);

    respond(&service, a);

    assert_eq!(*outcomes.borrow(), [None]);
    assert!(!requests.is_pending(&1));
    assert!(requests.is_pending(&2));
    assert_eq!(requests.len(), 1);
    assert_eq!(requests.keys().copied().collect::<Vec<_>>(), [2]);
    // NB: Completed, so there is nothing to cancel.
    assert!(!requests.remove(&1));
}

/// Dropping or clearing the set cancels every request still in flight.
#[test]
fn keyed_requests_drop_and_clear_cancel_all() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut requests = KeyedRequests::new();

    for key in 0..3u32 {
        let (_, request, _) = send_hello_serial(&service);
        requests.insert(key, request);
    }

    assert_eq!(requests.len(), 3);
    drop(requests);
    assert!(!has_pending(&service));

    let mut requests = KeyedRequests::new();

    for key in 0..3u32 {
        let (_, request, _) = send_hello_serial(&service);
        requests.insert(key, request);
    }

    requests.clear();
    assert!(requests.is_empty());
    assert!(!has_pending(&service));
}

/// Deliver a hello stating application version `app`, returning the request
/// the client answered it with.
fn hello_app(service: &Service<MockImpl>, app: u32) -> api::RequestHeader {
    let header = api::ResponseHeader {
        format: 0,
        app,
        ..broadcast_header(MessageId::SERVER_HELLO)
    };

    deliver(service, Mode::Binary, &header, b"");

    let ops = take_ops();

    let Some(Op::Send(mode, bytes)) = ops.first() else {
        panic!("Expected a negotiation request, got {ops:?}");
    };

    let mut at = 0;
    let request: api::RequestHeader = format::decode_envelope(*mode, bytes, &mut at).unwrap();
    assert_eq!(request.id, MessageId::NEGOTIATE.get());

    // NB: Whatever else happened is checked by the caller.
    for op in ops.into_iter().skip(1) {
        push_op(op);
    }

    request
}

/// Answer a negotiation the way a server which accepted it does.
fn accept_negotiation(service: &Service<MockImpl>, request: &api::RequestHeader) {
    let response = api::ResponseHeader {
        serial: request.serial,
        broadcast: 0,
        ..broadcast_header(MessageId::SERVER_HELLO)
    };

    deliver(service, Mode::Binary, &response, b"");
}

/// A server which states another application version leaves the client
/// incompatible rather than open, and it does not reconnect on its own.
#[test]
fn mismatched_app_version_is_incompatible() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.app_version(7).build();
    take_ops();

    let states = Rc::new(RefCell::new(Vec::new()));

    let (_, _listener) = service.handle().on_state_change({
        let states = states.clone();
        move |state: State| states.borrow_mut().push(state)
    });

    let request = hello_app(&service, 8);

    // The server still hears which version this client states.
    assert_eq!(request.app, 7);
    assert_eq!(take_ops(), vec![Op::Close]);

    let incompatible = State::Incompatible {
        server: 8,
        client: 7,
    };

    assert_eq!(state(&service), incompatible);
    assert_eq!(*states.borrow(), vec![incompatible]);
    assert_eq!(service.handle().generation(), 0);

    // Connecting again would only reach the same server.
    assert_eq!(pending_timers(), 0);
    assert_eq!(fire_timers(), 0);
    assert!(take_ops().is_empty());
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());

    // Opening again on purpose does try again.
    service.open();
    assert_eq!(take_ops(), vec![Op::Connect(String::from(URL))]);

    let request = hello_app(&service, 7);
    accept_negotiation(&service, &request);
    assert_eq!(state(&service), State::Open);
}

/// Matching application versions open the connection as usual.
#[test]
fn matching_app_version_opens() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.app_version(7).build();
    take_ops();

    let request = hello_app(&service, 7);
    assert_eq!(request.app, 7);
    assert!(take_ops().is_empty());

    accept_negotiation(&service, &request);
    assert_eq!(state(&service), State::Open);
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// The check only applies when both peers state an application version.
#[test]
fn unstated_app_version_is_not_checked() {
    for (client, server) in [(0, 8), (7, 0)] {
        let (builder, errors) = builder(Connect::url(String::from(URL)));
        let service = builder.app_version(client).build();
        take_ops();

        let request = hello_app(&service, server);
        assert_eq!(request.app, client);
        assert!(take_ops().is_empty());

        accept_negotiation(&service, &request);
        assert_eq!(
            state(&service),
            State::Open,
            "client {client}, server {server}"
        );
        assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
    }
}

/// A channel stays closed while the client is incompatible, and opens again
/// once the service is opened on purpose against a server which matches.
#[test]
fn incompatible_keeps_channels_closed() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.app_version(7).build();
    take_ops();

    let (events, channel) = open_channel(&service);
    open_session(&service);
    accept_channel(&service, 7);
    assert_eq!(take_events(&events), vec![String::from("opened 7")]);

    // The connection drops and comes back to a server from another build.
    service.shared.close_and_reconnect().unwrap();
    assert_eq!(take_events(&events), vec![String::from("closed")]);
    assert_eq!(fire_timers(), 1);
    take_ops();

    hello_app(&service, 8);
    assert_eq!(take_ops(), vec![Op::Close]);
    assert!(state(&service).is_incompatible());
    assert!(!channel.is_open());
    assert!(take_events(&events).is_empty());

    service.open();
    take_ops();
    let request = hello_app(&service, 7);
    accept_negotiation(&service, &request);
    accept_channel(&service, 9);

    assert_eq!(take_events(&events), vec![String::from("opened 9")]);
    assert!(channel.is_open());
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A request whose response is owned, so that it can be awaited with
/// [`Handle::call`].
#[derive(musli::Encode, musli::Decode)]
struct Fetch;

enum FetchEndpoint {}

impl api::Decodable for FetchEndpoint {
    type Type<'de> = Unrelated;

    fn __do_not_implement_decodable() {}
}

impl api::Endpoint for FetchEndpoint {
    const ID: MessageId = match MessageId::new(3) {
        Some(id) => id,
        None => panic!("Invalid message id"),
    };

    type Response<'de> = Unrelated;

    fn __do_not_implement_endpoint() {}
}

impl api::Request for Fetch {
    type Endpoint = FetchEndpoint;

    fn __do_not_implement_request() {}
}

/// A waker which counts how many times it is woken.
#[derive(Default)]
struct CountingWaker(std::sync::atomic::AtomicUsize);

impl std::task::Wake for CountingWaker {
    fn wake(self: alloc::sync::Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &alloc::sync::Arc<Self>) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl CountingWaker {
    fn new() -> (alloc::sync::Arc<Self>, Waker) {
        let counter = alloc::sync::Arc::new(Self::default());
        let waker = Waker::from(counter.clone());
        (counter, waker)
    }

    fn count(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

fn poll_call<T>(call: &mut Call<T>, waker: &Waker) -> Poll<Result<T>>
where
    for<'de> T: DecodeBody<'de>,
{
    Pin::new(call).poll(&mut Context::from_waker(waker))
}

/// Poll a call which is expected to have resolved already.
#[track_caller]
fn ready<T>(call: &mut Call<T>) -> Result<T>
where
    for<'de> T: DecodeBody<'de>,
{
    let (_, waker) = CountingWaker::new();

    match poll_call(call, &waker) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("Expected the call to have resolved"),
    }
}

/// The header of a successful response to `serial`.
fn response_header(serial: u32) -> api::ResponseHeader {
    api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: 0,
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    }
}

const FETCHED: Unrelated = Unrelated { a: 1, b: 2, c: 3 };

/// A call waits for its response, wakes the task once it arrives, and
/// resolves to the response decoded as an owned value.
#[test]
fn call_resolves_to_decoded_response() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut call = service.handle().call(Fetch);
    let serial = last_serial();
    assert!(call.is_pending());

    let (counter, waker) = CountingWaker::new();
    assert!(poll_call(&mut call, &waker).is_pending());
    assert_eq!(counter.count(), 0);

    let body = encode_payload(Format::DEFAULT, &FETCHED);
    deliver(&service, Mode::Binary, &response_header(serial), &body);

    assert_eq!(counter.count(), 1);
    assert!(!call.is_pending());

    let Poll::Ready(result) = poll_call(&mut call, &waker) else {
        panic!("Expected the call to resolve");
    };

    // NB: The response outlives the call and the packet it came in.
    drop(call);
    assert_eq!(result.unwrap(), FETCHED);
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A call made before the connection is open resolves straight away to an
/// error saying that it was never sent.
#[test]
fn call_before_open_is_not_connected() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();

    let mut call = service.handle().call(Fetch);
    assert!(!call.is_pending());

    let error = ready(&mut call).unwrap_err();
    assert_kind(&error, "not_connected");
    assert!(take_ops().is_empty());
}

/// A call over a channel which is not open is never sent.
#[test]
fn call_over_closed_channel_is_not_connected() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    // NB: Not answered, so the channel never opens.
    let channel = service.handle().channel().open();
    take_ops();

    let error = ready(&mut channel.call(Fetch)).unwrap_err();
    assert_kind(&error, "not_connected");
    assert_eq!(error.to_string(), "WebSocket request over closed channel");
    assert!(take_ops().is_empty());
}

/// A call which the server fails resolves to its server error, including a
/// typed one.
#[test]
fn call_resolves_to_server_error() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut call = service.handle().call(Fetch);
    let serial = last_serial();

    let header = error_header(serial, MessageId::ERROR_MESSAGE, Format::DEFAULT);
    deliver(&service, Mode::Binary, &header, &error_body("No such user"));

    let error = ready(&mut call).unwrap_err();
    assert_kind(&error, "server");
    assert_eq!(error.as_server_error(), Some("No such user"));
    assert_eq!(error.server_error::<Refusal>().unwrap(), None);

    let mut call = service.handle().call(Fetch);
    let serial = last_serial();

    let format = Format::DEFAULT;
    let payload = encode_payload(format, &Refusal::NotFound);
    let header = error_header(serial, MessageId::TYPED_ERROR_MESSAGE, format);
    deliver(
        &service,
        Mode::Binary,
        &header,
        &typed_error_body(format, "Not found", &payload),
    );

    let error = ready(&mut call).unwrap_err();
    assert_kind(&error, "server");
    assert_eq!(
        error.server_error::<Refusal>().unwrap(),
        Some(Refusal::NotFound)
    );
}

/// A response which does not decode as the response type is a decode error.
#[test]
fn call_with_malformed_response_is_decode_error() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let valid = encode_payload(Format::DEFAULT, &FETCHED);

    for body in [&[][..], &valid[..valid.len() - 1]] {
        let mut call = service.handle().call(Fetch);
        let serial = last_serial();

        deliver(&service, Mode::Binary, &response_header(serial), body);

        let error = ready(&mut call).unwrap_err();
        assert_kind(&error, "decode");
    }
}

/// A call pending when the connection closes resolves to the connection
/// being lost, and wakes the task waiting for it.
#[test]
fn call_is_lost_on_close() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let mut call = service.handle().call(Fetch);

    let (counter, waker) = CountingWaker::new();
    assert!(poll_call(&mut call, &waker).is_pending());

    service.shared.close_pending();
    assert_eq!(counter.count(), 1);

    let error = ready(&mut call).unwrap_err();
    assert_kind(&error, "connection_lost");
}

/// Dropping a call cancels the request, and a response arriving later is not
/// mistaken for the response to anything else.
#[test]
fn dropping_call_cancels_the_request() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);

    let call = service.handle().call(Fetch);
    let serial = last_serial();
    assert!(has_pending(&service));

    drop(call);
    assert!(!has_pending(&service));

    let body = encode_payload(Format::DEFAULT, &FETCHED);
    deliver(&service, Mode::Binary, &response_header(serial), &body);
    assert!(!has_pending(&service));
}
