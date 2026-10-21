use core::convert::Infallible;
use core::fmt;
use core::future::Future;

use alloc::string::ToString;
use alloc::vec::Vec;

use tokio::sync::oneshot;

use crate::api::{MessageId, Mode};

use super::{
    ClientImpl, Command, Error, ErrorKind, Message, Pending, RawPacket, Result, Service,
    SocketImpl, connect, sealed_client, sealed_socket,
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
    ];

    for (error, expected) in cases {
        assert_eq!(flags(&error), expected, "{error}");
    }
}
