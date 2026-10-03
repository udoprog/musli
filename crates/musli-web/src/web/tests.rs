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
        push_op(Op::Connect(url.to_string()));
        Ok(MockSocket)
    }

    fn send(&self, mode: Mode, data: &[u8]) -> Result<(), Error> {
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
        Ok(Location {
            protocol: String::from("http:"),
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
