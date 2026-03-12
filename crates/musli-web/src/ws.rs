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
//! struct MyHandler;
//!
//! impl ws::Handler for MyHandler {
//!     type Id = api::Request;
//!     type Response = Option<()>;
//!
//!     async fn handle<I, O>(
//!         &mut self,
//!         id: Self::Id,
//!         incoming: &mut I,
//!         outgoing: &mut O,
//!     ) -> Self::Response
//!     where
//!         I: for<'de> ws::Incoming<'de>,
//!         O: ws::Outgoing,
//!     {
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
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

use bytes::Bytes;
use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::{Decode, Encode};
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::time::{Duration, Instant, Sleep};

use crate::api::{Broadcast, ErrorMessage, Event, Id, MessageId, RequestHeader, ResponseHeader};
use crate::buf::InvalidFrame;
use crate::{Buf, Framework, Storage};

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
enum ErrorKind<E> {
    #[cfg(feature = "axum-core05")]
    AxumCore05 {
        error: axum_core05::Error,
    },
    FormatError,
    InvalidFrame {
        error: InvalidFrame,
    },
    Incoming {
        error: E,
    },
    Outgoing {
        error: E,
    },
    EncodeBroadcastHeader {
        error: E,
    },
    EncodeBroadcast {
        error: E,
    },
    ErrorMessageHeader {
        error: E,
    },
    ErrorMessage {
        error: E,
    },
}

/// The error produced by the server side of the websocket protocol
pub struct Error<F>
where
    F: Framework,
{
    kind: ErrorKind<F::Error>,
}

impl<F> Error<F>
where
    F: Framework,
{
    #[inline]
    const fn new(kind: ErrorKind<F::Error>) -> Self {
        Self { kind }
    }

    pub(crate) fn incoming(error: F::Error) -> Self {
        Self::new(ErrorKind::Incoming { error })
    }

    pub(crate) fn outgoing(error: F::Error) -> Self {
        Self::new(ErrorKind::Outgoing { error })
    }

    pub(crate) fn encode_broadcast_header(error: F::Error) -> Self {
        Self::new(ErrorKind::EncodeBroadcastHeader { error })
    }

    pub(crate) fn encode_broadcast(error: F::Error) -> Self {
        Self::new(ErrorKind::EncodeBroadcast { error })
    }

    pub(crate) fn encode_error_message_header(error: F::Error) -> Self {
        Self::new(ErrorKind::ErrorMessageHeader { error })
    }

    pub(crate) fn encode_error_message(error: F::Error) -> Self {
        Self::new(ErrorKind::ErrorMessage { error })
    }
}

impl<F> fmt::Display for Error<F>
where
    F: Framework,
{
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
            ErrorKind::ErrorMessageHeader { .. } => {
                write!(f, "Encoding error when encoding error message header")
            }
            ErrorKind::ErrorMessage { .. } => {
                write!(f, "Encoding error when encoding error message")
            }
        }
    }
}

impl<F> fmt::Debug for Error<F>
where
    F: Framework,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl<F> core::error::Error for Error<F>
where
    F: Framework,
{
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.kind {
            #[cfg(feature = "axum-core05")]
            ErrorKind::AxumCore05 { error } => Some(error),
            ErrorKind::Incoming { error } => Some(error),
            ErrorKind::Outgoing { error } => Some(error),
            ErrorKind::EncodeBroadcastHeader { error } => Some(error),
            ErrorKind::EncodeBroadcast { error } => Some(error),
            ErrorKind::ErrorMessageHeader { error } => Some(error),
            ErrorKind::ErrorMessage { error } => Some(error),
            _ => None,
        }
    }
}

#[cfg(feature = "axum-core05")]
impl<F> From<axum_core05::Error> for Error<F>
where
    F: Framework,
{
    #[inline]
    fn from(error: axum_core05::Error) -> Self {
        Self::new(ErrorKind::AxumCore05 { error })
    }
}

impl<F> From<ErrorKind<F::Error>> for Error<F>
where
    F: Framework,
{
    #[inline]
    fn from(kind: ErrorKind<F::Error>) -> Self {
        Self::new(kind)
    }
}

impl<F> From<InvalidFrame> for Error<F>
where
    F: Framework,
{
    #[inline]
    fn from(error: InvalidFrame) -> Self {
        Self::new(ErrorKind::InvalidFrame { error })
    }
}

/// The response meta from handling a request.
pub struct Response {
    handled: bool,
}

/// Trait governing how something can be converted into a response.
pub trait IntoResponse {
    /// The error variant being produced.
    type Error;

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
    E: fmt::Display,
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
pub trait Handler {
    /// The type of message id used.
    type Id: Id;
    /// The response type returned by the handler.
    type Response: IntoResponse;

    /// Handle a request.
    fn handle<'this, I, O>(
        &'this mut self,
        id: Self::Id,
        incoming: &'this mut I,
        outgoing: &'this mut O,
    ) -> impl Future<Output = Self::Response> + Send + 'this
    where
        I: ?Sized + Incoming<'this>,
        O: ?Sized + Outgoing;
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

/// The server side handle of the websocket protocol.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub struct Server<S, H, F = Storage>
where
    S: ServerImpl,
    F: Framework,
{
    closing: bool,
    outbound: Buf<F>,
    error: String,
    handler: H,
    last_ping: Option<[u8; 4]>,
    rng: SmallRng,
    max_capacity: usize,
    out: VecDeque<S::Message>,
    socket_send: bool,
    socket_flush: bool,
    pinned: Pin<Box<Pinned<S::Socket>>>,
}

impl<S, H, F> Server<S, H, F>
where
    S: ServerImpl,
    F: Framework,
{
    /// Construct a new server with the specified handler.
    #[inline]
    pub(crate) fn new(socket: S::Socket, handler: H) -> Self {
        let now = Instant::now();

        Self {
            closing: false,
            outbound: Buf::new(),
            error: String::new(),
            handler,
            last_ping: None,
            rng: SmallRng::seed_from_u64(DEFAULT_SEED),
            max_capacity: MAX_CAPACITY,
            out: VecDeque::new(),
            socket_send: false,
            socket_flush: false,
            pinned: Box::pin(Pinned {
                socket,
                close_sleep: tokio::time::sleep_until(now + CLOSE_TIMEOUT),
                ping_sleep: tokio::time::sleep_until(now + PING_TIMEOUT),
            }),
        }
    }

    /// Access a reference to the handler.
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Access a mutable reference to the handler.
    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
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

impl<S, H, F> Server<S, H, F>
where
    S: ServerImpl,
    F: Framework,
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
}

impl<S, H, F> Server<S, H, F>
where
    S: ServerImpl,
    Error<F>: From<S::Error>,
    H: Handler<Response: IntoResponse<Error: fmt::Display>>,
    F: Framework,
{
    /// Run the server.
    ///
    /// This must be called to handle buffered outgoing and incoming messages.
    pub async fn run(&mut self) -> Result<(), Error<F>> {
        loop {
            if self.closing && self.out.is_empty() && self.outbound.is_empty() {
                break;
            }

            self.handle_send()?;

            let result = {
                let inner = Select {
                    pinned: self.pinned.as_mut(),
                    wants_socket_send: !self.socket_send,
                    wants_socket_flush: self.socket_flush,
                };

                inner.await
            };

            tracing::debug!(?result);

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
            }
        }

        Ok(())
    }

    /// Write a broadcast message.
    ///
    /// Note that the written message is buffered, and will be sent when
    /// [`Server::run`] is called.
    pub fn broadcast<T>(&mut self, message: T) -> Result<(), Error<F>>
    where
        T: Event,
    {
        tracing::debug!(id = ?<T::Broadcast as Broadcast>::ID, "Broadcast");

        let mut writer = self.outbound.writer();

        writer
            .write(ResponseHeader {
                serial: 0,
                broadcast: <T::Broadcast as Broadcast>::ID.get(),
                error: 0,
            })
            .map_err(Error::encode_broadcast_header)?;

        writer.write(message).map_err(Error::encode_broadcast)?;
        writer.flush();
        Ok(())
    }

    fn format_error_message(&mut self, error: impl fmt::Display) -> Result<(), Error<F>> {
        self.error.clear();

        if write!(self.error, "{error}").is_err() {
            self.error.clear();
            return Err(Error::new(ErrorKind::FormatError));
        }

        Ok(())
    }

    fn format_error(&mut self, error: impl core::error::Error) -> Result<(), Error<F>> {
        self.error.clear();

        if write!(self.error, "{error:#}").is_err() {
            self.error.clear();
            return Err(Error::new(ErrorKind::FormatError));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, bytes))]
    async fn handle_message(&mut self, bytes: Bytes) -> Result<(), Error<F>> {
        let mut reader = SliceReader::new(&bytes);

        let header: RequestHeader = match F::decode(&mut reader) {
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

            let id = <H::Id as Id>::from_id(id);

            let res = match self.handle_request(reader, header.serial, id).await {
                Ok(res) => res,
                Err(error) => {
                    self.format_error(error)?;
                    break 'err true;
                }
            };

            let res = match res.into_response() {
                Ok(res) => res,
                Err(error) => {
                    self.format_error_message(format_args!("Error in handler: {error:#}"))?;
                    break 'err true;
                }
            };

            if !res.handled {
                self.format_error_message(format_args!("No support for request {}", header.id))?;
                break 'err true;
            }

            false
        };

        if err {
            // Reset the buffer to the previous start point.
            let mut writer = self.outbound.writer();

            let result = writer.write(ResponseHeader {
                serial: header.serial,
                broadcast: 0,
                error: MessageId::ERROR_MESSAGE.get(),
            });

            result.map_err(Error::encode_error_message_header)?;

            let result = writer.write(ErrorMessage {
                message: &self.error,
            });

            result.map_err(Error::encode_error_message)?;
            writer.flush();
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_ping(&mut self) -> Result<(), Error<F>> {
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
    fn handle_pong(&mut self, payload: Bytes) -> Result<(), Error<F>> {
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
    fn handle_send(&mut self) -> Result<(), Error<F>> {
        let (_, _, mut socket) = self.pinned.as_mut().project();

        if self.socket_send
            && let Some(message) = self.out.pop_front()
        {
            socket.as_mut().start_send(message)?;
            self.socket_flush = true;
            self.socket_send = false;
        }

        if self.socket_send
            && let Some(frame) = self.outbound.read()?
        {
            socket.as_mut().start_send(S::binary(frame))?;

            if self.outbound.is_empty() {
                self.outbound.clear();
            }

            self.socket_flush = true;
            self.socket_send = false;
        }

        Ok(())
    }

    async fn handle_request(
        &mut self,
        reader: SliceReader<'_>,
        serial: u32,
        id: H::Id,
    ) -> Result<H::Response, Error<F>> {
        tracing::debug!(serial, ?id, "Got request");

        let mut incoming = IncomingImpl {
            error: None,
            reader,
        };

        let mut outgoing = OutgoingImpl {
            serial: Some(serial),
            error: None,
            buf: &mut self.outbound,
        };

        let response = self.handler.handle(id, &mut incoming, &mut outgoing).await;

        if let Some(error) = incoming.error.take() {
            return Err(Error::incoming(error));
        }

        if let Some(error) = outgoing.error.take() {
            return Err(Error::outgoing(error));
        }

        Ok(response)
    }
}

#[derive(Debug)]
enum Output<E> {
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
}

struct Select<'a, S> {
    pinned: Pin<&'a mut Pinned<S>>,
    wants_socket_send: bool,
    wants_socket_flush: bool,
}

impl<S> Future for Select<'_, S>
where
    S: SocketImpl,
{
    type Output = Output<S::Error>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let close;
        let ping;
        let mut socket;
        let wants_socket_send;
        let wants_socket_flush;

        // SAFETY: This type is not Unpin.
        unsafe {
            let this = Pin::get_unchecked_mut(self);
            (close, ping, socket) = this.pinned.as_mut().project();
            wants_socket_send = this.wants_socket_send;
            wants_socket_flush = this.wants_socket_flush;
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

        Poll::Pending
    }
}

mod sealed_incoming {
    use super::{Framework, IncomingImpl};

    pub trait Sealed {}

    impl<'de, F> Sealed for IncomingImpl<'de, F> where F: Framework {}
}

/// The buffer for incoming requests.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub trait Incoming<'de>
where
    Self: self::sealed_incoming::Sealed,
{
    /// Read a request and return `Some(T)` if the request was successfully
    /// decoded.
    ///
    /// Note that any failure to decode will be propagated as an error
    /// automatically, the user does not have to deal with it themselves.
    /// Instead, failure to decode should be treated as if the request was
    /// unhandled by returning for example `false` or `Option::None`.
    fn read<T>(&mut self) -> Option<T>
    where
        T: Decode<'de, Binary, Global>;
}

pub(crate) struct IncomingImpl<'de, F>
where
    F: Framework,
{
    error: Option<F::Error>,
    reader: SliceReader<'de>,
}

impl<'de, F> Incoming<'de> for IncomingImpl<'de, F>
where
    F: Framework,
{
    #[inline]
    fn read<T>(&mut self) -> Option<T>
    where
        T: Decode<'de, Binary, Global>,
    {
        match F::decode(&mut self.reader) {
            Ok(value) => Some(value),
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }
}

mod sealed_outgoing {
    use super::{Framework, OutgoingImpl};

    pub trait Sealed {}

    impl<'a, F> Sealed for OutgoingImpl<'a, F> where F: Framework {}
}

/// The buffer for outgoing responses.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub trait Outgoing
where
    Self: self::sealed_outgoing::Sealed,
{
    /// Write a response.
    ///
    /// This can only be called once. Calling this multiple times has no effect.
    ///
    /// See [`server()`] for how to use with `axum`.
    ///
    /// [`server()`]: crate::axum08::server
    fn write(&mut self, value: impl Encode<Binary>);
}

pub(crate) struct OutgoingImpl<'a, F>
where
    F: Framework,
{
    serial: Option<u32>,
    error: Option<F::Error>,
    buf: &'a mut Buf<F>,
}

impl<F> Outgoing for OutgoingImpl<'_, F>
where
    F: Framework,
{
    fn write(&mut self, value: impl Encode<Binary>) {
        let Some(serial) = self.serial.take() else {
            return;
        };

        let mut writer = self.buf.writer();

        let result = writer.write(ResponseHeader {
            serial,
            broadcast: 0,
            error: 0,
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
