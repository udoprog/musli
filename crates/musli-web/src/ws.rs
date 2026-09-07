//! The server side of the websocket protocol.
//!
//! See [`server()`] for how to use with [axum].
//!
//! A connection starts life as a [`Connect`], which cannot send anything.
//! [`Connect::connect`] performs the [negotiation protocol] and hands back the
//! [`Server`], so by the time there is anything able to write a message the
//! [`Format`] it will be encoded with has been agreed with the client. A client
//! which does not negotiate never gets a [`Server`] at all.
//!
//! Handlers are implemented via the [`Handler`] trait, which allows returning
//! various forms of responses dictated through the [`IntoResponse`] trait. This
//! is primarily implemented for `bool`, where returning `false` indicates that
//! the given request kind is not supported.
//!
//! You can also return custom error for a handler by having it return anything
//! that implements [`fmt::Display`]:
//!
//! ```
//! use musli_web::api::MessageId;
//! use musli_web::ws;
//!
//! mod api {
//!     use musli::{Decode, Encode};
//!     use musli_web::api;
//!
//!     #[derive(Encode, Decode)]
//!     pub struct HelloRequest<'de> {
//!         pub message: &'de str,
//!     }
//!
//!     #[derive(Encode, Decode)]
//!     pub struct HelloResponse<'de> {
//!         pub message: &'de str,
//!     }
//!
//!     #[derive(Encode, Decode)]
//!     pub struct TickEvent<'de> {
//!         pub message: &'de str,
//!         pub tick: u32,
//!     }
//!
//!     api::define! {
//!         pub type Hello;
//!
//!         impl Endpoint for Hello {
//!             impl<'de> Request for HelloRequest<'de>;
//!             type Response<'de> = HelloResponse<'de>;
//!         }
//!
//!         pub type Tick;
//!
//!         impl Broadcast for Tick {
//!             impl<'de> Event for TickEvent<'de>;
//!         }
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Broadcast {
//!     Tick { tick: u32 },
//! }
//!
//! #[derive(Clone)]
//! struct MyHandler;
//!
//! impl ws::Handler for MyHandler {
//!     type Id = api::Request;
//!     type Response = Option<()>;
//!
//!     async fn handle(
//!         &self,
//!         id: Self::Id,
//!         incoming: &mut ws::Incoming<'_>,
//!         outgoing: &mut ws::Outgoing<'_>,
//!     ) -> Self::Response {
//!         tracing::info!("Handling: {id:?}");
//!
//!         match id {
//!             api::Request::Hello => {
//!                 let request = incoming.read::<api::HelloRequest<'_>>()?;
//!
//!                 outgoing.write(api::HelloResponse {
//!                     message: request.message,
//!                 });
//!
//!                 Some(())
//!             }
//!             api::Request::Unknown(id) => {
//!                 tracing::info!("Unknown request id: {}", id.get());
//!                 None
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! [`server()`]: crate::axum08::server
//! [axum]: <https://docs.rs/axum>
//! [negotiation protocol]: crate::api#negotiating-the-format

use core::convert::Infallible;
use core::fmt::{self, Write};
use core::future::Future;
use core::num::NonZeroU16;
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use bytes::Bytes;
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::{Duration, Instant, Sleep};

use crate::Buf;
use crate::api::{
    Broadcast, ChannelId, DecodeBody, EncodeBody, ErrorMessage, Event, Format, Id, MessageId, Mode,
    RequestHeader, ResponseHeader,
};
use crate::buf::{BufPool, InvalidFrame};
use crate::format;

const MAX_CAPACITY: usize = 1048576;
const CLOSE_NORMAL: u16 = 1000;
const CLOSE_PROTOCOL_ERROR: u16 = 1002;
const CLOSE_TIMEOUT: Duration = Duration::from_secs(30);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_SEED: u64 = 0xdeadbeef;

/// A websocket message.
#[derive(Debug)]
pub(crate) enum Message {
    /// A text message.
    Text(Bytes),
    /// A binary message.
    Binary(Bytes),
    /// A ping message.
    Ping(Bytes),
    /// A pong message.
    Pong(Bytes),
    /// A close message.
    Close,
}

pub(crate) mod socket_sealed {
    pub trait Sealed {}
}

pub(crate) trait SocketImpl
where
    Self: self::socket_sealed::Sealed,
{
    #[doc(hidden)]
    type Message;

    #[doc(hidden)]
    type Error: fmt::Debug;

    #[doc(hidden)]
    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Result<Message, Self::Error>>>;

    #[doc(hidden)]
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    #[doc(hidden)]
    fn start_send(self: Pin<&mut Self>, item: Self::Message) -> Result<(), Self::Error>;

    #[doc(hidden)]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
}

pub(crate) mod server_sealed {
    pub trait Sealed {}
}

/// The details of how a [`Server`] is implemented.
///
/// See [`AxumServer`] for an example.
///
/// [`AxumServer`]: crate::axum08::AxumServer
pub trait ServerImpl
where
    Self: self::server_sealed::Sealed,
{
    #[doc(hidden)]
    type Error: fmt::Debug;

    #[doc(hidden)]
    type Message;

    #[doc(hidden)]
    #[allow(private_bounds)]
    type Socket: SocketImpl<Message = Self::Message, Error = Self::Error>;

    #[doc(hidden)]
    fn ping(data: Bytes) -> Self::Message;

    #[doc(hidden)]
    fn pong(data: Bytes) -> Self::Message;

    #[doc(hidden)]
    fn binary(data: &[u8]) -> Self::Message;

    #[doc(hidden)]
    fn text(data: &[u8]) -> Self::Message;

    #[doc(hidden)]
    fn close(code: u16, reason: &str) -> Self::Message;
}

#[derive(Debug)]
enum ErrorKind {
    #[cfg(feature = "axum-core05")]
    AxumCore05 {
        error: axum_core05::Error,
    },
    FormatError,
    InvalidFrame {
        error: InvalidFrame,
    },
    Incoming {
        error: format::Error,
    },
    Outgoing {
        error: format::Error,
    },
    EncodeBroadcastHeader {
        error: format::Error,
    },
    EncodeBroadcast {
        error: format::Error,
    },
    EncodeConnectHeader {
        error: format::Error,
    },
    ErrorMessageHeader {
        error: format::Error,
    },
    ErrorMessage {
        error: format::Error,
    },
    OutOfBounds {
        offset: usize,
        len: usize,
    },
    /// The connection went away before the format had been negotiated.
    NotNegotiated,
    /// The client sent a message other than a negotiation as its first message.
    ExpectedNegotiate {
        id: u16,
    },
    /// The client sent a malformed envelope during negotiation.
    NegotiateHeader {
        error: format::Error,
    },
}

/// The error produced by the server side of the websocket protocol
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[inline]
    const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    pub(crate) fn incoming(error: format::Error) -> Self {
        Self::new(ErrorKind::Incoming { error })
    }

    pub(crate) fn outgoing(error: format::Error) -> Self {
        Self::new(ErrorKind::Outgoing { error })
    }

    pub(crate) fn encode_broadcast_header(error: format::Error) -> Self {
        Self::new(ErrorKind::EncodeBroadcastHeader { error })
    }

    pub(crate) fn encode_broadcast(error: format::Error) -> Self {
        Self::new(ErrorKind::EncodeBroadcast { error })
    }

    pub(crate) fn encode_connect_header(error: format::Error) -> Self {
        Self::new(ErrorKind::EncodeConnectHeader { error })
    }

    pub(crate) fn encode_error_message_header(error: format::Error) -> Self {
        Self::new(ErrorKind::ErrorMessageHeader { error })
    }

    pub(crate) fn encode_error_message(error: format::Error) -> Self {
        Self::new(ErrorKind::ErrorMessage { error })
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            #[cfg(feature = "axum-core05")]
            ErrorKind::AxumCore05 { .. } => write!(f, "Error in axum-core"),
            ErrorKind::FormatError => write!(f, "Error formatting error response"),
            ErrorKind::InvalidFrame { error } => error.fmt(f),
            ErrorKind::Incoming { .. } => {
                write!(f, "Encoding error when decoding incoming message")
            }
            ErrorKind::Outgoing { .. } => {
                write!(f, "Encoding error when encoding outgoing message")
            }
            ErrorKind::EncodeBroadcastHeader { .. } => {
                write!(f, "Encoding error when encoding broadcast header")
            }
            ErrorKind::EncodeBroadcast { .. } => {
                write!(f, "Encoding error when broadcasting message")
            }
            ErrorKind::EncodeConnectHeader { .. } => {
                write!(f, "Encoding error when encoding connect header")
            }
            ErrorKind::ErrorMessageHeader { .. } => {
                write!(f, "Encoding error when encoding error message header")
            }
            ErrorKind::ErrorMessage { .. } => {
                write!(f, "Encoding error when encoding error message")
            }
            ErrorKind::OutOfBounds { offset, len } => {
                write!(
                    f,
                    "Error when reading message: offset {} is out of bounds for length {}",
                    offset, len
                )
            }
            ErrorKind::NotNegotiated => {
                write!(f, "Connection closed before the format was negotiated")
            }
            ErrorKind::ExpectedNegotiate { id } => {
                write!(
                    f,
                    "Expected a negotiation as the first message, but got message id {id}"
                )
            }
            ErrorKind::NegotiateHeader { .. } => {
                write!(f, "Encoding error when decoding negotiation header")
            }
        }
    }
}

impl core::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.kind {
            #[cfg(feature = "axum-core05")]
            ErrorKind::AxumCore05 { error } => Some(error),
            ErrorKind::Incoming { error } => Some(error),
            ErrorKind::Outgoing { error } => Some(error),
            ErrorKind::EncodeBroadcastHeader { error } => Some(error),
            ErrorKind::EncodeBroadcast { error } => Some(error),
            ErrorKind::EncodeConnectHeader { error } => Some(error),
            ErrorKind::ErrorMessageHeader { error } => Some(error),
            ErrorKind::ErrorMessage { error } => Some(error),
            ErrorKind::NegotiateHeader { error } => Some(error),
            _ => None,
        }
    }
}

#[cfg(feature = "axum-core05")]
impl From<axum_core05::Error> for Error {
    #[inline]
    fn from(error: axum_core05::Error) -> Self {
        Self::new(ErrorKind::AxumCore05 { error })
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl From<InvalidFrame> for Error {
    #[inline]
    fn from(error: InvalidFrame) -> Self {
        Self::new(ErrorKind::InvalidFrame { error })
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

/// The response meta from handling a request.
pub struct Response {
    handled: bool,
}

/// Trait governing how something can be converted into a response.
pub trait IntoResponse
where
    Self: 'static + Send,
{
    /// The error variant being produced.
    type Error: fmt::Display;

    /// Convert self into a response.
    fn into_response(self) -> Result<Response, Self::Error>;
}

/// Implement [`IntoResponse`] for unit types `()`.
///
/// This indicates that the request has been handled.
impl IntoResponse for () {
    type Error = Infallible;

    #[inline]
    fn into_response(self) -> Result<Response, Self::Error> {
        Ok(Response { handled: true })
    }
}

/// Implement [`IntoResponse`] for `bool`.
///
/// On `true`, this means that the request was supported `false` means that it
/// wasn't.
impl IntoResponse for bool {
    type Error = Infallible;

    #[inline]
    fn into_response(self) -> Result<Response, Self::Error> {
        Ok(Response { handled: self })
    }
}

/// Implement [`IntoResponse`] for [`Result`] types.
///
/// Note that this allows anything that implements [`fmt::Display`] to be used
/// as an [`Err`] variant. The exact message it's being formatted into will be
/// forwarded as an error to the client.
///
/// [`Result`]: core::result::Result
impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse<Error = Infallible>,
    E: 'static + Send + fmt::Display,
{
    type Error = E;

    #[inline]
    fn into_response(self) -> Result<Response, E> {
        match self {
            Ok(into_response) => match IntoResponse::into_response(into_response) {
                Ok(response) => Ok(response),
                Err(error) => match error {},
            },
            Err(error) => Err(error),
        }
    }
}

/// Implement [`IntoResponse`] for [`Option`] types.
///
/// This will propagate any responses for the interior value if present. If the
/// value is [`None`] this will be treated as unhandled. This can be useful when
/// used in combination with [`Incoming::read`] since it returns an [`Option`].
impl<T> IntoResponse for Option<T>
where
    T: IntoResponse,
{
    type Error = T::Error;

    #[inline]
    fn into_response(self) -> Result<Response, Self::Error> {
        match self {
            Some(value) => value.into_response(),
            None => Ok(Response { handled: false }),
        }
    }
}

/// A handler for incoming requests.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub trait Handler
where
    Self: 'static + Send + Clone,
{
    /// The type of message id used.
    type Id: Id;
    /// The response type returned by the handler.
    type Response: IntoResponse;

    /// Indicates that a `channel` has been opened.
    ///
    /// This indicates that you are communicating with a client that has opened
    /// a channel with [`Handle::channel`].
    ///
    /// After this has been called, you can expected to receive requests from
    /// the [`ChannelId`] corresponding to `channel`. The channel id of incoming
    /// requests can be inspected with [`Incoming::channel`].
    ///
    /// [`Handle::channel`]: crate::web::Handle::channel
    fn open_channel<'this>(
        &'this self,
        channel: ChannelId,
    ) -> impl Future<Output = ()> + Send + 'this {
        async {
            _ = channel;
        }
    }

    /// Indicates that a `channel` has been cleanly closed.
    ///
    /// This indicates that communicating with a client that has opened a
    /// channel with [`Handle::channel`] has been cleanly closed, which occurs
    /// when the channel is cleanly closed by dropping the last handle to it.
    ///
    /// [`Handle::channel`]: crate::web::Handle::channel
    fn close_channel<'this>(
        &'this self,
        channel: ChannelId,
    ) -> impl Future<Output = ()> + Send + 'this {
        async {
            _ = channel;
        }
    }

    /// Handle a request.
    fn handle<'this>(
        &'this self,
        id: Self::Id,
        incoming: &'this mut Incoming<'_>,
        outgoing: &'this mut Outgoing<'_>,
    ) -> impl Future<Output = Self::Response> + Send + 'this;
}

struct Pinned<S> {
    socket: S,
    close_sleep: Sleep,
    ping_sleep: Sleep,
}

impl<S> Pinned<S> {
    #[inline]
    fn project(self: Pin<&mut Self>) -> (Pin<&mut Sleep>, Pin<&mut Sleep>, Pin<&mut S>) {
        unsafe {
            let this = self.get_unchecked_mut();
            (
                Pin::new_unchecked(&mut this.close_sleep),
                Pin::new_unchecked(&mut this.ping_sleep),
                Pin::new_unchecked(&mut this.socket),
            )
        }
    }
}

type HandlerOutput<H> = (Result<<H as Handler>::Response, Error>, RequestHeader, Buf);

/// Trait which governs how channel identifiers are allocated with a [`Server`].
///
/// By default channel identifiers are scoped to the server which is set up
/// per-connection. If you want distinct and unique channel identifiers across
/// multiple websocket connections a custom [`ChannelAllocator`] can be
/// constructed.
pub trait ChannelAllocator {
    /// Allocate the next channel id.
    ///
    /// Using `0` is equivalent to [`ChannelId::NONE`] so the allocator must
    /// avoid constructor identifiers with this value since it is equivalent to
    /// no channel.
    ///
    /// [`ChannelAllocator`]: crate::ws::ChannelAllocator
    fn next(&self) -> impl Future<Output = Option<ChannelId>> + Send + '_;

    /// Free the given channel id.
    fn free(&self, channel: ChannelId) -> impl Future<Output = ()> + Send + '_;
}

/// A connection which has not yet completed the [negotiation protocol].
///
/// This is what [`server()`] hands back, and it is the only way to obtain a
/// [`Server`]. Configuration lives here rather than on [`Server`], since every
/// setting has to be in place before the first byte goes over the wire.
///
/// Crucially this type cannot send messages. A [`Server`] — which can — only
/// exists once [`Connect::connect`] has resolved, which is precisely the point
/// at which the [`Format`] for the connection has been settled. Attempting to
/// broadcast before that is a compile error rather than a message the client
/// cannot read.
///
/// [`server()`]: crate::axum08::server
/// [negotiation protocol]: crate::api#negotiating-the-format
///
/// # Examples
///
/// ```
/// # extern crate axum08 as axum;
/// # use axum::extract::ws::WebSocket;
/// use musli_web::api::Format;
/// use musli_web::{axum08, ws};
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
///     api::define! {
///         pub type Hello;
///
///         impl Endpoint for Hello {
///             impl<'de> Request for HelloRequest<'de>;
///             type Response<'de> = HelloResponse<'de>;
///         }
///     }
/// }
///
/// #[derive(Clone)]
/// struct MyHandler;
///
/// impl ws::Handler for MyHandler {
///     type Id = api::Request;
///     type Response = bool;
///
///     async fn handle(
///         &self,
///         id: Self::Id,
///         incoming: &mut ws::Incoming<'_>,
///         outgoing: &mut ws::Outgoing<'_>,
///     ) -> bool {
///         false
///     }
/// }
///
/// # async fn example(socket: WebSocket) -> Result<(), ws::Error> {
/// let mut server = axum08::server(socket, MyHandler)
///     .with_formats(&[Format::Wire, Format::Json])
///     .connect()
///     .await?;
///
/// // Only reachable once the client has negotiated a format.
/// server.run().await?;
/// # Ok(())
/// # }
/// ```
///
/// Skipping the connection step does not compile, since a [`Connect`] has
/// nothing to broadcast with:
///
/// ```compile_fail
/// # extern crate axum08 as axum;
/// # use axum::extract::ws::WebSocket;
/// use musli_web::{axum08, ws};
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
///     pub struct TickEvent {
///         pub tick: u32,
///     }
///
///     api::define! {
///         pub type Hello;
///
///         impl Endpoint for Hello {
///             impl<'de> Request for HelloRequest<'de>;
///             type Response<'de> = HelloResponse<'de>;
///         }
///
///         pub type Tick;
///
///         impl Broadcast for Tick {
///             impl Event for TickEvent;
///         }
///     }
/// }
///
/// #[derive(Clone)]
/// struct MyHandler;
///
/// impl ws::Handler for MyHandler {
///     type Id = api::Request;
///     type Response = bool;
///
///     async fn handle(
///         &self,
///         id: Self::Id,
///         incoming: &mut ws::Incoming<'_>,
///         outgoing: &mut ws::Outgoing<'_>,
///     ) -> bool {
///         false
///     }
/// }
///
/// # async fn example(socket: WebSocket) -> Result<(), ws::Error> {
/// let mut server = axum08::server(socket, MyHandler);
/// // `Connect` has no `broadcast`, only the `Server` that `connect()` hands
/// // back does.
/// server.broadcast(api::TickEvent { tick: 1 })?;
/// # Ok(())
/// # }
/// ```
pub struct Connect<S, H, C = Channels>
where
    S: ServerImpl,
    H: Handler,
{
    handler: H,
    socket: S::Socket,
    channels: C,
    seed: u64,
    max_capacity: usize,
    /// Formats this server is willing to negotiate, or `None` to accept every
    /// format it was built with support for.
    formats: Option<&'static [Format]>,
}

impl<S, H> Connect<S, H, Channels>
where
    S: ServerImpl,
    H: Handler,
{
    /// Construct a new pending connection with the specified handler.
    #[inline]
    pub(crate) fn new(socket: S::Socket, handler: H) -> Self {
        Self {
            handler,
            socket,
            channels: Channels::default(),
            seed: DEFAULT_SEED,
            max_capacity: MAX_CAPACITY,
            formats: None,
        }
    }
}

impl<S, H, C> Connect<S, H, C>
where
    S: ServerImpl,
    H: Handler,
{
    /// Associate the specified seed with the connection.
    ///
    /// This affects the random number generation used for ping messages.
    ///
    /// By default the seed is a constant value.
    #[inline]
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Associate the specified channel allocator with the connection.
    #[inline]
    pub fn with_channel_allocator<U>(self, channels: U) -> Connect<S, H, U>
    where
        U: ChannelAllocator,
    {
        Connect {
            handler: self.handler,
            socket: self.socket,
            channels,
            seed: self.seed,
            max_capacity: self.max_capacity,
            formats: self.formats,
        }
    }

    /// Get a reference to the handler.
    #[inline]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Restrict the set of [`Format`]s this server is willing to negotiate.
    ///
    /// By default every format the crate was built with support for is
    /// accepted. A client which asks for a format outside of this set is
    /// rejected during the [negotiation protocol] and the connection settles on
    /// [`Format::DEFAULT`] instead.
    ///
    /// Note that this cannot widen the set, a format which was not compiled in
    /// is never accepted.
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    #[inline]
    pub fn with_formats(mut self, formats: &'static [Format]) -> Self {
        self.formats = Some(formats);
        self
    }

    /// Test if this server is willing to negotiate `format`.
    #[inline]
    pub fn accepts(&self, format: Format) -> bool {
        accepts(self.formats, format)
    }

    /// Modify the max allocated capacity of the buffers used for outgoing
    /// messages.
    ///
    /// This is not a hard limit. A message larger than this is still written,
    /// but once it has been flushed the allocation is released back down to the
    /// specified value rather than being kept for the lifetime of the
    /// connection.
    ///
    /// By default, the capacity is 1 MiB.
    #[inline]
    pub fn max_capacity(mut self, max_capacity: usize) -> Self {
        self.max_capacity = max_capacity;
        self
    }

    /// Modify the max allocated capacity of the outgoing buffers.
    ///
    /// This is an alias for [`Connect::max_capacity`].
    #[inline]
    pub fn with_max_capacity(self, max_capacity: usize) -> Self {
        self.max_capacity(max_capacity)
    }
}

impl<S, H, C> Connect<S, H, C>
where
    S: ServerImpl,
    Error: From<S::Error>,
    H: Handler,
    C: ChannelAllocator,
{
    /// Perform the [negotiation protocol] and hand back the [`Server`] it
    /// produced.
    ///
    /// This sends [`MessageId::SERVER_HELLO`] and then drives the socket —
    /// including keepalive pings — until the client has answered with a
    /// [`MessageId::NEGOTIATE`] request and the reply to it has been flushed.
    ///
    /// Until that has happened the connection has no [`Server`], so there is no
    /// way to write a message which the client might not be able to decode.
    ///
    /// # Errors
    ///
    /// Errors if the connection goes away before the format has been
    /// negotiated, or if the client sends anything other than a negotiation as
    /// its first message. Both tear the connection down, since a peer which
    /// does not negotiate cannot be talked to safely.
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    pub async fn connect(self) -> Result<Server<S, H, C>, Error> {
        let now = Instant::now();

        let mut server = Server {
            handler: self.handler,
            pinned: Box::pin(Pinned {
                socket: self.socket,
                close_sleep: tokio::time::sleep_until(now + CLOSE_TIMEOUT),
                ping_sleep: tokio::time::sleep_until(now + PING_TIMEOUT),
            }),
            channels: self.channels,
            closing: false,
            pool: BufPool::new(self.max_capacity),
            outbound: VecDeque::new(),
            error: String::new(),
            last_ping: None,
            rng: SmallRng::seed_from_u64(self.seed),
            out: VecDeque::new(),
            socket_send: false,
            socket_flush: false,
            socket_recv: true,
            set: JoinSet::new(),
            format: Format::DEFAULT,
            mode: Mode::DEFAULT,
            formats: self.formats,
        };

        server.hello()?;
        server.negotiate().await?;
        Ok(server)
    }
}

/// Test if `formats` is willing to negotiate `format`.
#[inline]
fn accepts(formats: Option<&'static [Format]>, format: Format) -> bool {
    format.is_supported() && formats.is_none_or(|f| f.contains(&format))
}

/// The server side handle of the websocket protocol.
///
/// This can only be constructed by completing the [negotiation protocol]
/// through [`Connect::connect`], so its mere existence means that the
/// [`Format`] used for everything the server originates has been agreed with
/// the client.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
/// [negotiation protocol]: crate::api#negotiating-the-format
pub struct Server<S, H, C = Channels>
where
    S: ServerImpl,
    H: Handler,
{
    handler: H,
    pinned: Pin<Box<Pinned<S::Socket>>>,
    channels: C,
    closing: bool,
    pool: BufPool,
    outbound: VecDeque<Buf>,
    error: String,
    last_ping: Option<[u8; 4]>,
    rng: SmallRng,
    out: VecDeque<S::Message>,
    socket_send: bool,
    socket_flush: bool,
    /// Whether the peer's stream might still produce a message.
    ///
    /// A stream which has ended reports `Ready(None)` on every poll and
    /// registers no waker, so it has to stop being polled once it has ended or
    /// it masks every arm of [`Select`] behind it.
    socket_recv: bool,
    set: JoinSet<HandlerOutput<H>>,
    /// The format used for messages the server originates on this connection,
    /// as agreed by the [negotiation protocol].
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    format: Format,
    /// The mode every frame on this connection is spelled in, as decided by the
    /// frame the client negotiated with.
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    mode: Mode,
    /// Formats this server is willing to negotiate, or `None` to accept every
    /// format it was built with support for.
    formats: Option<&'static [Format]>,
}

impl<S, H, C> Server<S, H, C>
where
    S: ServerImpl,
    H: Handler,
{
    /// Get a reference to the handler.
    #[inline]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// The [`Format`] used for messages this server originates, such as
    /// broadcasts.
    ///
    /// This is fixed for the lifetime of the connection and was agreed with the
    /// client by [`Connect::connect`], which is why it is never in doubt here.
    #[inline]
    pub fn format(&self) -> Format {
        self.format
    }

    /// The [`Mode`] every frame on this connection is spelled in.
    ///
    /// This is decided by the frame the client negotiated with and is fixed for
    /// the lifetime of the connection, see the [wire format].
    ///
    /// [wire format]: crate::api#wire-format
    #[inline]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Test if this server is willing to negotiate `format`.
    #[inline]
    pub fn accepts(&self, format: Format) -> bool {
        accepts(self.formats, format)
    }
}

impl<S, H, C> Server<S, H, C>
where
    S: ServerImpl,
    Error: From<S::Error>,
    H: Handler,
    C: ChannelAllocator,
{
    /// Drive the socket until the client has negotiated a [`Format`].
    ///
    /// This is the connection step every peer is forced through, see
    /// [`Connect::connect`]. It deliberately understands nothing but
    /// [`MessageId::NEGOTIATE`] and the keepalive machinery, so no handler is
    /// ever invoked and nothing the user could write is in flight yet.
    ///
    /// It returns once the format is settled *and* the reply confirming it has
    /// been flushed, so the very next thing on the wire can safely use it.
    async fn negotiate(&mut self) -> Result<(), Error> {
        let mut negotiated = false;
        // NB: Held rather than returned immediately so that the close frame
        // explaining the violation makes it onto the wire first.
        let mut failure = None::<Error>;

        loop {
            let drained = self.out.is_empty() && !self.socket_flush;

            if failure.is_some() && drained {
                break;
            }

            if negotiated && drained && self.outbound.is_empty() {
                break;
            }

            self.handle_send()?;

            let result = {
                let inner = Select::<S::Socket, H> {
                    pinned: self.pinned.as_mut(),
                    wants_socket_recv: self.socket_recv,
                    wants_socket_send: !self.socket_send,
                    wants_socket_flush: self.socket_flush,
                    set: &mut self.set,
                };

                inner.await
            };

            match result {
                Output::Close => {
                    return Err(Error::new(ErrorKind::NotNegotiated));
                }
                Output::Ping => {
                    self.handle_ping()?;
                }
                Output::Recv(message) => {
                    let Some(message) = message else {
                        return Err(Error::new(ErrorKind::NotNegotiated));
                    };

                    match message? {
                        // NB: The frame type is the mode, so the negotiation is
                        // what tells the server which envelope the rest of the
                        // connection is spelled in.
                        Message::Text(bytes) => match self.handle_negotiate(Mode::Text, bytes) {
                            Ok(()) => negotiated = true,
                            Err(error) => failure = Some(error),
                        },
                        Message::Binary(bytes) => {
                            match self.handle_negotiate(Mode::Binary, bytes) {
                                Ok(()) => negotiated = true,
                                Err(error) => failure = Some(error),
                            }
                        }
                        Message::Ping(payload) => {
                            self.out.push_back(S::pong(payload));
                        }
                        Message::Pong(data) => {
                            self.handle_pong(data)?;
                        }
                        Message::Close => {
                            return Err(Error::new(ErrorKind::NotNegotiated));
                        }
                    }
                }
                Output::Send(result) => {
                    result?;
                    self.socket_send = true;
                }
                Output::Flushed(result) => {
                    result?;
                    self.socket_flush = false;
                }
                Output::Handle(..) => {
                    // NB: No handler can have been spawned yet, since requests
                    // are only dispatched after negotiation.
                }
            }
        }

        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Process the single message which is legal before a format has been
    /// negotiated.
    ///
    /// Anything else closes the connection, since a peer which skips
    /// negotiation cannot be sent broadcasts safely.
    fn handle_negotiate(&mut self, mode: Mode, bytes: Bytes) -> Result<(), Error> {
        let mut at = 0;

        let header: RequestHeader = match format::decode_envelope(mode, &bytes, &mut at) {
            Ok(header) => header,
            Err(error) => {
                self.out
                    .push_back(S::close(CLOSE_PROTOCOL_ERROR, "Invalid request header"));
                return Err(Error::new(ErrorKind::NegotiateHeader { error }));
            }
        };

        if MessageId::new(header.id) != Some(MessageId::NEGOTIATE) {
            self.out.push_back(S::close(
                CLOSE_PROTOCOL_ERROR,
                "Expected a negotiation as the first message",
            ));

            return Err(Error::new(ErrorKind::ExpectedNegotiate { id: header.id }));
        }

        // NB: The connection stays up on a rejected format and settles on the
        // default instead, which is what the client falls back to. The error
        // tells it which formats it could have asked for.
        let Some(format) = Format::from_u8(header.format) else {
            self.format_error_message(format_args!(
                "Unknown format id {}, supported: {}",
                header.format,
                SupportedFormats(self.formats)
            ))?;

            self.format = Format::DEFAULT;
            return self.send_error(&header);
        };

        if !self.accepts(format) {
            self.format_error_message(format_args!(
                "Unsupported format `{format}`, supported: {}",
                SupportedFormats(self.formats)
            ))?;

            tracing::debug!(?format, "Rejected format");
            self.format = Format::DEFAULT;
            return self.send_error(&header);
        }

        // NB: A text frame has to be valid UTF-8 in its entirety, so a format
        // which is not human readable cannot be carried in one. The connection
        // settles on the defaults for both, and the reply which says so goes
        // out in the default mode, which is what tells the client.
        if !mode.accepts(format) {
            self.format_error_message(format_args!(
                "Mode `{mode}` cannot carry the `{format}` format"
            ))?;

            tracing::debug!(?mode, ?format, "Rejected mode");
            self.format = Format::DEFAULT;
            return self.send_error(&header);
        }

        tracing::debug!(?mode, ?format, "Negotiated format");
        self.format = format;
        self.mode = mode;
        self.send_negotiated(&header, format)
    }

    /// Acknowledge a negotiation by echoing the format that was accepted.
    fn send_negotiated(&mut self, header: &RequestHeader, format: Format) -> Result<(), Error> {
        let mode = self.mode;

        let buf = self.pool.with(|buf| {
            let mut writer = buf.writer();

            let result = writer.envelope(
                mode,
                &ResponseHeader {
                    serial: header.serial,
                    broadcast: 0,
                    error: 0,
                    format: format.to_u8(),
                    channel: header.channel,
                },
            );

            result.map_err(Error::encode_connect_header)?;
            writer.flush();
            Ok::<_, Error>(())
        })?;

        self.outbound.push_back(buf);
        Ok(())
    }

    /// Run the server.
    ///
    /// This must be called to handle buffered outgoing and incoming messages.
    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            // NB: `socket_flush` is part of this because `start_send` only
            // hands a message to the sink. Breaking as soon as the queues are
            // empty leaves the close frame in the sink's buffer, never on the
            // wire. Waiting is bounded by the deadline `begin_closing` arms.
            if self.closing && self.out.is_empty() && self.outbound.is_empty() && !self.socket_flush
            {
                break;
            }

            self.handle_send()?;

            let result = {
                let inner = Select::<S::Socket, H> {
                    pinned: self.pinned.as_mut(),
                    wants_socket_recv: self.socket_recv,
                    wants_socket_send: !self.socket_send,
                    wants_socket_flush: self.socket_flush,
                    set: &mut self.set,
                };

                inner.await
            };

            match result {
                Output::Close => {
                    // The deadline re-armed by `begin_closing` elapsed, so the
                    // peer never picked up the close frame which was queued for
                    // it. Drop the connection rather than queue another one.
                    if self.closing {
                        break;
                    }

                    self.out
                        .push_back(S::close(CLOSE_NORMAL, "connection timed out"));
                    self.begin_closing();
                }
                Output::Ping => {
                    self.handle_ping()?;
                }
                Output::Recv(message) => {
                    let Some(message) = message else {
                        // NB: An ended stream is `Ready(None)` on every poll,
                        // so it has to be fused here or it masks the send and
                        // flush arms and nothing can ever drain.
                        self.socket_recv = false;
                        self.begin_closing();
                        continue;
                    };

                    match message? {
                        // NB: The mode is fixed by the negotiation, so a frame
                        // of the other type carries an envelope this connection
                        // has no way to read.
                        Message::Text(bytes) => {
                            if self.mode.is_text() {
                                self.handle_message(bytes).await?;
                            } else {
                                self.out.push_back(S::close(
                                    CLOSE_PROTOCOL_ERROR,
                                    "Unexpected text message",
                                ));
                                self.begin_closing();
                            }
                        }
                        Message::Binary(bytes) => {
                            if self.mode.is_text() {
                                self.out.push_back(S::close(
                                    CLOSE_PROTOCOL_ERROR,
                                    "Unexpected binary message",
                                ));
                                self.begin_closing();
                            } else {
                                self.handle_message(bytes).await?;
                            }
                        }
                        Message::Ping(payload) => {
                            self.out.push_back(S::pong(payload));
                        }
                        Message::Pong(data) => {
                            self.handle_pong(data)?;
                        }
                        Message::Close => {
                            self.begin_closing();
                        }
                    }
                }
                Output::Send(result) => {
                    if let Err(err) = result {
                        return Err(Error::from(err));
                    };

                    self.socket_send = true;
                }
                Output::Flushed(result) => {
                    if let Err(err) = result {
                        return Err(Error::from(err));
                    };

                    self.socket_flush = false;
                }
                Output::Handle(result, header, buf) => {
                    let err = 'err: {
                        let res = match result {
                            Ok(res) => res,
                            Err(error) => {
                                self.format_error(error)?;
                                break 'err true;
                            }
                        };

                        let res = match res.into_response() {
                            Ok(res) => res,
                            Err(error) => {
                                self.format_error_message(error)?;
                                break 'err true;
                            }
                        };

                        if !res.handled {
                            self.format_error_message(format_args!(
                                "No support for request {}",
                                header.id
                            ))?;
                            break 'err true;
                        }

                        self.outbound.push_back(buf);
                        false
                    };

                    if err {
                        self.send_error(&header)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Write a broadcast message.
    ///
    /// Note that the written message is buffered, and will be sent when
    /// [`Server::run`] is called.
    pub fn broadcast<T>(&mut self, message: T) -> Result<(), Error>
    where
        T: Event,
    {
        self.broadcast_in(message, ChannelId::NONE)
    }

    /// Write a broadcast message over a specific `channel`.
    ///
    /// Note that the written message is buffered, and will be sent when
    /// [`Server::run`] is called.
    pub fn broadcast_in<T>(&mut self, message: T, channel: ChannelId) -> Result<(), Error>
    where
        T: Event,
    {
        tracing::debug!(id = ?<T::Broadcast as Broadcast>::ID, "Broadcast");

        let format = self.format;
        let mode = self.mode;

        let buf = self.pool.with(|buf| {
            let mut writer = buf.writer();

            writer
                .envelope(
                    mode,
                    &ResponseHeader {
                        serial: 0,
                        broadcast: <T::Broadcast as Broadcast>::ID.get(),
                        error: 0,
                        format: format.to_u8(),
                        channel,
                    },
                )
                .map_err(Error::encode_broadcast_header)?;

            writer
                .body(format, &message)
                .map_err(Error::encode_broadcast)?;
            writer.flush();
            Ok::<_, Error>(())
        })?;

        self.outbound.push_back(buf);
        Ok(())
    }

    /// Write a broadcast message over a specific connection.
    ///
    /// Note that the written message is buffered, and will be sent when
    /// [`Server::run`] is called.
    fn hello(&mut self) -> Result<(), Error> {
        tracing::debug!("Hello");

        let mut buf = self.pool.get();

        let result = (|| {
            let mut writer = buf.writer();

            writer
                // NB: Nothing has been heard from the client yet, so this is
                // the one message which cannot be spelled in a negotiated mode.
                .envelope(
                    Mode::DEFAULT,
                    &ResponseHeader {
                        serial: 0,
                        broadcast: MessageId::SERVER_HELLO.get(),
                        error: 0,
                        // NB: Carries no body, so no format applies.
                        format: 0,
                        channel: ChannelId::NONE,
                    },
                )
                .map_err(Error::encode_broadcast_header)?;

            writer.flush();
            Ok::<_, Error>(())
        })();

        if result.is_err() {
            self.pool.put(buf);
        } else {
            self.outbound.push_back(buf);
        }

        Ok(())
    }

    fn format_error_message(&mut self, error: impl fmt::Display) -> Result<(), Error> {
        self.error.clear();

        if write!(self.error, "{error}").is_err() {
            self.error.clear();
            return Err(Error::new(ErrorKind::FormatError));
        }

        Ok(())
    }

    fn format_error(&mut self, error: impl core::error::Error) -> Result<(), Error> {
        self.error.clear();

        if write!(self.error, "{error:#}").is_err() {
            self.error.clear();
            return Err(Error::new(ErrorKind::FormatError));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, bytes))]
    async fn handle_message(&mut self, bytes: Bytes) -> Result<(), Error> {
        let mut at = 0;

        let header: RequestHeader = match format::decode_envelope(self.mode, &bytes, &mut at) {
            Ok(header) => header,
            Err(error) => {
                tracing::debug!(?error, "Invalid request header");
                self.out
                    .push_back(S::close(CLOSE_PROTOCOL_ERROR, "Invalid request header"));
                self.begin_closing();
                return Ok(());
            }
        };

        let err = 'err: {
            let Some(id) = MessageId::new(header.id) else {
                self.format_error_message(format_args!("Unsupported message id {}", header.id))?;
                break 'err true;
            };

            match id {
                MessageId::CONNECT => {
                    let Some(channel) = self.channels.next().await else {
                        self.format_error_message(format_args!(
                            "Failed to allocate connection ID"
                        ))?;

                        break 'err true;
                    };

                    self.handler.open_channel(channel).await;

                    let mut buf = self.pool.get();
                    let mode = self.mode;

                    let result = (|| {
                        let mut writer = buf.writer();

                        let result = writer.envelope(
                            mode,
                            &ResponseHeader {
                                serial: header.serial,
                                broadcast: 0,
                                error: 0,
                                format: 0,
                                channel,
                            },
                        );

                        result.map_err(Error::encode_connect_header)?;
                        writer.flush();
                        Ok::<_, Error>(())
                    })();

                    if result.is_err() {
                        self.pool.put(buf);
                    } else {
                        self.outbound.push_back(buf);
                    }

                    result?;
                    break 'err false;
                }
                MessageId::DISCONNECT => {
                    self.channels.free(header.channel).await;
                    self.handler.close_channel(header.channel).await;
                    break 'err false;
                }
                MessageId::NEGOTIATE => {
                    // NB: The format is settled once and for all by the
                    // connection step, see `Connect::connect`. Letting it move
                    // afterwards would mean messages already queued for the old
                    // format go out under the new one.
                    let format = self.format;

                    self.format_error_message(format_args!(
                        "Format `{format}` has already been negotiated"
                    ))?;

                    break 'err true;
                }
                _ => {
                    let Some(format) = Format::from_u8(header.format) else {
                        self.format_error_message(format_args!(
                            "Unknown format id {}",
                            header.format
                        ))?;

                        break 'err true;
                    };

                    if !self.accepts(format) {
                        self.format_error_message(format_args!(
                            "Unsupported format `{format}`, supported: {}",
                            SupportedFormats(self.formats)
                        ))?;

                        break 'err true;
                    }

                    // NB: The response goes back in the same frame type the
                    // request arrived in, so a format which cannot be spelled
                    // that way has nowhere to go.
                    if !self.mode.accepts(format) {
                        let mode = self.mode;

                        self.format_error_message(format_args!(
                            "Mode `{mode}` cannot carry the `{format}` format"
                        ))?;

                        break 'err true;
                    }

                    let id = <H::Id as Id>::from_id(id);
                    self.handle_request(bytes, at, header, id, format);
                    return Ok(());
                }
            }
        };

        if err {
            self.send_error(&header)?;
        }

        Ok(())
    }

    fn send_error(&mut self, header: &RequestHeader) -> Result<(), Error> {
        // NB: Errors are encoded with the connection format rather than the
        // format the request asked for, since the request might have failed
        // precisely because that format is not supported. The client reads the
        // format back out of the response envelope either way.
        let format = self.format;
        let mode = self.mode;

        let buf = self.pool.with(|buf| {
            // Reset the buffer to the previous start point.
            let mut writer = buf.writer();

            let result = writer.envelope(
                mode,
                &ResponseHeader {
                    serial: header.serial,
                    broadcast: 0,
                    error: MessageId::ERROR_MESSAGE.get(),
                    format: format.to_u8(),
                    channel: header.channel,
                },
            );

            result.map_err(Error::encode_error_message_header)?;

            let result = writer.body(
                format,
                &ErrorMessage {
                    message: &self.error,
                },
            );

            result.map_err(Error::encode_error_message)?;
            writer.flush();
            Ok::<_, Error>(())
        })?;

        self.outbound.push_back(buf);
        Ok(())
    }

    /// Begin winding the connection down.
    ///
    /// This re-arms the close deadline so that draining whatever is left to
    /// send is bounded too. Re-arming is also what keeps [`Server::run`] from
    /// spinning: an elapsed [`Sleep`] reports `Ready` every time it is polled,
    /// so leaving it elapsed would make [`Select`] hand back [`Output::Close`]
    /// over and over without the loop ever making progress.
    fn begin_closing(&mut self) {
        if self.closing {
            return;
        }

        self.closing = true;

        let (close_sleep, _, _) = self.pinned.as_mut().project();
        close_sleep.reset(Instant::now() + CLOSE_TIMEOUT);
    }

    #[tracing::instrument(skip(self))]
    fn handle_ping(&mut self) -> Result<(), Error> {
        let (_, mut ping_sleep, _) = self.pinned.as_mut().project();

        let payload = self.rng.random::<u32>();
        let payload = payload.to_ne_bytes();

        self.last_ping = Some(payload);

        tracing::debug!(data = ?&payload[..], "Sending ping");

        self.out
            .push_back(S::ping(Bytes::from_owner(Vec::from(payload))));

        let now = Instant::now();
        ping_sleep.as_mut().reset(now + PING_TIMEOUT);
        Ok(())
    }

    #[tracing::instrument(skip(self, payload))]
    fn handle_pong(&mut self, payload: Bytes) -> Result<(), Error> {
        let (close_sleep, ping_sleep, _) = self.pinned.as_mut().project();

        tracing::debug!(payload = ?&payload[..], "Pong");

        let Some(expected) = self.last_ping else {
            tracing::debug!("No ping sent");
            return Ok(());
        };

        if expected[..] != payload[..] {
            tracing::debug!(?expected, ?payload, "Pong doesn't match");
            return Ok(());
        }

        let now = Instant::now();

        close_sleep.reset(now + CLOSE_TIMEOUT);
        ping_sleep.reset(now + PING_TIMEOUT);
        self.last_ping = None;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_send(&mut self) -> Result<(), Error> {
        let (_, _, mut socket) = self.pinned.as_mut().project();

        if self.socket_send
            && let Some(message) = self.out.pop_front()
        {
            socket.as_mut().start_send(message)?;
            self.socket_flush = true;
            self.socket_send = false;
        }

        while self.socket_send
            && let Some(buf) = self.outbound.front_mut()
        {
            let Some(frame) = buf.read()? else {
                if let Some(buf) = self.outbound.pop_front() {
                    self.pool.put(buf);
                }

                continue;
            };

            let message = match self.mode {
                Mode::Binary => S::binary(frame),
                Mode::Text => S::text(frame),
            };

            socket.as_mut().start_send(message)?;

            self.socket_flush = true;
            self.socket_send = false;
            break;
        }

        Ok(())
    }

    fn handle_request(
        &mut self,
        bytes: Bytes,
        offset: usize,
        header: RequestHeader,
        id: H::Id,
        format: Format,
    ) {
        tracing::debug!(header.serial, ?id, ?format, "Got request");

        let mut buf = self.pool.get();
        let handler = self.handler.clone();
        let mode = self.mode;

        self.set.spawn(async move {
            if offset > bytes.len() {
                let kind = ErrorKind::OutOfBounds {
                    offset,
                    len: bytes.len(),
                };

                return (Err(Error::new(kind)), header, buf);
            }

            let mut incoming = Incoming {
                error: None,
                buf: &bytes,
                at: offset,
                format,
                channel: header.channel,
            };

            let mut outgoing = Outgoing {
                serial: Some(header.serial),
                error: None,
                buf: &mut buf,
                format,
                mode,
                channel: header.channel,
            };

            let response = handler.handle(id, &mut incoming, &mut outgoing).await;

            if let Some(error) = incoming.error.take() {
                return (Err(Error::incoming(error)), header, buf);
            }

            if let Some(error) = outgoing.error.take() {
                return (Err(Error::outgoing(error)), header, buf);
            }

            (Ok(response), header, buf)
        });
    }
}

enum Output<E, R> {
    /// The connection should be closed.
    Close,
    /// A ping message was received.
    Ping,
    /// A message was received.
    Recv(Option<Result<Message, E>>),
    /// A message is ready to be sent.
    Send(Result<(), E>),
    /// Outgoing messages have been successfully flushed.
    Flushed(Result<(), E>),
    /// Handle output.
    Handle(Result<R, Error>, RequestHeader, Buf),
}

struct Select<'a, S, H>
where
    H: Handler,
{
    pinned: Pin<&'a mut Pinned<S>>,
    wants_socket_recv: bool,
    wants_socket_send: bool,
    wants_socket_flush: bool,
    set: &'a mut JoinSet<HandlerOutput<H>>,
}

impl<S, H> Future for Select<'_, S, H>
where
    S: SocketImpl,
    H: Handler,
{
    type Output = Output<S::Error, H::Response>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let close;
        let ping;
        let mut socket;
        let wants_socket_recv;
        let wants_socket_send;
        let wants_socket_flush;
        let set;

        // SAFETY: This type is not Unpin.
        unsafe {
            let this = Pin::get_unchecked_mut(self);
            (close, ping, socket) = this.pinned.as_mut().project();
            wants_socket_recv = this.wants_socket_recv;
            wants_socket_send = this.wants_socket_send;
            wants_socket_flush = this.wants_socket_flush;
            set = &mut this.set;
        };

        if close.poll(cx).is_ready() {
            return Poll::Ready(Output::Close);
        }

        if ping.poll(cx).is_ready() {
            return Poll::Ready(Output::Ping);
        }

        if wants_socket_recv && let Poll::Ready(output) = socket.as_mut().poll_next(cx) {
            return Poll::Ready(Output::Recv(output));
        }

        if wants_socket_send && let Poll::Ready(result) = socket.as_mut().poll_ready(cx) {
            return Poll::Ready(Output::Send(result));
        }

        if wants_socket_flush && let Poll::Ready(result) = socket.as_mut().poll_flush(cx) {
            return Poll::Ready(Output::Flushed(result));
        }

        // NB: An empty `JoinSet` is `Ready(None)` and registers no waker, so
        // this drops through to `Pending` having registered nothing of its
        // own. That is only safe because the two deadlines above are polled
        // unconditionally and have registered theirs.
        if let Poll::Ready(output) = set.poll_join_next(cx)
            && let Some(output) = output
        {
            let output = match output {
                Ok(output) => output,
                Err(error) => {
                    tracing::debug!(?error, "Join error in handler task");
                    return Poll::Ready(Output::Close);
                }
            };

            let (result, header, buf) = output;
            return Poll::Ready(Output::Handle(result, header, buf));
        }

        Poll::Pending
    }
}

/// The buffer for incoming requests.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub struct Incoming<'de> {
    error: Option<format::Error>,
    buf: &'de [u8],
    at: usize,
    format: Format,
    channel: ChannelId,
}

impl<'de> Incoming<'de> {
    /// The channel over which the incoming request was received.
    ///
    /// This is [`ChannelId::NONE`] unless the packet belongs to a response to a
    /// handle constructed with [`Handle::channel`].
    ///
    /// [`Handle::channel`]: crate::web::Handle::channel
    pub fn channel(&self) -> ChannelId {
        self.channel
    }

    /// The [`Format`] the incoming request body is encoded with.
    ///
    /// This is the format the client declared for this particular request, and
    /// is also the format the response will be written with.
    #[inline]
    pub fn format(&self) -> Format {
        self.format
    }

    /// Read a request and return `Some(T)` if the request was successfully
    /// decoded.
    ///
    /// Note that any failure to decode will be propagated as an error
    /// automatically, the user does not have to deal with it themselves.
    /// Instead, failure to decode should be treated as if the request was
    /// unhandled by returning for example `false` or `Option::None`.
    #[inline]
    pub fn read<T>(&mut self) -> Option<T>
    where
        T: DecodeBody<'de>,
    {
        match self.format.decode(self.buf, &mut self.at) {
            Ok(value) => Some(value),
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }
}

/// The buffer for outgoing responses.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub struct Outgoing<'a> {
    serial: Option<u32>,
    error: Option<format::Error>,
    buf: &'a mut Buf,
    format: Format,
    mode: Mode,
    channel: ChannelId,
}

impl Outgoing<'_> {
    /// The [`Format`] the response will be encoded with, which is the format
    /// the corresponding request declared.
    #[inline]
    pub fn format(&self) -> Format {
        self.format
    }

    /// The [`Mode`] the response will be spelled in, which is the mode the
    /// corresponding request arrived in.
    #[inline]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Write a response.
    ///
    /// This can only be called once. Calling this multiple times has no effect.
    ///
    /// See [`server()`] for how to use with `axum`.
    ///
    /// [`server()`]: crate::axum08::server
    pub fn write<T>(&mut self, value: T)
    where
        T: EncodeBody,
    {
        let Some(serial) = self.serial.take() else {
            return;
        };

        let mut writer = self.buf.writer();

        let result = writer.envelope(
            self.mode,
            &ResponseHeader {
                serial,
                broadcast: 0,
                error: 0,
                format: self.format.to_u8(),
                channel: self.channel,
            },
        );

        if let Err(error) = result {
            self.error = Some(error);
            return;
        }

        if let Err(error) = writer.body(self.format, &value) {
            self.error = Some(error);
        }

        writer.flush();
    }
}

/// Scramble a sequential channel id into a value that looks random to clients.
///
/// Uses an odd multiply followed by an XOR-shift (bijective over u16, preserves
/// 0, self-inverse).
#[inline]
fn scramble_channel(x: u16) -> u16 {
    let x = x.wrapping_mul(0x9285);
    x ^ (x >> 8)
}

/// Inverse of [`scramble_channel`].
#[inline]
#[cfg(test)]
fn unscramble_channel(x: u16) -> u16 {
    let x = x ^ (x >> 8);
    x.wrapping_mul(0x964d)
}

#[test]
fn test_scramble() {
    assert_eq!(scramble_channel(0), 0);
    assert_eq!(unscramble_channel(0), 0);

    for i in 1..=u16::MAX {
        let scrambled = scramble_channel(i);
        let unscrambled = unscramble_channel(scrambled);
        assert_eq!(i, unscrambled, "Failed to unscramble channel id");
    }
}

#[derive(Default)]
struct ChannelsInner {
    last: u16,
    free: VecDeque<NonZeroU16>,
}

/// A global channel allocator which can be cloned and re-used across multiple
/// servers allowing channels across servers to have distinct channel
/// identifiers.
#[derive(Default, Clone)]
pub struct Channels {
    inner: Arc<Mutex<ChannelsInner>>,
}

impl ChannelAllocator for Channels {
    #[inline]
    async fn next(&self) -> Option<ChannelId> {
        let mut inner = self.inner.lock().await;

        if let Some(id) = inner.free.pop_front() {
            return Some(ChannelId::from_u16(id.get()));
        }

        let id = NonZeroU16::new(inner.last.wrapping_add(1))?;
        inner.last = id.get();

        tracing::debug!(?id, "Allocated channel id");
        Some(ChannelId::from_u16(scramble_channel(id.get())))
    }

    #[inline]
    async fn free(&self, id: ChannelId) {
        tracing::debug!(?id, "Freeing channel id");

        let mut inner = self.inner.lock().await;

        if let Some(id) = NonZeroU16::new(id.raw()) {
            inner.free.push_back(id);
        }
    }
}

/// Renders the formats a server is willing to negotiate, for error messages.
struct SupportedFormats(Option<&'static [Format]>);

impl fmt::Display for SupportedFormats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;

        for format in Format::supported() {
            if let Some(formats) = self.0
                && !formats.contains(&format)
            {
                continue;
            }

            if !first {
                f.write_str(", ")?;
            }

            write!(f, "`{format}`")?;
            first = false;
        }

        if first {
            f.write_str("none")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use std::thread_local;

    use super::*;

    /// How many close frames the loop is allowed to manufacture before the test
    /// concludes that it is spinning.
    ///
    /// A close frame is the only observable thing the wind-down path produces,
    /// so this is what bounds the test. Without it a regression makes
    /// [`Server::run`] loop forever inside a single poll and the test hangs
    /// rather than fails.
    const CLOSE_BUDGET: usize = 4;

    /// How many times a stream which has ended is allowed to be polled before
    /// the test concludes that [`Server::run`] is spinning on it.
    ///
    /// Such a stream reports `Ready(None)` on every poll, so a regression makes
    /// the loop poll it forever inside a single task poll and the test hangs
    /// rather than fails.
    const RECV_BUDGET: usize = 4;

    thread_local! {
        /// The number of close frames [`TestServerImpl::close`] has handed out
        /// on this thread, which is one test.
        static CLOSE_FRAMES: Cell<usize> = const { Cell::new(0) };
    }

    #[derive(Debug)]
    enum TestError {}

    impl From<TestError> for Error {
        #[inline]
        fn from(error: TestError) -> Self {
            match error {}
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum TestMessage {
        Ping(Bytes),
        Pong(Bytes),
        Binary(Vec<u8>),
        Text(Vec<u8>),
        Close(u16),
    }

    /// A socket which never hears anything from its peer, which is the state a
    /// connection is in when its close deadline elapses.
    #[derive(Default)]
    struct TestSocket {
        sent: Vec<TestMessage>,
        /// Whether the peer has gone away, which makes the stream report
        /// `Ready(None)` on every poll for the rest of its life.
        ended: bool,
        polls_after_end: usize,
        /// Whether the write side has backpressure, which is what a real sink
        /// does while the socket buffer it is writing into has not drained.
        write_blocked: bool,
    }

    impl socket_sealed::Sealed for TestSocket {}

    impl SocketImpl for TestSocket {
        type Message = TestMessage;
        type Error = TestError;

        fn poll_next(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<Message, Self::Error>>> {
            // SAFETY: Nothing in this socket is structurally pinned.
            let this = unsafe { Pin::get_unchecked_mut(self) };

            if !this.ended {
                return Poll::Pending;
            }

            this.polls_after_end += 1;

            assert!(
                this.polls_after_end <= RECV_BUDGET,
                "`Server::run` is spinning: an ended stream was polled {} times without any progress",
                this.polls_after_end
            );

            Poll::Ready(None)
        }

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            // SAFETY: Nothing in this socket is structurally pinned.
            if unsafe { Pin::get_unchecked_mut(self) }.write_blocked {
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: Self::Message) -> Result<(), Self::Error> {
            // SAFETY: Nothing in this socket is structurally pinned.
            unsafe { Pin::get_unchecked_mut(self).sent.push(item) };
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            // SAFETY: Nothing in this socket is structurally pinned.
            if unsafe { Pin::get_unchecked_mut(self) }.write_blocked {
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        }
    }

    struct TestServerImpl;

    impl server_sealed::Sealed for TestServerImpl {}

    impl ServerImpl for TestServerImpl {
        type Error = TestError;
        type Message = TestMessage;
        type Socket = TestSocket;

        fn ping(data: Bytes) -> Self::Message {
            TestMessage::Ping(data)
        }

        fn pong(data: Bytes) -> Self::Message {
            TestMessage::Pong(data)
        }

        fn binary(data: &[u8]) -> Self::Message {
            TestMessage::Binary(data.to_vec())
        }

        fn text(data: &[u8]) -> Self::Message {
            TestMessage::Text(data.to_vec())
        }

        fn close(code: u16, _: &str) -> Self::Message {
            let frames = CLOSE_FRAMES.with(|frames| {
                let count = frames.get() + 1;
                frames.set(count);
                count
            });

            assert!(
                frames <= CLOSE_BUDGET,
                "`Server::run` is spinning: {frames} close frames without any progress"
            );

            TestMessage::Close(code)
        }
    }

    #[derive(Debug)]
    struct TestId;

    impl Id for TestId {
        fn id(&self) -> MessageId {
            MessageId::NEGOTIATE
        }

        fn from_id(_: MessageId) -> Self {
            Self
        }

        fn __do_not_implement_id() {}
    }

    #[derive(Clone)]
    struct TestHandler;

    impl Handler for TestHandler {
        type Id = TestId;
        type Response = bool;

        async fn handle(&self, _: Self::Id, _: &mut Incoming<'_>, _: &mut Outgoing<'_>) -> bool {
            false
        }
    }

    /// Build a negotiated server whose close deadline elapsed `elapsed` ago.
    fn timed_out_server(elapsed: Duration) -> Server<TestServerImpl, TestHandler, Channels> {
        server_with(Instant::now() - elapsed)
    }

    /// The test socket behind the server's pinned state.
    fn socket(server: &mut Server<TestServerImpl, TestHandler, Channels>) -> &mut TestSocket {
        let (_, _, socket) = server.pinned.as_mut().project();
        // SAFETY: Nothing in this socket is structurally pinned.
        unsafe { Pin::get_unchecked_mut(socket) }
    }

    /// A response buffer of the shape a handler leaves behind in `outbound`.
    fn response(server: &Server<TestServerImpl, TestHandler, Channels>) -> Buf {
        server
            .pool
            .with(|buf| {
                let mut writer = buf.writer();

                writer
                    .envelope(
                        Mode::DEFAULT,
                        &ResponseHeader {
                            serial: 1,
                            broadcast: 0,
                            error: 0,
                            format: Format::DEFAULT.to_u8(),
                            channel: ChannelId::NONE,
                        },
                    )
                    .map_err(Error::encode_connect_header)?;

                writer.flush();
                Ok::<_, Error>(())
            })
            .unwrap()
    }

    /// Build a negotiated server which winds down at `close_deadline`.
    fn server_with(close_deadline: Instant) -> Server<TestServerImpl, TestHandler, Channels> {
        let now = Instant::now();

        Server {
            handler: TestHandler,
            pinned: Box::pin(Pinned {
                socket: TestSocket::default(),
                close_sleep: tokio::time::sleep_until(close_deadline),
                ping_sleep: tokio::time::sleep_until(now + PING_TIMEOUT),
            }),
            channels: Channels::default(),
            closing: false,
            pool: BufPool::new(MAX_CAPACITY),
            outbound: VecDeque::new(),
            error: String::new(),
            last_ping: None,
            rng: SmallRng::seed_from_u64(DEFAULT_SEED),
            out: VecDeque::new(),
            socket_send: false,
            socket_flush: false,
            socket_recv: true,
            set: JoinSet::new(),
            format: Format::DEFAULT,
            mode: Mode::DEFAULT,
            formats: None,
        }
    }

    /// An elapsed close deadline must wind the connection down once, rather
    /// than being reported by [`Select`] over and over.
    ///
    /// A [`Sleep`] which has elapsed is `Ready` on every poll, so a close
    /// deadline which is left elapsed turns [`Server::run`] into a busy loop
    /// which queues an unbounded number of close frames and never returns.
    #[tokio::test]
    async fn close_deadline_winds_down_once() {
        let mut server = timed_out_server(Duration::from_secs(1));

        server.run().await.unwrap();

        assert_eq!(socket(&mut server).sent, [TestMessage::Close(CLOSE_NORMAL)]);
        assert!(server.out.is_empty());
        assert!(server.closing);
        // The close frame has to reach the wire, not just the sink's buffer.
        assert!(!server.socket_flush);
    }

    /// The close deadline elapsing while the connection is already winding
    /// down must tear it down instead of queueing yet another close frame.
    #[tokio::test]
    async fn close_deadline_while_closing_gives_up() {
        let mut server = timed_out_server(Duration::from_secs(1));

        // The state left behind by a close frame which the socket has not been
        // able to take yet, with the wind-down deadline elapsed on top of it.
        server.closing = true;
        server.out.push_back(TestMessage::Close(CLOSE_NORMAL));

        server.run().await.unwrap();

        assert!(socket(&mut server).sent.is_empty());
        assert_eq!(server.out.len(), 1);
    }

    /// A peer which disconnects with a response still queued must not pin the
    /// connection task forever.
    ///
    /// A stream which has ended reports `Ready(None)` on every poll, which sits
    /// ahead of the send and flush arms in [`Select`] and used to mask them, so
    /// `outbound` could never drain and [`Server::run`] could never reach its
    /// break.
    ///
    /// Nothing here may lean on the close deadline: a loop which never yields
    /// is exactly the condition under which the runtime cannot deliver a timer,
    /// so the deadline is left a full [`CLOSE_TIMEOUT`] out and this test would
    /// hang rather than pass if the wind-down needed it.
    #[tokio::test]
    async fn ended_stream_drains_outbound() {
        let mut server = server_with(Instant::now() + CLOSE_TIMEOUT);

        // The state a connection is left in when the peer goes away after its
        // request has been handled but before the response reached the socket.
        server.closing = true;
        server.socket_flush = true;
        server.outbound.push_back(response(&server));
        socket(&mut server).ended = true;

        server.run().await.unwrap();

        let sent = socket(&mut server).sent.as_slice();
        assert!(matches!(sent, [TestMessage::Binary(..)]));
        assert!(server.outbound.is_empty());
        assert!(!server.socket_flush);
        assert!(!server.socket_recv);
    }

    /// A peer which is gone while the write side still has backpressure must
    /// wind down on the close deadline rather than spin.
    ///
    /// This is the case which separates fusing the stream from merely polling
    /// the send and flush arms ahead of it. Reordering the arms would rescue
    /// [`ended_stream_drains_outbound`], where the sink takes the write
    /// immediately, but not this: with the write side `Pending` there is no arm
    /// left to reach, so a stream which is still polled after it ended hands
    /// back `Ready(None)` forever and [`Server::run`] never yields.
    ///
    /// Fused, [`Select`] returns `Pending`, the task parks, and the deadline
    /// gets the chance to fire which the spin used to deny it. Leaning on it
    /// here is the point, so the deadline is short.
    #[tokio::test]
    async fn blocked_write_winds_down_on_deadline() {
        let mut server = server_with(Instant::now() + Duration::from_millis(50));

        server.closing = true;
        server.socket_flush = true;
        server.outbound.push_back(response(&server));

        {
            let socket = socket(&mut server);
            socket.ended = true;
            socket.write_blocked = true;
        }

        server.run().await.unwrap();

        // Nothing could be delivered, which is the honest outcome against a
        // socket which never took the write. Ending at all is what matters.
        assert!(socket(&mut server).sent.is_empty());
        assert_eq!(server.outbound.len(), 1);
        assert!(!server.socket_recv);
    }
}
