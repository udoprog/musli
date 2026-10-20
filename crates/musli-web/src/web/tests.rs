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

/// Open a channel, recording what its callback is called with.
fn open_channel(service: &Service<MockImpl>) -> (Outcomes, Request) {
    let outcomes = Outcomes::default();

    let request = service
        .handle()
        .channel()
        .on_open({
            let outcomes = outcomes.clone();
            move |result: Result<Channel<MockImpl>>| outcomes.borrow_mut().push(outcome(result))
        })
        .send();

    (outcomes, request)
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

    let (outcomes, _request) = open_channel(&service);

    assert_eq!(*outcomes.borrow(), vec![Some(String::from("Send failed"))]);
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
/// channel.
#[test]
fn channel_send_error_after_pending_requests_were_failed() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    open_session(&service);
    fail_pending_on_send(&service);

    let (outcomes, _request) = open_channel(&service);

    assert_eq!(
        *outcomes.borrow(),
        vec![Some(String::from("Connection closed"))]
    );
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

    let channel = Channel {
        handle: service.handle().clone(),
        id: ChannelId::NONE,
        generation: service.handle().generation(),
    };

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

/// Open a channel and answer the request with `id`, returning the channel.
fn connect_channel(service: &Service<MockImpl>, id: u16) -> Channel<MockImpl> {
    let channel = Rc::new(RefCell::new(None));

    let _request = service
        .handle()
        .channel()
        .on_open({
            let channel = channel.clone();
            move |result: Result<Channel<MockImpl>>| *channel.borrow_mut() = Some(result.unwrap())
        })
        .send();

    let response = api::ResponseHeader {
        version: api::VERSION,
        serial: last_serial(),
        broadcast: 0,
        error: 0,
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::from_u16(id),
    };

    deliver(service, Mode::Binary, &response, b"");
    let channel = channel.borrow_mut().take();
    channel.expect("Channel was not opened")
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

/// A channel remembers the generation it was opened in, and is closed for
/// good once the connection has reopened.
#[test]
fn channel_from_earlier_connection_is_stale() {
    let (builder, errors) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();
    open_session(&service);

    let channel = connect_channel(&service, 7);
    assert_eq!(channel.id(), ChannelId::from_u16(7));
    assert_eq!(channel.generation(), 1);
    assert!(channel.is_open());

    service.shared.close_and_reconnect().unwrap();
    assert!(!channel.is_open());
    assert_eq!(fire_timers(), 1);
    take_ops();
    open_session(&service);

    assert_eq!(channel.generation(), 1);
    assert!(!channel.is_open());

    let failures = Failures::default();

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
    assert!(take_ops().is_empty());

    // The new connection may hand the same id to another channel, which
    // dropping the stale one must leave alone.
    let current = connect_channel(&service, 7);
    assert_eq!(current.generation(), 2);
    assert!(current.is_open());

    drop(channel);
    assert!(!disconnects(&take_ops(), 7));
    assert!(current.is_open());

    drop(current);
    assert!(disconnects(&take_ops(), 7));
    assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
}

/// A channel opened over the current connection is still disconnected when
/// dropped.
#[test]
fn channel_is_disconnected_on_drop() {
    let (builder, _) = builder(Connect::url(String::from(URL)));
    let service = builder.build();
    take_ops();
    open_session(&service);

    let channel = connect_channel(&service, 3);
    take_ops();
    drop(channel);
    assert!(disconnects(&take_ops(), 3));
}
