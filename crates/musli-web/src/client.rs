//! The generic asynchronous client implementation.
//!
//! This implements the client side of the same websocket protocol which is
//! implemented by [`web`] for browsers, but built for native asynchronous
//! runtimes instead of the browser event loop.
//!
//! This is specialized over the `T` parameter through modules such as:
//!
//! * [`tungstenite029`] for `tokio-tungstenite` `0.29.x`.
//! * [`tungstenite030`] for `tokio-tungstenite` `0.30.x`.
//!
//! [`tungstenite029`]: crate::tungstenite029
//! [`tungstenite030`]: crate::tungstenite030
//! [`web`]: <https://docs.rs/musli-web/latest/musli_web/web/>
//!
//! # Overview
//!
//! A [`Service`] is a driver which owns the underlying socket. It has to be
//! driven by calling [`Service::run`], which is typically done in a dedicated
//! task.
//!
//! A [`Handle`] is a cheap and [`Clone`]-able handle to the service which can be
//! shared and moved freely between tasks. It is used to perform requests, open
//! channels, and to listen for broadcasts and state changes.

use core::cell::Cell;
use core::fmt;
use core::future::Future;
use core::marker::PhantomData;
use core::{any, mem};

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use bytes::Bytes;
use rand::prelude::*;
use rand::rngs::SmallRng;
use slab::Slab;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{Duration, Instant};

use crate::api::{self, ChannelId, DecodeBody, Event, Format, MessageId, Mode};
use crate::format;

/// The initial reconnect timeout.
const INITIAL_TIMEOUT: Duration = Duration::from_millis(250);
/// The maximum reconnect timeout.
const MAX_TIMEOUT: Duration = Duration::from_millis(4000);
/// The maximum amount of fuzz added to a reconnect timeout.
const MAX_FUZZ: u64 = 50;
/// The default seed used for reconnect fuzzing.
const DEFAULT_SEED: u64 = 0xdeadbeef;

/// An empty request body.
#[non_exhaustive]
pub struct EmptyBody;

/// An empty callback.
#[non_exhaustive]
pub struct EmptyCallback;

/// A message received over the underlying socket.
///
/// The variant is what tells the client which [`Mode`] the envelope of the
/// message is spelled in, see the [wire format].
///
/// [wire format]: crate::api#wire-format
///
/// NB: The variants are constructed by the transport bindings, such as
/// [`tungstenite029`], so none are constructed when the generic core is built
/// on its own.
///
/// [`tungstenite029`]: crate::tungstenite029
#[cfg_attr(
    not(any(feature = "tungstenite029", feature = "tungstenite030")),
    allow(dead_code)
)]
pub(crate) enum Message {
    /// A text message was received, which is what [`Mode::Text`] uses.
    Text(Bytes),
    /// A binary message was received.
    Binary(Bytes),
    /// A ping message was received.
    ///
    /// Implementations are expected to respond with a pong on their own.
    Ping,
    /// A pong message was received.
    Pong,
    /// A close message was received.
    Close,
    /// A message the protocol has no meaning for was received, which tears the
    /// connection down.
    Unsupported,
}

pub(crate) mod sealed_socket {
    pub trait Sealed {}
}

pub(crate) trait SocketImpl
where
    Self: 'static + Send + Sized + self::sealed_socket::Sealed,
{
    #[doc(hidden)]
    type Error;

    /// Receive the next message.
    ///
    /// The returned future must be cancel safe, since it is used in a `select!`
    /// loop together with the command queue of the service.
    #[doc(hidden)]
    fn recv(&mut self) -> impl Future<Output = Option<Result<Message, Self::Error>>> + Send + '_;

    /// Send a message in the given [`Mode`] and flush it.
    ///
    /// [`Mode::Binary`] writes a binary frame and [`Mode::Text`] a text frame,
    /// which is what carries the mode between peers.
    #[doc(hidden)]
    fn send(
        &mut self,
        mode: Mode,
        data: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;

    /// Close the socket.
    #[doc(hidden)]
    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;
}

pub(crate) mod sealed_client {
    pub trait Sealed {}
}

/// Central trait for asynchronous client integration.
///
/// Since websocket clients are provided by many different crates, this trait
/// abstracts over the details of establishing and driving one.
///
/// The corresponding modules provide integrations:
///
/// * [`tungstenite029`] for `tokio-tungstenite` `0.29.x`.
/// * [`tungstenite030`] for `tokio-tungstenite` `0.30.x`.
///
/// [`tungstenite029`]: crate::tungstenite029
/// [`tungstenite030`]: crate::tungstenite030
pub trait ClientImpl
where
    Self: 'static + Copy + Sized + self::sealed_client::Sealed,
{
    #[doc(hidden)]
    type Error: 'static + Send + Sync + core::error::Error;

    #[doc(hidden)]
    #[allow(private_bounds)]
    type Socket: SocketImpl<Error = Self::Error>;

    /// A caller-supplied way of establishing the underlying socket, used in
    /// place of connecting to the url. See constructors such as
    /// [`tungstenite029::connect_with`].
    ///
    /// [`tungstenite029::connect_with`]: crate::tungstenite029::connect_with
    #[doc(hidden)]
    type Connector: 'static + Send;

    /// The headers added to the websocket handshake, see
    /// [`ServiceBuilder::header`].
    #[doc(hidden)]
    type Headers: 'static + Send + Sync + Default;

    /// Validate a header and add it to `headers`.
    #[doc(hidden)]
    fn header(headers: &mut Self::Headers, name: &str, value: &str) -> Result<(), Self::Error>;

    #[doc(hidden)]
    fn connect<'a>(
        url: &'a str,
        headers: &'a Self::Headers,
    ) -> impl Future<Output = Result<Self::Socket, Self::Error>> + Send + 'a;

    #[doc(hidden)]
    fn connect_with<'a>(
        connector: &'a mut Self::Connector,
        headers: &'a Self::Headers,
    ) -> impl Future<Output = Result<Self::Socket, Self::Error>> + Send + 'a;
}

/// Construct a new [`ServiceBuilder`] which will connect to `url`.
pub fn connect<T>(url: impl AsRef<str>) -> ServiceBuilder<T, EmptyCallback>
where
    T: ClientImpl,
{
    ServiceBuilder {
        url: url.as_ref().to_string(),
        connector: None,
        headers: T::Headers::default(),
        invalid_header: None,
        on_error: EmptyCallback,
        reconnect: true,
        seed: DEFAULT_SEED,
        format: Format::DEFAULT,
        mode: Mode::DEFAULT,
        app: 0,
        _marker: PhantomData,
    }
}

/// Construct a new [`ServiceBuilder`] which establishes its socket through
/// `connector` instead of connecting to `url`.
///
/// What `url` is used for during the handshake is up to the implementation; it
/// is never connected to directly.
#[cfg_attr(
    not(any(feature = "tungstenite029", feature = "tungstenite030")),
    allow(dead_code)
)]
pub(crate) fn connect_with<T>(
    url: impl AsRef<str>,
    connector: T::Connector,
) -> ServiceBuilder<T, EmptyCallback>
where
    T: ClientImpl,
{
    ServiceBuilder {
        connector: Some(connector),
        ..connect(url)
    }
}

/// The state of the connection.
///
/// A listener for state changes can be set up through [`Handle::state`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum State {
    /// The connection is open.
    Open,
    /// The connection is closed.
    Closed,
    /// The server states an application version other than the one this
    /// client was built with, see [`ServiceBuilder::app_version`].
    ///
    /// The connection is closed and [`Service::run`] returns an error for
    /// which [`Error::is_incompatible`] holds, since connecting again reaches
    /// the same server. This state is kept once the service is gone.
    Incompatible {
        /// The application version the server states.
        server: u32,
        /// The application version this client states.
        client: u32,
    },
}

impl State {
    /// Check if the state is open.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::State;
    ///
    /// assert!(State::Open.is_open());
    /// assert!(!State::Closed.is_open());
    /// ```
    #[inline]
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Check if the connection was refused because the server states another
    /// application version, see [`State::Incompatible`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::State;
    ///
    /// assert!(State::Incompatible { server: 2, client: 1 }.is_incompatible());
    /// assert!(!State::Closed.is_incompatible());
    /// ```
    #[inline]
    pub fn is_incompatible(&self) -> bool {
        matches!(self, Self::Incompatible { .. })
    }
}

/// Trait governing how callbacks are called.
pub trait Callback<I>
where
    Self: 'static + Send + Sync,
{
    /// Call the callback.
    fn call(&self, input: I);
}

impl<I> Callback<I> for EmptyCallback {
    #[inline]
    fn call(&self, _: I) {}
}

impl<F, I> Callback<I> for F
where
    F: 'static + Send + Sync + Fn(I),
{
    #[inline]
    fn call(&self, input: I) {
        self(input)
    }
}

/// Error type for the client.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[inline]
    const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    #[inline]
    const fn not_connected() -> Self {
        Self::new(ErrorKind::NotConnected("Client is not connected"))
    }

    /// Check if the error is caused by an empty packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// let e = packet.decode::<u32>().unwrap_err();
    ///
    /// assert!(e.is_empty_packet());
    /// assert!(e.is_decode());
    /// ```
    #[inline]
    pub fn is_empty_packet(&self) -> bool {
        matches!(self.kind, ErrorKind::EmptyPacket)
    }

    /// Check if the request was never sent, since there was no open
    /// connection to send it over.
    ///
    /// This is the error which is produced if a request is performed while the
    /// service is not connected or after it has been dropped. Since the server
    /// never saw the request, it is safe to send it again once the connection
    /// is open. See [`Handle::wait_until_open`] for how to wait until the
    /// connection is available.
    #[inline]
    pub fn is_not_connected(&self) -> bool {
        matches!(self.kind, ErrorKind::NotConnected(..))
    }

    /// Check if the connection closed before a response to the request
    /// arrived.
    ///
    /// Unlike [`is_not_connected`], the request was sent, or was being sent,
    /// so the server may or may not have handled it. This is the error every
    /// pending request gets when the connection closes or when the service is
    /// dropped or closed.
    ///
    /// [`is_not_connected`]: Self::is_not_connected
    #[inline]
    pub fn is_connection_lost(&self) -> bool {
        matches!(self.kind, ErrorKind::ConnectionLost(..))
    }

    /// Check if the request was cancelled before it was answered, since
    /// another request took its place.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        matches!(self.kind, ErrorKind::Cancelled)
    }

    /// Check if the error is caused by something received which could not be
    /// decoded.
    ///
    /// This covers a response or broadcast whose body does not decode as the
    /// requested type, an [empty packet](Self::is_empty_packet), a frame from
    /// the server whose header or error message could not be decoded, and a
    /// message body in an unknown format. Failing to decode a
    /// [typed error](Self::server_error) is one too.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// let e = packet.decode::<u32>().unwrap_err();
    ///
    /// assert!(e.is_decode());
    /// ```
    #[inline]
    pub fn is_decode(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::EmptyPacket
                | ErrorKind::UnknownFormat(..)
                | ErrorKind::DecodeResponseHeader(..)
                | ErrorKind::DecodeErrorMessage(..)
                | ErrorKind::DecodePacket(..)
        )
    }

    /// Check if the error is caused by a header passed to
    /// [`ServiceBuilder::header`] which cannot be sent, such as a name or a
    /// value which is not valid in HTTP.
    ///
    /// This is what [`Service::run`] and [`Service::try_connect`] fail with
    /// right away, without trying to connect, since every attempt would send
    /// the same header. The underlying error is available through
    /// [`core::error::Error::source`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::tungstenite030;
    ///
    /// # #[tokio::main(flavor = "current_thread")] async fn main() {
    /// let mut service = tungstenite030::connect("ws://127.0.0.1:8080/ws")
    ///     .header("Cookie", "session=\nabc")
    ///     .build();
    ///
    /// let error = service.try_connect().await.unwrap_err();
    /// assert!(error.is_invalid_header());
    /// # }
    /// ```
    #[inline]
    pub fn is_invalid_header(&self) -> bool {
        matches!(self.kind, ErrorKind::InvalidHeader(..))
    }

    /// Check if the server states an application version other than the one
    /// this client states, see [`ServiceBuilder::app_version`].
    ///
    /// This is what [`Service::run`] and [`Handle::wait_until_open`] fail with
    /// once the service has found out, and the versions on either side are
    /// available through [`State::Incompatible`].
    #[inline]
    pub fn is_incompatible(&self) -> bool {
        matches!(self.kind, ErrorKind::Incompatible { .. })
    }

    /// Check if the error is a server error, and if so return the message
    /// reported by the server.
    ///
    /// Server errors are produced by a [`Handler`] which returns an error or
    /// which indicates that it does not support the request being made.
    ///
    /// This is the message of a [typed error](Self::server_error) too.
    ///
    /// [`Handler`]: <https://docs.rs/musli-web/latest/musli_web/ws/trait.Handler.html>
    #[inline]
    pub fn as_server_error(&self) -> Option<&str> {
        match &self.kind {
            ErrorKind::Server(error) => Some(&error.message),
            _ => None,
        }
    }

    /// Decode the typed error the server answered with.
    ///
    /// A handler which fails with a [`ws::TypedError<E>`] sends `E` encoded in
    /// the body format of the connection, see [Errors]. This decodes it back
    /// as `T`, which should be the same type as `E` or one which is
    /// compatible with it, typically from an API crate shared with the server.
    ///
    /// Returns `Ok(None)` if this is not a server error, or if it is one which
    /// carries no typed error, such as a handler error which is plain text or
    /// an error the server produced on its own for a request it does not
    /// support. Those still have a message, see [`as_server_error`].
    ///
    /// # Errors
    ///
    /// Errors if the typed error does not decode as `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli_web::client::Error;
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// enum ServerError {
    ///     NotFound,
    /// }
    ///
    /// let error = Error::message("Something else");
    /// assert_eq!(error.server_error::<ServerError>().unwrap(), None);
    /// ```
    ///
    /// [`ws::TypedError<E>`]: <https://docs.rs/musli-web/latest/musli_web/ws/struct.TypedError.html>
    /// [Errors]: <https://docs.rs/musli-web/latest/musli_web/ws/index.html#errors>
    /// [`as_server_error`]: Self::as_server_error
    pub fn server_error<'de, T>(&'de self) -> Result<Option<T>>
    where
        T: DecodeBody<'de>,
    {
        let ErrorKind::Server(error) = &self.kind else {
            return Ok(None);
        };

        let Some((format, payload)) = &error.payload else {
            return Ok(None);
        };

        let value = format
            .decode(payload, &mut 0)
            .map_err(Error::decode_error_message)?;

        Ok(Some(value))
    }

    #[inline]
    fn incompatible(server: u32, client: u32) -> Self {
        Self::new(ErrorKind::Incompatible { server, client })
    }

    /// Format a client error consisting of a message.
    ///
    /// None of the accessors on [`Error`] match an error constructed this way.
    #[inline]
    pub fn message(message: impl fmt::Display) -> Self {
        Self::new(ErrorKind::Message(message.to_string()))
    }

    #[inline]
    fn server(error: ServerError) -> Self {
        Self::new(ErrorKind::Server(error))
    }

    #[inline]
    fn transport<E>(error: E) -> Self
    where
        E: 'static + Send + Sync + core::error::Error,
    {
        Self::new(ErrorKind::Transport(Box::new(error)))
    }

    #[inline]
    fn decode_response_header(error: format::Error) -> Self {
        Self::new(ErrorKind::DecodeResponseHeader(error))
    }

    #[inline]
    fn decode_error_message(error: format::Error) -> Self {
        Self::new(ErrorKind::DecodeErrorMessage(error))
    }

    #[inline]
    fn decode_packet(error: format::Error) -> Self {
        Self::new(ErrorKind::DecodePacket(error))
    }

    #[inline]
    fn encoding_header(error: format::Error) -> Self {
        Self::new(ErrorKind::EncodingHeader(error))
    }

    #[inline]
    fn encoding_body(error: format::Error) -> Self {
        Self::new(ErrorKind::EncodingBody(error))
    }
}

#[derive(Debug)]
enum ErrorKind {
    EmptyPacket,
    /// The request was never sent. Carries the text to display.
    NotConnected(&'static str),
    /// The connection closed while the request was pending. Carries the text
    /// to display.
    ConnectionLost(&'static str),
    Cancelled,
    InvalidHeader(InvalidHeader),
    UnknownFormat(u8),
    Incompatible {
        server: u32,
        client: u32,
    },
    Message(String),
    Server(ServerError),
    Transport(Box<dyn core::error::Error + Send + Sync>),
    DecodeResponseHeader(format::Error),
    DecodeErrorMessage(format::Error),
    DecodePacket(format::Error),
    EncodingHeader(format::Error),
    EncodingBody(format::Error),
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::EmptyPacket => write!(f, "Packet is empty"),
            ErrorKind::NotConnected(message) => write!(f, "{message}"),
            ErrorKind::ConnectionLost(message) => write!(f, "{message}"),
            ErrorKind::Cancelled => write!(f, "Request cancelled"),
            ErrorKind::InvalidHeader(header) => {
                write!(f, "Invalid handshake header `{}`", header.name)
            }
            ErrorKind::UnknownFormat(format) => write!(
                f,
                "Server used unknown format id {format} for a message body"
            ),
            ErrorKind::Incompatible { server, client } => write!(
                f,
                "Server states application version {server}, this client states {client}"
            ),
            ErrorKind::Message(message) => write!(f, "{message}"),
            ErrorKind::Server(error) => write!(f, "Server error: {}", error.message),
            ErrorKind::Transport(..) => write!(f, "Error in underlying transport"),
            ErrorKind::DecodeResponseHeader(..) => {
                write!(f, "Encoding error when decoding response header")
            }
            ErrorKind::DecodeErrorMessage(..) => {
                write!(f, "Encoding error when decoding error response")
            }
            ErrorKind::DecodePacket(..) => write!(f, "Encoding error when decoding packet"),
            ErrorKind::EncodingHeader(..) => write!(f, "Encoding error when encoding header"),
            ErrorKind::EncodingBody(..) => write!(f, "Encoding error when encoding body"),
        }
    }
}

impl core::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Transport(error) => Some(&**error),
            ErrorKind::InvalidHeader(header) => Some(&*header.error),
            ErrorKind::DecodeResponseHeader(error) => Some(error),
            ErrorKind::DecodeErrorMessage(error) => Some(error),
            ErrorKind::DecodePacket(error) => Some(error),
            ErrorKind::EncodingHeader(error) => Some(error),
            ErrorKind::EncodingBody(error) => Some(error),
            _ => None,
        }
    }
}

/// A header passed to [`ServiceBuilder::header`] which cannot be sent.
#[derive(Debug, Clone)]
struct InvalidHeader {
    /// The name of the header, as it was given.
    name: String,
    /// Why it cannot be sent.
    error: Arc<dyn core::error::Error + Send + Sync>,
}

/// An error response from the server, see [error
/// responses](crate::api#error-responses).
#[derive(Debug, Clone)]
struct ServerError {
    /// The human readable message.
    message: String,
    /// The typed payload, and the format it is encoded with.
    payload: Option<(Format, Bytes)>,
}

impl ServerError {
    /// A server error without a payload.
    #[inline]
    fn message(message: &str) -> Self {
        Self {
            message: String::from(message),
            payload: None,
        }
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

/// Slab of broadcast listeners.
type Broadcasts = HashMap<MessageId, Slab<mpsc::UnboundedSender<Result<RawPacket>>>>;

/// A command sent from a [`Handle`] to the [`Service`] driving the connection.
enum Command {
    /// Send an already encoded message and register a pending response.
    Send {
        serial: u32,
        mode: Mode,
        data: Vec<u8>,
        pending: Pending,
    },
    /// Cleanly disconnect a channel.
    Disconnect { channel: ChannelId },
    /// Close the service, causing [`Service::run`] to return.
    Close,
}

/// A pending response being waited for.
enum Pending {
    /// A format negotiation issued by the service itself.
    Negotiate { format: Format, mode: Mode },
    /// A regular request, tagged with the endpoint it belongs to.
    Request {
        id: MessageId,
        reply: oneshot::Sender<Result<RawPacket>>,
    },
    /// A channel being opened.
    Channel {
        reply: oneshot::Sender<Result<ChannelId>>,
    },
}

impl Pending {
    #[inline]
    fn error(self, error: Error) {
        match self {
            Pending::Negotiate { .. } => {
                tracing::debug!("Format negotiation failed: {error}");
            }
            Pending::Request { reply, .. } => {
                _ = reply.send(Err(error));
            }
            Pending::Channel { reply } => {
                _ = reply.send(Err(error));
            }
        }
    }
}

/// State shared between a [`Service`] and every [`Handle`] associated with it.
struct Shared {
    tx: mpsc::UnboundedSender<Command>,
    serial: AtomicU32,
    state: watch::Sender<State>,
    broadcasts: Mutex<Broadcasts>,
    /// Set once the service driving this state is gone, at which point no
    /// further state changes can be observed.
    gone: AtomicBool,
    /// The format which is actually in effect, as agreed by the [negotiation
    /// protocol]. This can differ from the requested format if the server did
    /// not support it.
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    format: AtomicU8,
    /// The mode which is actually in effect. This can differ from the requested
    /// mode if the server could not carry the format that way.
    mode: AtomicU8,
}

impl Shared {
    #[inline]
    fn next_serial(&self) -> u32 {
        self.serial.fetch_add(1, Ordering::Relaxed)
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.state.borrow().is_open()
    }

    /// The format currently in effect.
    #[inline]
    fn format(&self) -> Format {
        Format::from_u8(self.format.load(Ordering::Acquire)).unwrap_or(Format::DEFAULT)
    }

    #[inline]
    fn set_format(&self, format: Format) {
        self.format.store(format.to_u8(), Ordering::Release);
    }

    /// The mode currently in effect.
    #[inline]
    fn mode(&self) -> Mode {
        Mode::from_index(self.mode.load(Ordering::Acquire)).unwrap_or(Mode::DEFAULT)
    }

    #[inline]
    fn set_mode(&self, mode: Mode) {
        self.mode.store(mode.to_index(), Ordering::Release);
    }

    /// Test if the service driving this state is gone.
    #[inline]
    fn is_gone(&self) -> bool {
        self.gone.load(Ordering::Acquire)
    }

    /// Mark the service as gone and wake up anyone waiting for a state change.
    fn set_gone(&self) {
        self.gone.store(true, Ordering::Release);
        // NB: Unlike `send_if_modified` this always notifies, which is needed
        // to wake up waiters even if the state itself did not change.
        self.state.send_modify(|state| {
            // NB: Kept, since it is what explains why the service is gone.
            if !state.is_incompatible() {
                *state = State::Closed;
            }
        });
    }

    #[inline]
    fn send(&self, command: Command) -> Result<()> {
        if self.tx.send(command).is_err() {
            return Err(Error::new(ErrorKind::NotConnected(
                "Client service is down",
            )));
        }

        Ok(())
    }
}

/// Builder of a [`Service`].
///
/// Constructed through [`connect()`].
pub struct ServiceBuilder<T, E>
where
    T: ClientImpl,
{
    url: String,
    connector: Option<T::Connector>,
    headers: T::Headers,
    /// The first header which could not be added, see
    /// [`ServiceBuilder::header`].
    invalid_header: Option<InvalidHeader>,
    on_error: E,
    reconnect: bool,
    seed: u64,
    format: Format,
    mode: Mode,
    app: u32,
    _marker: PhantomData<T>,
}

impl<T, E> ServiceBuilder<T, E>
where
    T: ClientImpl,
    E: Callback<Error>,
{
    /// Set the error handler to use for the service.
    ///
    /// Errors which are reported here are errors which cannot be associated
    /// with a particular request, such as a failure to connect or a message
    /// which could not be decoded.
    #[inline]
    pub fn on_error<U>(self, on_error: U) -> ServiceBuilder<T, U>
    where
        U: Callback<Error>,
    {
        ServiceBuilder {
            url: self.url,
            connector: self.connector,
            headers: self.headers,
            invalid_header: self.invalid_header,
            on_error,
            reconnect: self.reconnect,
            seed: self.seed,
            format: self.format,
            mode: self.mode,
            app: self.app,
            _marker: self._marker,
        }
    }

    /// Add a header to the HTTP request which opens the websocket, the
    /// handshake.
    ///
    /// This is how a client authenticates with a server which expects a
    /// session cookie or a token, such as `Cookie` or `Authorization`. The
    /// header is sent on every connection attempt, so it is sent again when the
    /// service [reconnects], and it is sent whether the service connects to the
    /// url or [over a stream of its own].
    ///
    /// Calling this again with the same name adds another value rather than
    /// replacing the first one. Header names are case insensitive.
    ///
    /// # Errors
    ///
    /// The headers the handshake itself is made of, `Host`, `Connection`,
    /// `Upgrade`, `Sec-WebSocket-Key` and `Sec-WebSocket-Version`, are set by
    /// the client and cannot be added. Neither can a name or a value which is
    /// not valid in HTTP, such as a value with a line break in it.
    ///
    /// Since every attempt would send the same header, such a header is not
    /// retried. Instead [`Service::run`] and [`Service::try_connect`] return an
    /// error for which [`Error::is_invalid_header`] holds, without trying to
    /// connect.
    ///
    /// [reconnects]: Self::reconnect
    /// [over a stream of its own]: crate::tungstenite030::connect_with
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use musli_web::tungstenite030;
    ///
    /// # async fn example(token: &str) -> anyhow::Result<()> {
    /// let mut service = tungstenite030::connect("ws://127.0.0.1:8080/ws")
    ///     .header("Cookie", "session=4b0c2d")
    ///     .header("Authorization", format!("Bearer {token}"))
    ///     .build();
    ///
    /// service.run().await?;
    /// # Ok(()) }
    /// ```
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let name = name.as_ref();

        if let Err(error) = T::header(&mut self.headers, name, value.as_ref()) {
            // NB: The first one is reported, since it is the first one which
            // has to be fixed.
            if self.invalid_header.is_none() {
                self.invalid_header = Some(InvalidHeader {
                    name: name.to_string(),
                    error: Arc::new(error),
                });
            }
        }

        self
    }

    /// Set the [`Format`] to use for message bodies.
    ///
    /// The format is negotiated with the server once the connection is
    /// established, see the [negotiation protocol]. If the server does not
    /// support it the error is reported through [`ServiceBuilder::on_error`]
    /// and the connection falls back to [`Format::DEFAULT`], which can be
    /// observed through [`Handle::format`].
    ///
    /// Defaults to [`Format::DEFAULT`].
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    #[inline]
    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Set the [`Mode`] the socket should run in.
    ///
    /// [`Mode::Text`] makes every frame human readable, which is what makes it
    /// possible to read the traffic in a proxy or an inspector. It requires a
    /// [`Format`] which is human readable too, so it is normally combined with
    /// [`Format::Json`]:
    ///
    /// ```
    /// use musli_web::api::{Format, Mode};
    ///
    /// assert!(Mode::Text.accepts(Format::Json));
    /// ```
    ///
    /// Like the format, the mode is settled with the server once the connection
    /// is established, see the [negotiation protocol]. A server which cannot
    /// carry the requested format in the requested mode reports the error
    /// through [`ServiceBuilder::on_error`] and the connection falls back to
    /// [`Mode::DEFAULT`], which can be observed through [`Handle::mode`].
    ///
    /// Defaults to [`Mode::DEFAULT`].
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    #[inline]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// State the version of the application's API this client was built
    /// with.
    ///
    /// This is a version of *your* protocol, the requests and broadcasts
    /// defined with [`api::define!`], as opposed to [`api::VERSION`] which is
    /// the version of the envelope this crate puts around them. Give it the
    /// same constant the server passes to `ws::Connect::app_version`, and bump
    /// it whenever the API changes in a way an older client cannot cope with.
    ///
    /// When the connection is established the server states its version, and
    /// if it is not this one the service reports [`State::Incompatible`]
    /// instead of [`State::Open`], closes the connection, and [`Service::run`]
    /// returns an error for which [`Error::is_incompatible`] holds. It does so
    /// whether or not [reconnecting] is enabled, since connecting again would
    /// reach the same server.
    ///
    /// Zero, which is the default, states no version. The check only applies
    /// when both peers state one, so a client or a server which does not opt
    /// in behaves exactly as if this did not exist. See [application
    /// versions].
    ///
    /// [`api::define!`]: crate::api::define
    /// [reconnecting]: Self::reconnect
    /// [application versions]: crate::api#application-versions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use musli_web::tungstenite030;
    ///
    /// /// Shared with the server, bumped whenever the API changes.
    /// const APP_VERSION: u32 = 7;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut service = tungstenite030::connect("ws://127.0.0.1:8080/ws")
    ///     .app_version(APP_VERSION)
    ///     .build();
    ///
    /// if let Err(error) = service.run().await {
    ///     if error.is_incompatible() {
    ///         println!("{}, please upgrade", error);
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    #[inline]
    pub fn app_version(mut self, version: u32) -> Self {
        self.app = version;
        self
    }

    /// Configure whether the service should try to reconnect when the
    /// connection is lost.
    ///
    /// This defaults to `true`. If this is disabled, [`Service::run`] returns
    /// once the connection has been lost or could not be established.
    #[inline]
    pub fn reconnect(mut self, reconnect: bool) -> Self {
        self.reconnect = reconnect;
        self
    }

    /// Associate the specified seed with the service.
    ///
    /// This affects the random fuzzing which is applied to reconnect timeouts.
    ///
    /// By default the seed is a constant value.
    #[inline]
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Build the service.
    ///
    /// Note that no connection is established until [`Service::run`] or
    /// [`Service::try_connect`] is called.
    pub fn build(self) -> Service<T> {
        let (tx, rx) = mpsc::unbounded_channel();
        let (state, _) = watch::channel(State::Closed);

        let shared = Arc::new(Shared {
            tx,
            serial: AtomicU32::new(0),
            state,
            broadcasts: Mutex::new(Broadcasts::new()),
            gone: AtomicBool::new(false),
            format: AtomicU8::new(self.format.to_u8()),
            mode: AtomicU8::new(self.mode.to_index()),
        });

        Service {
            handle: Handle {
                shared: shared.clone(),
            },
            shared,
            rx,
            url: self.url,
            connector: self.connector,
            headers: self.headers,
            invalid_header: self.invalid_header,
            on_error: Box::new(self.on_error),
            reconnect: self.reconnect,
            socket: None,
            pending: HashMap::new(),
            timeout: INITIAL_TIMEOUT,
            next_attempt: Some(Instant::now()),
            rng: SmallRng::seed_from_u64(self.seed),
            closed: false,
            incompatible: None,
            requested: self.format,
            requested_mode: self.mode,
            app: self.app,
        }
    }
}

/// The service which drives a connection.
///
/// This is constructed through [`connect()`] and has to be driven by calling
/// [`Service::run`].
pub struct Service<T>
where
    T: ClientImpl,
{
    handle: Handle,
    shared: Arc<Shared>,
    rx: mpsc::UnboundedReceiver<Command>,
    url: String,
    connector: Option<T::Connector>,
    /// The headers added to every handshake.
    headers: T::Headers,
    /// A header which cannot be sent, which fails the service before it
    /// connects.
    invalid_header: Option<InvalidHeader>,
    on_error: Box<dyn Callback<Error>>,
    reconnect: bool,
    socket: Option<T::Socket>,
    pending: HashMap<u32, Pending>,
    timeout: Duration,
    next_attempt: Option<Instant>,
    rng: SmallRng,
    closed: bool,
    /// The application versions of the server and of this client, once the
    /// server has been found to state another one than this client.
    incompatible: Option<(u32, u32)>,
    /// The format the user asked for, which is re-negotiated on every
    /// reconnect.
    requested: Format,
    /// The mode which was asked for, which the connection falls back from if
    /// the server cannot carry the negotiated format that way.
    requested_mode: Mode,
    /// The application version this client states, or zero for none.
    app: u32,
}

/// The event produced by one iteration of the [`Service::run`] loop.
enum Output<E> {
    /// A message was received over the socket.
    Message(Option<Result<Message, E>>),
    /// A command was received from a handle.
    Command(Option<Command>),
    /// It is time to try and establish a connection.
    Connect,
}

impl<T> Service<T>
where
    T: ClientImpl,
{
    /// Get a handle to the service.
    ///
    /// The returned handle can be cloned and moved freely between tasks.
    #[inline]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Run the service.
    ///
    /// This drives the underlying connection and must be called for any
    /// requests to be processed. It is typically spawned onto a task of its
    /// own.
    ///
    /// Unless disabled through [`ServiceBuilder::reconnect`], a connection
    /// which is lost is re-established with an exponential backoff. Any
    /// requests which were in flight at that point are failed.
    ///
    /// This returns once [`Handle::close`] has been called, or the connection
    /// has been lost while reconnecting is disabled.
    ///
    /// # Errors
    ///
    /// Errors if the server states another application version, see
    /// [`ServiceBuilder::app_version`]. The service has then stopped for good,
    /// and calling this again returns immediately.
    ///
    /// Errors right away, without connecting, if a header was passed to
    /// [`ServiceBuilder::header`] which cannot be sent.
    pub async fn run(&mut self) -> Result<()> {
        if let Some(header) = &self.invalid_header {
            return Err(Error::new(ErrorKind::InvalidHeader(header.clone())));
        }

        if let Some((server, client)) = self.incompatible {
            return Err(Error::incompatible(server, client));
        }

        while !self.closed {
            let output = {
                let rx = &mut self.rx;

                match &mut self.socket {
                    Some(socket) => {
                        tokio::select! {
                            message = socket.recv() => Output::Message(message),
                            command = rx.recv() => Output::Command(command),
                        }
                    }
                    None => match self.next_attempt {
                        Some(deadline) => {
                            tokio::select! {
                                _ = tokio::time::sleep_until(deadline) => Output::Connect,
                                command = rx.recv() => Output::Command(command),
                            }
                        }
                        None => Output::Command(rx.recv().await),
                    },
                }
            };

            match output {
                Output::Connect => {
                    self.connect().await;
                }
                Output::Command(command) => {
                    let Some(command) = command else {
                        // Every handle has been dropped and no more commands
                        // can be received.
                        break;
                    };

                    self.command(command).await;
                }
                Output::Message(message) => {
                    let Some(message) = message else {
                        tracing::debug!("Connection closed by server");
                        self.disconnect().await;
                        continue;
                    };

                    let message = match message {
                        Ok(message) => message,
                        Err(error) => {
                            self.on_error.call(Error::transport(error));
                            self.disconnect().await;
                            continue;
                        }
                    };

                    match message {
                        // NB: The frame type is the mode, so a message can
                        // always be read regardless of what has been agreed on.
                        Message::Binary(bytes) => {
                            let post = self.message(Mode::Binary, bytes);
                            self.post(post).await;
                        }
                        Message::Text(bytes) => {
                            let post = self.message(Mode::Text, bytes);
                            self.post(post).await;
                        }
                        Message::Unsupported => {
                            self.on_error.call(Error::message("Unsupported message"));
                            self.disconnect().await;
                        }
                        Message::Ping | Message::Pong => {}
                        Message::Close => {
                            tracing::debug!("Close message received");
                            self.disconnect().await;
                        }
                    }
                }
            }
        }

        self.shutdown().await;

        match self.incompatible {
            Some((server, client)) => Err(Error::incompatible(server, client)),
            None => Ok(()),
        }
    }

    /// Carry out what processing a message said has to happen next.
    async fn post(&mut self, post: Result<Post>) {
        match post {
            Ok(Post::Negotiate) => self.send_negotiate().await,
            Ok(Post::None) => {}
            Ok(Post::Reject) => self.disconnect().await,
            Ok(Post::Incompatible { server }) => {
                // NB: Negotiated anyway, so that the server hears which
                // version this client states and can tell why it goes away.
                self.send_negotiate().await;
                self.incompatible(server).await;
            }
            Err(error) => self.on_error.call(error),
        }
    }

    /// Give up on a server which states application version `server`.
    async fn incompatible(&mut self, server: u32) {
        let client = self.app;
        tracing::debug!(server, client, "Incompatible server");

        if let Some(mut socket) = self.socket.take() {
            _ = socket.close().await;
        }

        self.close_pending(|| Error::incompatible(server, client));
        self.emit_state(State::Incompatible { server, client });
        self.next_attempt = None;
        self.closed = true;
        self.incompatible = Some((server, client));
    }

    /// Establish the connection now, returning a failure to the caller instead
    /// of scheduling a reconnect.
    ///
    /// This lets a caller fail fast when there is nothing to connect to, such
    /// as a refused connection or a missing unix socket, rather than leaving
    /// [`Service::run`] to retry with a backoff. On success the established
    /// connection is the one [`Service::run`] goes on to drive. If a connection
    /// is already established this does nothing.
    ///
    /// The error is not reported through [`ServiceBuilder::on_error`]. The
    /// underlying transport error is available through
    /// [`core::error::Error::source`] on the returned error.
    ///
    /// Errors without connecting if a header was passed to
    /// [`ServiceBuilder::header`] which cannot be sent, see
    /// [`Error::is_invalid_header`].
    pub async fn try_connect(&mut self) -> Result<()> {
        if let Some(header) = &self.invalid_header {
            return Err(Error::new(ErrorKind::InvalidHeader(header.clone())));
        }

        if self.socket.is_some() {
            return Ok(());
        }

        let socket = self.open().await.map_err(Error::transport)?;
        self.established(socket);
        Ok(())
    }

    /// Open a new socket, either through the caller-supplied connector or by
    /// connecting to the url.
    async fn open(&mut self) -> Result<T::Socket, T::Error> {
        tracing::debug!(url = self.url.as_str(), "Connecting");

        match &mut self.connector {
            Some(connector) => T::connect_with(connector, &self.headers).await,
            None => T::connect(&self.url, &self.headers).await,
        }
    }

    /// Install a newly established socket.
    fn established(&mut self, socket: T::Socket) {
        tracing::debug!("Connection established");
        self.socket = Some(socket);
        self.next_attempt = None;
        self.timeout = INITIAL_TIMEOUT;
    }

    /// Try to establish a connection.
    async fn connect(&mut self) {
        match self.open().await {
            Ok(socket) => {
                self.established(socket);
            }
            Err(error) => {
                self.on_error.call(Error::transport(error));
                self.schedule_reconnect();
            }
        }
    }

    /// Tear down the current connection and schedule a reconnect.
    async fn disconnect(&mut self) {
        if let Some(mut socket) = self.socket.take() {
            _ = socket.close().await;
        }

        self.emit_state(State::Closed);
        self.close_pending(|| Error::new(ErrorKind::ConnectionLost("Connection closed")));
        self.schedule_reconnect();
    }

    /// Shut the service down for good.
    async fn shutdown(&mut self) {
        if let Some(mut socket) = self.socket.take() {
            _ = socket.close().await;
        }

        // NB: Kept, since it is what explains why the service stopped.
        if !self.shared.state.borrow().is_incompatible() {
            self.emit_state(State::Closed);
        }

        self.close_pending(|| Error::new(ErrorKind::ConnectionLost("Client service closed")));
    }

    fn schedule_reconnect(&mut self) {
        if !self.reconnect {
            tracing::debug!("Reconnecting is disabled, closing service");
            self.closed = true;
            return;
        }

        let fuzz = self.rng.random_range(0..=MAX_FUZZ);

        let timeout = self
            .timeout
            .saturating_add(Duration::from_millis(fuzz))
            .min(MAX_TIMEOUT);

        self.timeout = self.timeout.saturating_mul(2).min(MAX_TIMEOUT);
        self.next_attempt = Some(Instant::now() + timeout);
        tracing::debug!(?timeout, "Scheduling reconnect");
    }

    /// Fail every pending request, since there is no chance they will be
    /// responded to any more.
    fn close_pending(&mut self, error: impl Fn() -> Error) {
        for (_, pending) in self.pending.drain() {
            pending.error(error());
        }
    }

    fn emit_state(&mut self, state: State) {
        self.shared.state.send_if_modified(|current| {
            if *current == state {
                return false;
            }

            *current = state;
            true
        });
    }

    /// Handle a command received from a handle.
    async fn command(&mut self, command: Command) {
        match command {
            Command::Send {
                serial,
                mode,
                data,
                pending,
            } => {
                let Some(socket) = self.socket.as_mut() else {
                    pending.error(Error::not_connected());
                    return;
                };

                if let Err(error) = socket.send(mode, &data).await {
                    pending.error(Error::transport(error));
                    self.disconnect().await;
                    return;
                }

                if let Some(existing) = self.pending.insert(serial, pending) {
                    existing.error(Error::new(ErrorKind::Cancelled));
                }
            }
            Command::Disconnect { channel } => {
                if let Err(error) = self.send_disconnect(channel).await {
                    self.on_error.call(error);
                }
            }
            Command::Close => {
                self.closed = true;
            }
        }
    }

    async fn send_disconnect(&mut self, channel: ChannelId) -> Result<()> {
        let Some(socket) = self.socket.as_mut() else {
            return Ok(());
        };

        let mut data = Vec::new();
        let mode = self.shared.mode();

        let header = api::RequestHeader {
            version: api::VERSION,
            serial: 0,
            id: MessageId::DISCONNECT.get(),
            // NB: Carries no body.
            format: 0,
            channel,
            app: 0,
        };

        format::encode_envelope(mode, &mut data, &header).map_err(Error::encoding_header)?;

        tracing::debug!(?channel, "Sending disconnect");

        if let Err(error) = socket.send(mode, &data).await {
            let error = Error::transport(error);
            self.disconnect().await;
            return Err(error);
        }

        Ok(())
    }

    /// Dispatch a value to every listener of the given broadcast.
    ///
    /// Note that listeners are never removed here, since removal is the
    /// exclusive responsibility of the corresponding [`Listener`]. Sending to a
    /// listener which has been dropped but not yet cleared is simply ignored.
    fn dispatch(&self, id: MessageId, value: impl Fn() -> Result<RawPacket>) {
        let broadcasts = self
            .shared
            .broadcasts
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let Some(slots) = broadcasts.get(&id) else {
            return;
        };

        for (_, tx) in slots.iter() {
            _ = tx.send(value());
        }
    }

    /// Resolve the format a message body is encoded with from its envelope.
    fn body_format(header: &api::ResponseHeader) -> Result<Format> {
        let Some(format) = Format::from_u8(header.format) else {
            return Err(Error::new(ErrorKind::UnknownFormat(header.format)));
        };

        Ok(format)
    }

    /// Decode the body of an error response, whose `error` header is `id`.
    ///
    /// An error id which carries no message is reported with `unsupported`.
    fn server_error(
        id: MessageId,
        header: &api::ResponseHeader,
        bytes: &Bytes,
        mut at: usize,
        unsupported: &str,
    ) -> Result<ServerError> {
        let typed = match id {
            MessageId::ERROR_MESSAGE => false,
            MessageId::TYPED_ERROR_MESSAGE => true,
            _ => return Ok(ServerError::message(unsupported)),
        };

        let format = Self::body_format(header)?;

        let error: api::ErrorMessage<'_> = format
            .decode(bytes, &mut at)
            .map_err(Error::decode_error_message)?;

        // NB: The payload is only decoded once the caller says what type it
        // is, see `Error::server_error`.
        let payload = if typed {
            Some((format, bytes.slice(at.min(bytes.len())..)))
        } else {
            None
        };

        Ok(ServerError {
            message: String::from(error.message),
            payload,
        })
    }

    /// Process an incoming message which arrived in the given mode.
    fn message(&mut self, mode: Mode, bytes: Bytes) -> Result<Post> {
        let mut at = 0;

        let header: api::ResponseHeader = match format::decode_envelope(mode, &bytes, &mut at) {
            Ok(header) => header,
            // NB: The server speaks a protocol this build does not have, so
            // there is nothing to be gained by carrying on with the session.
            // The connection is dropped rather than negotiated.
            Err(error) if error.unsupported_version().is_some() => {
                self.on_error.call(Error::decode_response_header(error));
                return Ok(Post::Reject);
            }
            Err(error) => return Err(Error::decode_response_header(error)),
        };

        if let Some(broadcast) = MessageId::new(header.broadcast) {
            tracing::debug!(?header, "Got broadcast");

            if broadcast == MessageId::SERVER_HELLO {
                if self.app != 0 && header.app != 0 && header.app != self.app {
                    return Ok(Post::Incompatible { server: header.app });
                }

                // NB: The connection is not reported as open until the format
                // has been negotiated, so that server-initiated messages are
                // never encoded with a format this client did not agree to.
                tracing::debug!("Server hello, negotiating format");
                return Ok(Post::Negotiate);
            }

            if let Some(id) = MessageId::new(header.error) {
                let error = Self::server_error(id, &header, &bytes, at, "Unsupported broadcast")?;
                self.dispatch(broadcast, || Err(Error::server(error.clone())));
                return Ok(Post::None);
            }

            let format = Self::body_format(&header)?;

            let packet = RawPacket {
                id: broadcast,
                buf: bytes,
                at: Cell::new(at),
                format,
                channel: header.channel,
            };

            self.dispatch(broadcast, || Ok(packet.clone()));
            return Ok(Post::None);
        }

        tracing::debug!(?header, "Got response");

        let Some(pending) = self.pending.remove(&header.serial) else {
            // NB: This is normal, it simply indicates that the request has been
            // cancelled.
            tracing::trace!(?header.serial, "Got message with unknown serial");
            return Ok(Post::None);
        };

        if let Some(id) = MessageId::new(header.error) {
            let error = match Self::server_error(id, &header, &bytes, at, "Unsupported request") {
                Ok(error) => error,
                Err(error) => match pending {
                    Pending::Negotiate { .. } => return Err(error),
                    // NB: Fail the request rather than leave it to find out
                    // that its reply was dropped.
                    pending => {
                        pending.error(error);
                        return Ok(Post::None);
                    }
                },
            };

            match pending {
                Pending::Negotiate {
                    format,
                    mode: requested,
                } => {
                    // NB: The server cannot speak the requested format, or
                    // cannot carry it in the requested mode, so fall back to the
                    // defaults rather than leaving the connection unusable. What
                    // is actually in effect is observable through
                    // `Handle::format` and `Handle::mode`.
                    self.on_error.call(Error::message(format_args!(
                        "Server rejected format `{format}` in `{requested}` mode ({}), falling back to `{}`",
                        error.message,
                        Format::DEFAULT
                    )));

                    self.shared.set_format(Format::DEFAULT);
                    // NB: The frame this arrived in is the mode the server
                    // settled on, so there is nothing to guess at.
                    self.shared.set_mode(mode);
                    self.emit_state(State::Open);
                }
                pending => {
                    pending.error(Error::server(error));
                }
            }

            return Ok(Post::None);
        }

        match pending {
            Pending::Negotiate { format, .. } => {
                // NB: Trust the format the server echoed back over the one that
                // was asked for, so that a server which downgrades is honored.
                // The mode is read off the frame for the same reason.
                let accepted = Format::from_u8(header.format).unwrap_or(format);
                tracing::debug!(?accepted, ?mode, "Format negotiated");
                self.shared.set_format(accepted);
                self.shared.set_mode(mode);
                self.emit_state(State::Open);
            }
            Pending::Channel { reply } => {
                _ = reply.send(Ok(header.channel));
            }
            Pending::Request { id, reply } => {
                let format = Self::body_format(&header)?;

                let packet = RawPacket {
                    id,
                    buf: bytes,
                    at: Cell::new(at),
                    format,
                    channel: header.channel,
                };

                _ = reply.send(Ok(packet));
            }
        }

        Ok(Post::None)
    }

    /// Ask the server to use the requested format for the rest of the
    /// connection.
    async fn send_negotiate(&mut self) {
        let format = self.requested;
        let mode = self.requested_mode;
        let serial = self.shared.next_serial();

        let header = api::RequestHeader {
            version: api::VERSION,
            serial,
            id: MessageId::NEGOTIATE.get(),
            format: format.to_u8(),
            // NB: Carries no body.
            channel: ChannelId::NONE,
            app: self.app,
        };

        let mut data = Vec::new();

        // NB: The frame this goes out in is what asks for the mode, so the
        // envelope has to be spelled the same way.
        if let Err(error) = format::encode_envelope(mode, &mut data, &header) {
            self.on_error.call(Error::encoding_header(error));
            return;
        }

        let Some(socket) = self.socket.as_mut() else {
            return;
        };

        tracing::debug!(?format, ?mode, "Requesting format");

        if let Err(error) = socket.send(mode, &data).await {
            self.on_error.call(Error::transport(error));
            self.disconnect().await;
            return;
        }

        self.pending
            .insert(serial, Pending::Negotiate { format, mode });
    }
}

/// Work which has to happen after a message has been processed, once the
/// borrow of the message buffer has been released.
enum Post {
    /// Nothing to do.
    None,
    /// The format has to be negotiated with the server.
    Negotiate,
    /// The server cannot be talked to, so the connection has to be dropped.
    Reject,
    /// The server states application version `server`, which is not the one
    /// this client states.
    Incompatible { server: u32 },
}

impl<T> Drop for Service<T>
where
    T: ClientImpl,
{
    fn drop(&mut self) {
        self.shared.set_gone();

        for (_, pending) in self.pending.drain() {
            pending.error(Error::new(ErrorKind::ConnectionLost(
                "Client service closed",
            )));
        }

        self.shared
            .broadcasts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

impl<T> fmt::Debug for Service<T>
where
    T: ClientImpl,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Service")
            .field("url", &self.url)
            .field("state", &*self.shared.state.borrow())
            .finish_non_exhaustive()
    }
}

/// A handle to the client service.
///
/// This is cheap to clone and can be moved freely between tasks.
#[derive(Clone)]
pub struct Handle {
    shared: Arc<Shared>,
}

impl Handle {
    /// Get the current state of the connection.
    #[inline]
    pub fn state(&self) -> State {
        *self.shared.state.borrow()
    }

    /// Check if the connection is currently open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.shared.is_open()
    }

    /// The [`Format`] currently in effect for message bodies.
    ///
    /// Before the connection has been opened this is the format which was
    /// requested through [`ServiceBuilder::format`]. Once the connection is
    /// open it is the format the server actually agreed to, which can differ if
    /// the server does not support what was asked for.
    #[inline]
    pub fn format(&self) -> Format {
        self.shared.format()
    }

    /// The [`Mode`] the socket is currently running in.
    ///
    /// Before the connection is open this is the mode which was requested
    /// through [`ServiceBuilder::mode`]. Once the connection is open it is the
    /// mode which was settled on, see the [negotiation protocol].
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    #[inline]
    pub fn mode(&self) -> Mode {
        self.shared.mode()
    }

    /// Listen for state changes to the underlying connection.
    ///
    /// This indicates when the connection is open and ready to receive requests
    /// through [`State::Open`], or if it's closed and requests will be rejected
    /// through [`State::Closed`]. A server which states another application
    /// version is reported through [`State::Incompatible`], see
    /// [`ServiceBuilder::app_version`].
    #[inline]
    pub fn on_state_change(&self) -> StateListener {
        StateListener {
            rx: self.shared.state.subscribe(),
            shared: self.shared.clone(),
        }
    }

    /// Wait until the connection is open.
    ///
    /// Since a connection is established asynchronously, this must be called
    /// before performing the first request unless you are prepared to handle
    /// the [`Error::is_not_connected`] error.
    ///
    /// This errors if the [`Service`] driving the connection is gone, since the
    /// connection can then never be established, and with an error for which
    /// [`Error::is_incompatible`] holds if the server states another
    /// application version, see [`ServiceBuilder::app_version`].
    pub async fn wait_until_open(&self) -> Result<()> {
        let mut rx = self.shared.state.subscribe();

        loop {
            match *rx.borrow_and_update() {
                State::Open => return Ok(()),
                State::Incompatible { server, client } => {
                    return Err(Error::incompatible(server, client));
                }
                State::Closed => {}
            }

            if self.shared.is_gone() || rx.changed().await.is_err() {
                return Err(Error::new(ErrorKind::NotConnected(
                    "Client service is down",
                )));
            }
        }
    }

    /// Open a new logical channel to the websocket server.
    ///
    /// A channel can be uniquely identified on the client and server side over
    /// a single connection. This means that if you send a request over a
    /// channel using [`Channel::request`], the server can access the
    /// [`ChannelId`] to determine which channel sent the request and the client
    /// has the ability to correlate any responses sent by the server by
    /// inspecting [`Packet::channel`] or [`RawPacket::channel`].
    ///
    /// The maximum number of channels is implementation defined, but expect it
    /// to be relatively low like `65535` (non-zero 16 bits) to reduce payload
    /// sizes. Failure to allocate a channel is an error.
    ///
    /// Note that a channel is scoped to the connection it was opened over. If
    /// the connection is lost and re-established, any channel opened over the
    /// old connection is stale and has to be opened again.
    pub async fn channel(&self) -> Result<Channel> {
        if !self.shared.is_open() {
            return Err(Error::not_connected());
        }

        let serial = self.shared.next_serial();

        let header = api::RequestHeader {
            version: api::VERSION,
            serial,
            id: MessageId::CONNECT.get(),
            // NB: Carries no body.
            format: 0,
            channel: ChannelId::NONE,
            app: 0,
        };

        let mode = self.shared.mode();
        let mut data = Vec::new();
        format::encode_envelope(mode, &mut data, &header).map_err(Error::encoding_header)?;

        let (reply, rx) = oneshot::channel();

        self.shared.send(Command::Send {
            serial,
            mode,
            data,
            pending: Pending::Channel { reply },
        })?;

        let Ok(result) = rx.await else {
            return Err(Error::new(ErrorKind::ConnectionLost(
                "Client service is down",
            )));
        };

        Ok(Channel {
            shared: self.shared.clone(),
            id: result?,
        })
    }

    /// Send a request over the default channel.
    ///
    /// See [`RequestBuilder::send`] for how the request is completed.
    #[inline]
    pub fn request(&self) -> RequestBuilder<'_, EmptyBody> {
        RequestBuilder {
            shared: &self.shared,
            channel: ChannelId::NONE,
            body: EmptyBody,
        }
    }

    /// Send a request over the default channel and decode its response.
    ///
    /// This sends `body`, waits for the response and decodes it as the
    /// response of the endpoint `body` is a request for. It is the same as
    /// [`request`] followed by [`RequestBuilder::send`] and
    /// [`Packet::decode_response`], except that the response is returned as
    /// an owned value which does not borrow from the packet it was decoded
    /// from.
    ///
    /// The response type of the endpoint must therefore be owned, which is
    /// what the `for<'de> ...: DecodeBody<'de>` bound says: it decodes from
    /// any buffer, so it holds on to none of them. A response declared as
    /// `type Response<'de> = User;` qualifies, one which borrows such as
    /// `type Response<'de> = UserRef<'de>;` does not compile here. For those
    /// keep the [`Packet`] around through [`RequestBuilder::send`] and decode
    /// from it instead.
    ///
    /// See [`Channel::call`] to call over a channel.
    ///
    /// [`request`]: Self::request
    ///
    /// # Errors
    ///
    /// The request fails with an [`Error`] which tells what went wrong:
    ///
    /// * [`Error::is_not_connected`] if the connection is not open, so the
    ///   request was never sent. See [`Handle::wait_until_open`].
    /// * [`Error::as_server_error`] if the handler on the server failed, and
    ///   [`Error::server_error`] to decode the typed error it failed with.
    /// * A decoding error if the response does not decode as the response
    ///   type, for instance since the server is built from another version of
    ///   the API.
    /// * Any other error if the connection is lost or the [`Service`] is
    ///   gone before the response arrives.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::{Error, Handle};
    ///
    /// mod api {
    ///     use musli::{Decode, Encode};
    ///     use musli_web::api;
    ///
    ///     #[derive(Encode, Decode)]
    ///     pub struct UserRequest<'de> {
    ///         pub name: &'de str,
    ///     }
    ///
    ///     #[derive(Debug, Encode, Decode)]
    ///     pub struct UserResponse {
    ///         pub name: String,
    ///         pub age: u32,
    ///     }
    ///
    ///     /// The typed error the server fails with.
    ///     #[derive(Debug, Encode, Decode)]
    ///     pub enum UserError {
    ///         NotFound,
    ///     }
    ///
    ///     api::define! {
    ///         pub type User;
    ///
    ///         impl Endpoint for User {
    ///             impl<'de> Request for UserRequest<'de>;
    ///             type Response<'de> = UserResponse;
    ///         }
    ///     }
    /// }
    ///
    /// /// Look up the age of a user, or `None` if there is no such user.
    /// async fn age(handle: &Handle, name: &str) -> Result<Option<u32>, Error> {
    ///     match handle.call(api::UserRequest { name }).await {
    ///         Ok(user) => Ok(Some(user.age)),
    ///         Err(error) => match error.server_error::<api::UserError>()? {
    ///             Some(api::UserError::NotFound) => Ok(None),
    ///             None => Err(error),
    ///         },
    ///     }
    /// }
    /// ```
    pub async fn call<B>(
        &self,
        body: B,
    ) -> Result<<B::Endpoint as api::Endpoint>::Response<'static>>
    where
        B: api::Request,
        for<'de> <B::Endpoint as api::Endpoint>::Response<'static>: DecodeBody<'de>,
    {
        self.request()
            .body(body)
            .send()
            .await?
            .decode_any_response()
    }

    /// Listen for broadcasts of type `T`.
    ///
    /// Broadcasts are buffered in the returned listener until they are received
    /// with [`Listener::recv`]. Dropping the listener removes it.
    ///
    /// Note that the buffer is unbounded, so a listener which is not drained
    /// keeps accumulating broadcasts. Drop or [`clear`] a listener which is no
    /// longer of interest.
    ///
    /// [`clear`]: Listener::clear
    pub fn on_broadcast<T>(&self) -> Listener<T>
    where
        T: api::Broadcast,
    {
        let (tx, rx) = mpsc::unbounded_channel();

        let index = {
            let mut broadcasts = self
                .shared
                .broadcasts
                .lock()
                .unwrap_or_else(|e| e.into_inner());

            broadcasts.entry(T::ID).or_default().insert(tx)
        };

        Listener {
            shared: Some(self.shared.clone()),
            id: T::ID,
            index,
            rx,
            _marker: PhantomData,
        }
    }

    /// Close the service.
    ///
    /// This causes [`Service::run`] to return.
    #[inline]
    pub fn close(&self) {
        _ = self.shared.tx.send(Command::Close);
    }
}

impl PartialEq for Handle {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl Eq for Handle {}

impl fmt::Debug for Handle {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("state", &*self.shared.state.borrow())
            .finish_non_exhaustive()
    }
}

/// A channel to a websocket server.
///
/// See [`Handle::channel`] for more details.
///
/// Dropping this cleanly disconnects the channel, which is signalled to the
/// server through [`Handler::close_channel`].
///
/// [`Handler::close_channel`]: <https://docs.rs/musli-web/latest/musli_web/ws/trait.Handler.html#method.close_channel>
pub struct Channel {
    shared: Arc<Shared>,
    id: ChannelId,
}

impl Channel {
    /// Get the channel identifier for this channel.
    #[inline]
    pub fn id(&self) -> ChannelId {
        self.id
    }

    /// Get a handle associated with this channel.
    ///
    /// A handle sheds the channel information and allows for setting up things
    /// like broadcast listeners.
    #[inline]
    pub fn handle(&self) -> Handle {
        Handle {
            shared: self.shared.clone(),
        }
    }

    /// Send a request over the current channel.
    ///
    /// See [`RequestBuilder::send`] for how the request is completed.
    #[inline]
    pub fn request(&self) -> RequestBuilder<'_, EmptyBody> {
        RequestBuilder {
            shared: &self.shared,
            channel: self.id,
            body: EmptyBody,
        }
    }

    /// Send a request over the current channel and decode its response.
    ///
    /// This is the same as [`Handle::call`] except that the request is sent
    /// over this channel, see it for how the response is decoded and the
    /// errors it fails with.
    pub async fn call<B>(
        &self,
        body: B,
    ) -> Result<<B::Endpoint as api::Endpoint>::Response<'static>>
    where
        B: api::Request,
        for<'de> <B::Endpoint as api::Endpoint>::Response<'static>: DecodeBody<'de>,
    {
        self.request()
            .body(body)
            .send()
            .await?
            .decode_any_response()
    }
}

impl Drop for Channel {
    #[inline]
    fn drop(&mut self) {
        _ = self.shared.send(Command::Disconnect { channel: self.id });
    }
}

impl fmt::Debug for Channel {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Channel")
            .field("id", &self.id)
            .field("state", &*self.shared.state.borrow())
            .finish_non_exhaustive()
    }
}

/// A request builder.
///
/// Set the body of the request with [`RequestBuilder::body`] and send it with
/// [`RequestBuilder::send`].
pub struct RequestBuilder<'a, B> {
    shared: &'a Arc<Shared>,
    channel: ChannelId,
    body: B,
}

impl<'a, B> RequestBuilder<'a, B> {
    /// Set the body of the request.
    #[inline]
    pub fn body<U>(self, body: U) -> RequestBuilder<'a, U>
    where
        U: api::Request,
    {
        RequestBuilder {
            shared: self.shared,
            channel: self.channel,
            body,
        }
    }
}

impl<B> RequestBuilder<'_, B>
where
    B: api::Request,
{
    /// Send the request and wait for the typed response.
    pub async fn send(self) -> Result<Packet<B::Endpoint>> {
        Ok(Packet::new(self.send_raw().await?))
    }

    /// Send the request and wait for the raw response.
    pub async fn send_raw(self) -> Result<RawPacket> {
        let id = <B::Endpoint as api::Endpoint>::ID;

        if !self.shared.is_open() {
            return Err(Error::not_connected());
        }

        let serial = self.shared.next_serial();
        let format = self.shared.format();
        let mode = self.shared.mode();

        let header = api::RequestHeader {
            version: api::VERSION,
            serial,
            id: id.get(),
            format: format.to_u8(),
            channel: self.channel,
            app: 0,
        };

        let mut data = Vec::new();
        format::encode_envelope(mode, &mut data, &header).map_err(Error::encoding_header)?;
        format
            .encode(&mut data, &self.body)
            .map_err(Error::encoding_body)?;

        tracing::debug!(
            serial,
            ?id,
            ?format,
            ?mode,
            len = data.len(),
            "Sending request"
        );

        let (reply, rx) = oneshot::channel();

        self.shared.send(Command::Send {
            serial,
            mode,
            data,
            pending: Pending::Request { id, reply },
        })?;

        let Ok(result) = rx.await else {
            return Err(Error::new(ErrorKind::ConnectionLost(
                "Client service is down",
            )));
        };

        result
    }
}

/// A listener for broadcasts of type `T`.
///
/// Constructed through [`Handle::on_broadcast`]. Dropping this removes the
/// listener.
pub struct Listener<T> {
    shared: Option<Arc<Shared>>,
    id: MessageId,
    index: usize,
    rx: mpsc::UnboundedReceiver<Result<RawPacket>>,
    _marker: PhantomData<T>,
}

impl<T> Listener<T> {
    /// Receive the next raw broadcast.
    ///
    /// Returns `None` if the service has been shut down.
    #[inline]
    pub async fn recv_raw(&mut self) -> Option<Result<RawPacket>> {
        self.rx.recv().await
    }

    /// Receive the next broadcast.
    ///
    /// Returns `None` if the service has been shut down.
    #[inline]
    pub async fn recv(&mut self) -> Option<Result<Packet<T>>> {
        Some(match self.rx.recv().await? {
            Ok(packet) => Ok(Packet::new(packet)),
            Err(error) => Err(error),
        })
    }

    /// Clear the listener without dropping it.
    ///
    /// This removes the associated listener from being notified. Any broadcasts
    /// which have already been buffered can still be received, after which
    /// [`Listener::recv`] returns `None`.
    pub fn clear(&mut self) {
        let Some(shared) = self.shared.take() else {
            return;
        };

        let index = mem::take(&mut self.index);

        let mut broadcasts = shared.broadcasts.lock().unwrap_or_else(|e| e.into_inner());

        let Entry::Occupied(mut e) = broadcasts.entry(self.id) else {
            return;
        };

        _ = e.get_mut().try_remove(index);

        if e.get().is_empty() {
            e.remove();
        }
    }
}

impl<T> Drop for Listener<T> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> fmt::Debug for Listener<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Listener")
            .field("type", &any::type_name::<T>())
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// A listener for state changes.
///
/// Constructed through [`Handle::on_state_change`].
pub struct StateListener {
    rx: watch::Receiver<State>,
    shared: Arc<Shared>,
}

impl StateListener {
    /// Get the most recently observed state.
    #[inline]
    pub fn state(&self) -> State {
        *self.rx.borrow()
    }

    /// Wait for the state to change and return the new state.
    ///
    /// Returns `None` if the [`Service`] driving the connection is gone.
    #[inline]
    pub async fn changed(&mut self) -> Option<State> {
        if self.shared.is_gone() {
            return None;
        }

        self.rx.changed().await.ok()?;

        if self.shared.is_gone() {
            return None;
        }

        Some(*self.rx.borrow_and_update())
    }

    /// Wait until the observed state is `state`.
    ///
    /// Returns `false` if the [`Service`] driving the connection is gone before
    /// the state could be observed.
    pub async fn wait_until(&mut self, state: State) -> bool {
        loop {
            if *self.rx.borrow_and_update() == state {
                return true;
            }

            if self.shared.is_gone() {
                return false;
            }

            if self.rx.changed().await.is_err() {
                return false;
            }
        }
    }
}

impl Clone for StateListener {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.clone(),
            shared: self.shared.clone(),
        }
    }
}

impl fmt::Debug for StateListener {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateListener")
            .field("state", &*self.rx.borrow())
            .finish()
    }
}

/// A raw packet of data.
#[derive(Clone)]
pub struct RawPacket {
    id: MessageId,
    buf: Bytes,
    at: Cell<usize>,
    format: Format,
    channel: ChannelId,
}

impl RawPacket {
    /// Construct an empty raw packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::MessageId;
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    ///
    /// assert!(packet.is_empty());
    /// assert_eq!(packet.id(), MessageId::EMPTY);
    /// ```
    #[inline]
    pub const fn empty() -> Self {
        Self {
            id: MessageId::EMPTY,
            buf: Bytes::new(),
            at: Cell::new(0),
            format: Format::DEFAULT,
            channel: ChannelId::NONE,
        }
    }

    /// The [`Format`] the body of this packet is encoded with.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// assert_eq!(packet.format(), Format::DEFAULT);
    /// ```
    #[inline]
    pub fn format(&self) -> Format {
        self.format
    }

    /// Return the channel this packet belongs to.
    ///
    /// For a response this is the channel the request was sent over, and for
    /// a broadcast it is the channel the broadcast originated from, as written
    /// by [`ws::Server::broadcast_from`]. A page can compare it to the id of its
    /// own channel to recognize the echo of a change it made itself, see
    /// [`RawPacket::is_from`].
    ///
    /// This is [`ChannelId::NONE`] for a request which was not sent over a
    /// channel, and for a broadcast which no channel is credited with, such as
    /// one written with [`ws::Server::broadcast`].
    ///
    /// [`ws::Server::broadcast`]: https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html#method.broadcast
    /// [`ws::Server::broadcast_from`]: https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html#method.broadcast_from
    #[inline]
    pub fn channel(&self) -> ChannelId {
        self.channel
    }

    /// Test if this packet originated from `channel`.
    ///
    /// This is `true` if the packet is a broadcast which was written with
    /// [`ws::Server::broadcast_from`] and `channel` as its origin, or a
    /// response to a request sent over `channel`. It is never `true` for
    /// [`ChannelId::NONE`], so a broadcast which no channel is credited with
    /// is not mistaken for the echo of a page which has not opened a channel
    /// yet.
    ///
    /// Channel ids are unique across every connection to a server by
    /// default, so a broadcast caused by another client never matches.
    ///
    /// [`ws::Server::broadcast_from`]: https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html#method.broadcast_from
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::ChannelId;
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// assert_eq!(packet.channel(), ChannelId::NONE);
    /// assert!(!packet.is_from(ChannelId::NONE));
    /// ```
    #[inline]
    pub fn is_from(&self, channel: ChannelId) -> bool {
        channel != ChannelId::NONE && self.channel == channel
    }

    /// Decode the contents of a raw packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`RawPacket::is_empty`].
    pub fn decode<'this, T>(&'this self) -> Result<T>
    where
        T: DecodeBody<'this>,
    {
        if self.id == MessageId::EMPTY {
            return Err(Error::new(ErrorKind::EmptyPacket));
        }

        let mut at = self.at.get();

        match self.format.decode(&self.buf, &mut at) {
            Ok(value) => {
                self.at.set(at);
                Ok(value)
            }
            Err(error) => {
                self.at.set(self.len());
                Err(Error::decode_packet(error))
            }
        }
    }

    /// Get the underlying byte slice of the packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// assert_eq!(packet.as_slice(), &[] as &[u8]);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// Get the number of bytes remaining to be decoded in the packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// assert_eq!(packet.remaining(), 0);
    /// ```
    #[inline]
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.at.get())
    }

    /// Get the length of the packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// assert_eq!(packet.len(), 0);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Check if the packet is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::RawPacket;
    ///
    /// let packet = RawPacket::empty();
    /// assert!(packet.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.at.get() >= self.len()
    }

    /// The id of the packet this is a response to as specified by
    /// [`Endpoint::ID`] or [`Broadcast::ID`].
    ///
    /// [`Endpoint::ID`]: crate::api::Endpoint::ID
    /// [`Broadcast::ID`]: crate::api::Broadcast::ID
    #[inline]
    pub fn id(&self) -> MessageId {
        self.id
    }
}

impl fmt::Debug for RawPacket {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawPacket")
            .field("id", &self.id)
            .field("remaining", &self.remaining())
            .finish()
    }
}

/// A typed packet of data.
pub struct Packet<T> {
    raw: RawPacket,
    _marker: PhantomData<T>,
}

impl<T> Packet<T> {
    /// Construct an empty packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::MessageId;
    /// use musli_web::client::Packet;
    ///
    /// let packet = Packet::<()>::empty();
    ///
    /// assert!(packet.is_empty());
    /// assert_eq!(packet.id(), MessageId::EMPTY);
    /// ```
    #[inline]
    pub const fn empty() -> Self {
        Self {
            raw: RawPacket::empty(),
            _marker: PhantomData,
        }
    }

    /// Construct a new typed packet from a raw one.
    ///
    /// Note that this does not guarantee that the typed packet is correct, but
    /// the `T` parameter becomes associated with it allowing it to be used
    /// automatically with methods such as [`Packet::decode`].
    #[inline]
    pub fn new(raw: RawPacket) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    /// Return the channel this packet belongs to.
    ///
    /// For a response this is the channel the request was sent over, and for
    /// a broadcast it is the channel the broadcast originated from, as written
    /// by [`ws::Server::broadcast_from`]. A page can compare it to the id of its
    /// own channel to recognize the echo of a change it made itself, see
    /// [`Packet::is_from`].
    ///
    /// This is [`ChannelId::NONE`] for a request which was not sent over a
    /// channel, and for a broadcast which no channel is credited with, such as
    /// one written with [`ws::Server::broadcast`].
    ///
    /// [`ws::Server::broadcast`]: https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html#method.broadcast
    /// [`ws::Server::broadcast_from`]: https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html#method.broadcast_from
    #[inline]
    pub fn channel(&self) -> ChannelId {
        self.raw.channel()
    }

    /// Test if this packet originated from `channel`.
    ///
    /// This is `true` if the packet is a broadcast which was written with
    /// [`ws::Server::broadcast_from`] and `channel` as its origin, or a
    /// response to a request sent over `channel`. It is never `true` for
    /// [`ChannelId::NONE`], so a broadcast which no channel is credited with
    /// is not mistaken for the echo of a page which has not opened a channel
    /// yet.
    ///
    /// Channel ids are unique across every connection to a server by
    /// default, so a broadcast caused by another client never matches.
    ///
    /// [`ws::Server::broadcast_from`]: https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html#method.broadcast_from
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::ChannelId;
    /// use musli_web::client::Packet;
    ///
    /// let packet = Packet::<()>::empty();
    /// assert_eq!(packet.channel(), ChannelId::NONE);
    /// assert!(!packet.is_from(ChannelId::NONE));
    /// ```
    #[inline]
    pub fn is_from(&self, channel: ChannelId) -> bool {
        self.raw.is_from(channel)
    }

    /// The [`Format`] the body of this packet is encoded with.
    #[inline]
    pub fn format(&self) -> Format {
        self.raw.format()
    }

    /// Convert a packet into a raw packet.
    #[inline]
    pub fn into_raw(self) -> RawPacket {
        self.raw
    }

    /// Get the number of bytes remaining to be decoded in the packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::Packet;
    ///
    /// let packet = Packet::<()>::empty();
    /// assert_eq!(packet.remaining(), 0);
    /// ```
    #[inline]
    pub fn remaining(&self) -> usize {
        self.raw.remaining()
    }

    /// Check if the packet is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::client::Packet;
    ///
    /// let packet = Packet::<()>::empty();
    /// assert!(packet.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// The id of the packet this is a response to as specified by
    /// [`Endpoint::ID`] or [`Broadcast::ID`].
    ///
    /// [`Endpoint::ID`]: crate::api::Endpoint::ID
    /// [`Broadcast::ID`]: crate::api::Broadcast::ID
    #[inline]
    pub fn id(&self) -> MessageId {
        self.raw.id()
    }
}

impl<T> Packet<T>
where
    T: api::Decodable,
{
    /// Decode the contents of a packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`Packet::is_empty`].
    #[inline]
    pub fn decode(&self) -> Result<T::Type<'_>> {
        self.decode_any()
    }

    /// Decode any contents of a packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`Packet::is_empty`].
    #[inline]
    pub fn decode_any<'de, R>(&'de self) -> Result<R>
    where
        R: DecodeBody<'de>,
    {
        self.raw.decode()
    }
}

impl<T> Packet<T>
where
    T: api::Endpoint,
{
    /// Decode the response of a packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`Packet::is_empty`].
    #[inline]
    pub fn decode_response(&self) -> Result<T::Response<'_>> {
        self.decode_any_response()
    }

    /// Decode any response of a packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`Packet::is_empty`].
    #[inline]
    pub fn decode_any_response<'de, R>(&'de self) -> Result<R>
    where
        R: DecodeBody<'de>,
    {
        self.raw.decode()
    }
}

impl<T> Packet<T>
where
    T: api::Broadcast,
{
    /// Decode the primary event related to a broadcast.
    #[inline]
    pub fn decode_event<'de>(&'de self) -> Result<T::Event<'de>>
    where
        T: api::BroadcastWithEvent,
    {
        self.decode_event_any()
    }

    /// Decode any event related to a broadcast.
    #[inline]
    pub fn decode_event_any<'de, E>(&'de self) -> Result<E>
    where
        E: Event<Broadcast = T> + DecodeBody<'de>,
    {
        self.raw.decode()
    }
}

impl<T> Clone for Packet<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for Packet<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("type", &any::type_name::<T>())
            .field("remaining", &self.remaining())
            .finish()
    }
}

#[cfg(test)]
mod tests;
