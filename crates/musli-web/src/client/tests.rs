use core::convert::Infallible;
use core::fmt;
use core::future::Future;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::Bytes;
use tokio::sync::oneshot;

use crate::api::{self, ChannelId, Format, MessageId, Mode};
use crate::format;

use super::{
    ClientImpl, Command, Error, ErrorKind, Message, Pending, RawPacket, Refused, Result, Service,
    SocketImpl, State, connect, sealed_client, sealed_socket,
};

#[derive(Debug)]
struct MockError;

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mock error")
    }
}

impl core::error::Error for MockError {}

/// A socket which accepts everything sent and never receives anything.
struct MockSocket;

impl sealed_socket::Sealed for MockSocket {}

impl SocketImpl for MockSocket {
    type Error = MockError;

    async fn recv(&mut self) -> Option<Result<Message, Self::Error>> {
        core::future::pending().await
    }

    // NB: Spelled out to match the `Send` bound of the trait.
    #[allow(clippy::manual_async_fn)]
    fn send(
        &mut self,
        _: Mode,
        _: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        async { Ok(()) }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct MockImpl;

impl sealed_client::Sealed for MockImpl {}

impl ClientImpl for MockImpl {
    type Error = MockError;
    type Socket = MockSocket;
    type Connector = Infallible;
    type Headers = ();

    fn header(_: &mut Self::Headers, _: &str, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn connect(_: &str, _: &Self::Headers) -> Result<Self::Socket, Self::Error> {
        Ok(MockSocket)
    }

    async fn connect_with(
        connector: &mut Self::Connector,
        _: &Self::Headers,
    ) -> Result<Self::Socket, Self::Error> {
        match *connector {}
    }
}

fn request(serial: u32) -> (Command, oneshot::Receiver<Result<RawPacket>>) {
    let (reply, rx) = oneshot::channel();

    let command = Command::Send {
        serial,
        mode: Mode::Binary,
        data: Vec::new(),
        pending: Pending::Request {
            id: MessageId::CONNECT,
            reply,
        },
    };

    (command, rx)
}

/// A request replaced by another under the same serial is cancelled.
///
/// Serials only repeat once the counter wraps around, so this is arranged by
/// hand.
#[tokio::test]
async fn replaced_request_is_cancelled() {
    let mut service: Service<MockImpl> = connect::<MockImpl>("mock://").build();
    service.socket = Some(MockSocket);

    let (first, mut first_rx) = request(7);
    let (second, mut second_rx) = request(7);

    service.command(first).await;
    assert!(first_rx.try_recv().is_err(), "Still pending");

    service.command(second).await;

    let error = first_rx
        .try_recv()
        .expect("The replaced request must be answered")
        .expect_err("The replaced request must fail");

    assert!(error.is_cancelled());
    assert!(!error.is_connection_lost());
    assert!(!error.is_not_connected());
    assert!(!error.is_decode());
    assert!(error.as_server_error().is_none());
    assert_eq!(error.to_string(), "Request cancelled");

    // The replacement is the one still pending.
    assert!(second_rx.try_recv().is_err(), "Still pending");
}

/// Every accessor holds for exactly the kinds it documents.
#[test]
fn accessors_are_exclusive() {
    fn flags(error: &Error) -> [bool; 5] {
        [
            error.is_not_connected(),
            error.is_connection_lost(),
            error.is_cancelled(),
            error.as_server_error().is_some(),
            error.is_decode(),
        ]
    }

    let cases = [
        (Error::not_connected(), [true, false, false, false, false]),
        (
            Error::new(ErrorKind::ConnectionLost("Connection closed")),
            [false, true, false, false, false],
        ),
        (
            Error::new(ErrorKind::Cancelled),
            [false, false, true, false, false],
        ),
        (
            Error::new(ErrorKind::EmptyPacket),
            [false, false, false, false, true],
        ),
        (
            Error::new(ErrorKind::UnknownFormat(200)),
            [false, false, false, false, true],
        ),
        (Error::message("other"), [false; 5]),
        (Error::incompatible(1, 2), [false; 5]),
        (
            Error::refused(Refused {
                code: 4000,
                reason: Arc::from("no"),
            }),
            [false; 5],
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(flags(&error), expected, "{error}");
    }
}

/// A service with an established mock socket over which a negotiation has
/// been sent, and a request pending.
async fn negotiating(service: &mut Service<MockImpl>) -> oneshot::Receiver<Result<RawPacket>> {
    service.established(MockSocket);
    service.send_negotiate().await;
    assert!(service.negotiating);

    let (command, rx) = request(100);
    service.command(command).await;
    rx
}

/// Answer the pending negotiation, which opens the connection.
fn negotiated(service: &mut Service<MockImpl>) {
    let serial = service
        .pending
        .iter()
        .find_map(|(serial, pending)| {
            matches!(pending, Pending::Negotiate { .. }).then_some(*serial)
        })
        .expect("A negotiation must be pending");

    let header = api::ResponseHeader {
        version: api::VERSION,
        serial,
        broadcast: 0,
        error: 0,
        format: Format::DEFAULT.to_u8(),
        channel: ChannelId::NONE,
        app: 0,
    };

    let mut buf = Vec::new();
    format::encode_envelope(Mode::Binary, &mut buf, &header).unwrap();
    let post = service.message(Mode::Binary, Bytes::from(buf));
    assert!(matches!(post, Ok(super::Post::None)));
    assert!(service.handle.state().is_open());
    assert!(!service.negotiating);
}

/// A close in the application range during negotiation is a refusal, which
/// fails pending requests and stops the service for good.
#[tokio::test]
async fn refusal_during_negotiation() {
    let mut service = connect::<MockImpl>("mock://").build();
    let mut rx = negotiating(&mut service).await;

    service
        .close_received(Some((4003, String::from("Impersonation has ended"))))
        .await;

    let State::Refused(refused) = service.handle.state() else {
        panic!("Expected a refusal, got {:?}", service.handle.state());
    };

    assert_eq!(refused.code(), 4003);
    assert_eq!(refused.reason(), "Impersonation has ended");
    assert!(service.handle.state().is_refused());
    assert!(service.closed);
    assert!(service.next_attempt.is_none());
    assert!(service.socket.is_none());

    let error = rx.try_recv().unwrap().unwrap_err();
    assert!(error.is_refused());
    assert_eq!(error.as_refused(), Some(&refused));

    let error = service.run().await.unwrap_err();
    assert!(error.is_refused());
    assert_eq!(error.as_refused().map(Refused::code), Some(4003));
    assert_eq!(
        error.to_string(),
        "Connection refused by server (4003): Impersonation has ended"
    );

    let error = service.handle.wait_until_open().await.unwrap_err();
    assert!(error.is_refused());

    // NB: Kept once the service is gone, since it explains why.
    let handle = service.handle.clone();
    drop(service);
    assert!(handle.state().is_refused());
}

/// A close in the application range once the connection is open is a dropped
/// connection, which is retried.
#[tokio::test]
async fn application_close_after_open_is_dropped() {
    let mut service = connect::<MockImpl>("mock://").build();
    let mut rx = negotiating(&mut service).await;
    negotiated(&mut service);

    service
        .close_received(Some((4000, String::from("later"))))
        .await;

    assert_eq!(service.handle.state(), State::Closed);
    assert!(!service.closed);
    assert!(service.next_attempt.is_some());
    assert!(service.stopped.is_none());

    let error = rx.try_recv().unwrap().unwrap_err();
    assert!(error.is_connection_lost());
    assert!(!error.is_refused());
}

/// A close outside the application range during negotiation, or one without
/// a frame, is a dropped connection.
#[tokio::test]
async fn other_close_during_negotiation_is_dropped() {
    for frame in [
        Some((1008, String::from("policy"))),
        Some((3999, String::from("registered"))),
        Some((5000, String::from("reserved"))),
        None,
    ] {
        let mut service = connect::<MockImpl>("mock://").build();
        let mut rx = negotiating(&mut service).await;

        service.close_received(frame.clone()).await;

        assert_eq!(service.handle.state(), State::Closed, "{frame:?}");
        assert!(!service.closed, "{frame:?}");
        assert!(service.next_attempt.is_some(), "{frame:?}");

        let error = rx.try_recv().unwrap().unwrap_err();
        assert!(error.is_connection_lost(), "{frame:?}");
    }
}

/// A close in the application range before anything was negotiated is a
/// dropped connection.
#[tokio::test]
async fn application_close_before_negotiation_is_dropped() {
    let mut service = connect::<MockImpl>("mock://").build();
    service.established(MockSocket);

    service
        .close_received(Some((4000, String::from("early"))))
        .await;

    assert_eq!(service.handle.state(), State::Closed);
    assert!(!service.closed);
}

/// The metadata closure is called for every negotiation.
#[tokio::test]
async fn metadata_is_built_per_negotiation() {
    let calls = Arc::new(AtomicUsize::new(0));

    let mut service = connect::<MockImpl>("mock://")
        .metadata({
            let calls = calls.clone();
            move |metadata| {
                let n = calls.fetch_add(1, Ordering::SeqCst);
                metadata.insert("attempt", &n.to_string());
            }
        })
        .build();

    service.established(MockSocket);
    service.send_negotiate().await;
    service.disconnect().await;
    service.established(MockSocket);
    service.send_negotiate().await;

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(service.stopped.is_none());
}

/// Invalid metadata stops the service for good.
#[tokio::test]
async fn invalid_metadata_stops_the_service() {
    let errors = Arc::new(AtomicUsize::new(0));

    let mut service = connect::<MockImpl>("mock://")
        .on_error({
            let errors = errors.clone();
            move |error: Error| {
                assert!(error.is_invalid_metadata());
                errors.fetch_add(1, Ordering::SeqCst);
            }
        })
        .metadata(|metadata| metadata.insert("Upper", "x"))
        .build();

    let mut rx = negotiating_invalid(&mut service).await;

    assert_eq!(errors.load(Ordering::SeqCst), 1);
    assert_eq!(service.handle.state(), State::Closed);
    assert!(service.closed);
    assert!(service.next_attempt.is_none());
    assert!(service.socket.is_none());
    assert!(rx.try_recv().unwrap().unwrap_err().is_invalid_metadata());

    let error = service.run().await.unwrap_err();
    assert!(error.is_invalid_metadata());
    assert!(core::error::Error::source(&error).is_some());
}

async fn negotiating_invalid(
    service: &mut Service<MockImpl>,
) -> oneshot::Receiver<Result<RawPacket>> {
    service.established(MockSocket);

    let (command, rx) = request(100);
    service.command(command).await;

    service.send_negotiate().await;
    assert!(!service.negotiating);
    rx
}
