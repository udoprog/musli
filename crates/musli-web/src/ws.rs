//! The server side of the websocket protocol.
//!
//! See [`server()`] for how to use with [axum].
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
use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::storage;
use musli::{Decode, Encode};
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::{Duration, Instant, Sleep};

use crate::Buf;
use crate::api::{
    Broadcast, ChannelId, ErrorMessage, Event, Id, MessageId, RequestHeader, ResponseHeader,
};
use crate::buf::{BufPool, InvalidFrame};

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
    Text,
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
        error: storage::Error,
    },
    Outgoing {
        error: storage::Error,
    },
    EncodeBroadcastHeader {
        error: storage::Error,
    },
    EncodeBroadcast {
        error: storage::Error,
    },
    EncodeConnectHeader {
        error: storage::Error,
    },
    ErrorMessageHeader {
        error: storage::Error,
    },
    ErrorMessage {
        error: storage::Error,
    },
    OutOfBounds {
        offset: usize,
        len: usize,
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

    pub(crate) fn incoming(error: storage::Error) -> Self {
        Self::new(ErrorKind::Incoming { error })
    }

    pub(crate) fn outgoing(error: storage::Error) -> Self {
        Self::new(ErrorKind::Outgoing { error })
    }

    pub(crate) fn encode_broadcast_header(error: storage::Error) -> Self {
        Self::new(ErrorKind::EncodeBroadcastHeader { error })
    }

    pub(crate) fn encode_broadcast(error: storage::Error) -> Self {
        Self::new(ErrorKind::EncodeBroadcast { error })
    }

    pub(crate) fn encode_connect_header(error: storage::Error) -> Self {
        Self::new(ErrorKind::EncodeConnectHeader { error })
    }

    pub(crate) fn encode_error_message_header(error: storage::Error) -> Self {
        Self::new(ErrorKind::ErrorMessageHeader { error })
    }

    pub(crate) fn encode_error_message(error: storage::Error) -> Self {
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

/// The server side handle of the websocket protocol.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub struct Server<S, H, C = Channels>
where
    S: ServerImpl,
    H: Handler,
{
    handler: H,
    pinned: Pin<Box<Pinned<S::Socket>>>,
    channels: C,
    started: bool,
    closing: bool,
    pool: BufPool,
    outbound: VecDeque<Buf>,
    error: String,
    last_ping: Option<[u8; 4]>,
    rng: SmallRng,
    max_capacity: usize,
    out: VecDeque<S::Message>,
    socket_send: bool,
    socket_flush: bool,
    set: JoinSet<HandlerOutput<H>>,
}

impl<S, H> Server<S, H, Channels>
where
    S: ServerImpl,
    H: Handler,
{
    /// Construct a new server with the specified handler.
    #[inline]
    pub(crate) fn new(socket: S::Socket, handler: H) -> Self {
        let now = Instant::now();

        Self {
            handler,
            socket_flush: false,
            pinned: Box::pin(Pinned {
                socket,
                close_sleep: tokio::time::sleep_until(now + CLOSE_TIMEOUT),
                ping_sleep: tokio::time::sleep_until(now + PING_TIMEOUT),
            }),
            channels: Channels::default(),
            started: false,
            closing: false,
            pool: BufPool::default(),
            outbound: VecDeque::new(),
            error: String::new(),
            last_ping: None,
            rng: SmallRng::seed_from_u64(DEFAULT_SEED),
            max_capacity: MAX_CAPACITY,
            out: VecDeque::new(),
            socket_send: false,
            set: JoinSet::new(),
        }
    }
}

impl<S, H, C> Server<S, H, C>
where
    S: ServerImpl,
    H: Handler,
{
    /// Associated the specified seed with the server.
    ///
    /// This affects the random number generation used for ping messages.
    ///
    /// By default the seed is a constant value.
    #[inline]
    pub fn seed(mut self, seed: u64) -> Self {
        self.rng = SmallRng::seed_from_u64(seed);
        self
    }

    /// Associate the specified channel allocator with the server.
    #[inline]
    pub fn with_channel_allocator<U>(self, channels: U) -> Server<S, H, U>
    where
        U: ChannelAllocator,
    {
        Server {
            handler: self.handler,
            pinned: self.pinned,
            channels,
            started: self.started,
            closing: self.closing,
            pool: self.pool,
            outbound: self.outbound,
            error: self.error,
            last_ping: self.last_ping,
            rng: self.rng,
            max_capacity: self.max_capacity,
            out: self.out,
            socket_send: self.socket_send,
            socket_flush: self.socket_flush,
            set: self.set,
        }
    }

    /// Get a reference to the handler.
    #[inline]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Modify the maximum capacity of the buffer used for outgoing messages.
    ///
    /// This is used to prevent unbounded memory usage when writing large
    /// messages. If the buffer exceeds this capacity, it will be flushed
    /// immediately. Note that this is not a hard limit, and messages larger than
    /// this can still be written, but they will be flushed immediately.
    ///
    /// By default, the capacity is 1 MiB.
    #[inline]
    pub fn max_capacity(mut self, max_capacity: usize) -> Self {
        self.max_capacity = max_capacity;
        self
    }

    /// Modify the max allocated capacity of the outgoing buffer.
    ///
    /// Note that this capacity can be exceeded by writing large messages, but
    /// once messages have been flushed the allocation will be shrunk to the
    /// specified value.
    #[inline]
    pub fn with_max_capacity(mut self, max_capacity: usize) -> Self {
        self.max_capacity = max_capacity;
        self
    }
}

impl<S, H, C> Server<S, H, C>
where
    S: ServerImpl,
    Error: From<S::Error>,
    H: Handler,
    C: ChannelAllocator,
{
    /// Run the server.
    ///
    /// This must be called to handle buffered outgoing and incoming messages.
    pub async fn run(&mut self) -> Result<(), Error> {
        if !self.started {
            self.started = true;
            self.hello()?;
        }

        loop {
            if self.closing && self.out.is_empty() && self.outbound.is_empty() {
                break;
            }

            self.handle_send()?;

            let result = {
                let inner = Select::<S::Socket, H> {
                    pinned: self.pinned.as_mut(),
                    wants_socket_send: !self.socket_send,
                    wants_socket_flush: self.socket_flush,
                    set: &mut self.set,
                };

                inner.await
            };

            match result {
                Output::Close => {
                    self.out
                        .push_back(S::close(CLOSE_NORMAL, "connection timed out"));
                    self.closing = true;
                }
                Output::Ping => {
                    self.handle_ping()?;
                }
                Output::Recv(message) => {
                    let Some(message) = message else {
                        self.closing = true;
                        continue;
                    };

                    match message? {
                        Message::Text => {
                            self.out.push_back(S::close(
                                CLOSE_PROTOCOL_ERROR,
                                "Unsupported text message",
                            ));
                            self.closing = true;
                        }
                        Message::Binary(bytes) => {
                            self.handle_message(bytes).await?;
                        }
                        Message::Ping(payload) => {
                            self.out.push_back(S::pong(payload));
                        }
                        Message::Pong(data) => {
                            self.handle_pong(data)?;
                        }
                        Message::Close => {
                            self.closing = true;
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

        let buf = self.pool.with(|buf| {
            let mut writer = buf.writer();

            writer
                .write(ResponseHeader {
                    serial: 0,
                    broadcast: <T::Broadcast as Broadcast>::ID.get(),
                    error: 0,
                    channel,
                })
                .map_err(Error::encode_broadcast_header)?;

            writer.write(message).map_err(Error::encode_broadcast)?;
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
                .write(ResponseHeader {
                    serial: 0,
                    broadcast: MessageId::SERVER_HELLO.get(),
                    error: 0,
                    channel: ChannelId::NONE,
                })
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
        let mut reader = SliceReader::new(&bytes);

        let header: RequestHeader = match storage::decode(&mut reader) {
            Ok(header) => header,
            Err(error) => {
                tracing::debug!(?error, "Invalid request header");
                self.out
                    .push_back(S::close(CLOSE_PROTOCOL_ERROR, "Invalid request header"));
                self.closing = true;
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

                    let result = (|| {
                        let mut writer = buf.writer();

                        let result = writer.write(ResponseHeader {
                            serial: header.serial,
                            broadcast: 0,
                            error: 0,
                            channel,
                        });

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
                _ => {
                    let id = <H::Id as Id>::from_id(id);
                    let offset = bytes.len() - reader.remaining();
                    self.handle_request(bytes, offset, header, id);
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
        let buf = self.pool.with(|buf| {
            // Reset the buffer to the previous start point.
            let mut writer = buf.writer();

            let result = writer.write(ResponseHeader {
                serial: header.serial,
                broadcast: 0,
                error: MessageId::ERROR_MESSAGE.get(),
                channel: header.channel,
            });

            result.map_err(Error::encode_error_message_header)?;

            let result = writer.write(ErrorMessage {
                message: &self.error,
            });

            result.map_err(Error::encode_error_message)?;
            writer.flush();
            Ok::<_, Error>(())
        })?;

        self.outbound.push_back(buf);
        Ok(())
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

            socket.as_mut().start_send(S::binary(frame))?;

            self.socket_flush = true;
            self.socket_send = false;
            break;
        }

        Ok(())
    }

    fn handle_request(&mut self, bytes: Bytes, offset: usize, header: RequestHeader, id: H::Id) {
        tracing::debug!(header.serial, ?id, "Got request");

        let mut buf = self.pool.get();
        let handler = self.handler.clone();

        self.set.spawn(async move {
            let Some(bytes) = bytes.get(offset..) else {
                let kind = ErrorKind::OutOfBounds {
                    offset,
                    len: bytes.len(),
                };

                return (Err(Error::new(kind)), header, buf);
            };

            let reader = SliceReader::new(bytes);

            let mut incoming = Incoming {
                error: None,
                reader,
                channel: header.channel,
            };

            let mut outgoing = Outgoing {
                serial: Some(header.serial),
                error: None,
                buf: &mut buf,
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
        let wants_socket_send;
        let wants_socket_flush;
        let set;

        // SAFETY: This type is not Unpin.
        unsafe {
            let this = Pin::get_unchecked_mut(self);
            (close, ping, socket) = this.pinned.as_mut().project();
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

        if let Poll::Ready(output) = socket.as_mut().poll_next(cx) {
            return Poll::Ready(Output::Recv(output));
        }

        if wants_socket_send && let Poll::Ready(result) = socket.as_mut().poll_ready(cx) {
            return Poll::Ready(Output::Send(result));
        }

        if wants_socket_flush && let Poll::Ready(result) = socket.as_mut().poll_flush(cx) {
            return Poll::Ready(Output::Flushed(result));
        }

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
    error: Option<storage::Error>,
    reader: SliceReader<'de>,
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
        T: Decode<'de, Binary, Global>,
    {
        match storage::decode(&mut self.reader) {
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
    error: Option<storage::Error>,
    buf: &'a mut Buf,
    channel: ChannelId,
}

impl Outgoing<'_> {
    /// Write a response.
    ///
    /// This can only be called once. Calling this multiple times has no effect.
    ///
    /// See [`server()`] for how to use with `axum`.
    ///
    /// [`server()`]: crate::axum08::server
    pub fn write<T>(&mut self, value: T)
    where
        T: Encode<Binary>,
    {
        let Some(serial) = self.serial.take() else {
            return;
        };

        let mut writer = self.buf.writer();

        let result = writer.write(ResponseHeader {
            serial,
            broadcast: 0,
            error: 0,
            channel: self.channel,
        });

        if let Err(error) = result {
            self.error = Some(error);
            return;
        }

        if let Err(error) = writer.write(value) {
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
