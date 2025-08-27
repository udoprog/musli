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
//! use musli::{Decode, Encode};
//! use musli_web::ws;
//!
//! #[derive(Decode)]
//! struct Request<'de> {
//!     message: &'de str,
//! }
//!
//! #[derive(Encode)]
//! struct Response<'de> {
//!     message: &'de str,
//! }
//!
//! struct MyHandler;
//!
//! impl ws::Handler for MyHandler {
//!     type Response = Result<bool, &'static str>;
//!
//!     async fn handle(
//!         &mut self,
//!         kind: &str,
//!         incoming: &mut ws::Incoming<'_>,
//!         outgoing: &mut ws::Outgoing<'_>,
//!     ) -> Self::Response {
//!         tracing::info!("Handling: {kind}");
//!
//!         match kind {
//!             "request" => {
//!                 let Some(request) = incoming.read::<Request<'_>>() else {
//!                     return Ok(false);
//!                 };
//!
//!                 if !request.message.contains("hello") {
//!                     return Err("Rude message");
//!                 }
//!
//!                 outgoing.write(Response {
//!                     message: request.message,
//!                 });
//!
//!                 Ok(true)
//!             }
//!             _ => Ok(false),
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
use core::ops::Range;
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::string::String;
use alloc::vec::Vec;

use bytes::Bytes;
use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::storage;
use musli::{Decode, Encode};
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::time::{Duration, Instant, Sleep};

use crate::Buf;
use crate::api;

const MAX_CAPACITY: usize = 1048576;
const CLOSE_NORMAL: u16 = 1000;
const CLOSE_PROTOCOL_ERROR: u16 = 1002;
const CLOSE_TIMEOUT: Duration = Duration::from_secs(30);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_SEED: u64 = 0xdeadbeef;

mod sealed {
    pub trait Sealed {}

    #[cfg(feature = "axum08")]
    impl Sealed for crate::axum08::AxumServer {}
    #[cfg(feature = "axum08")]
    impl Sealed for axum08::extract::ws::WebSocket {}
}

/// A websocket message.
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

#[allow(async_fn_in_trait)]
pub(crate) trait Socket
where
    Self: self::sealed::Sealed,
{
    #[doc(hidden)]
    type Message;

    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Result<Message, Self::Error>>>;

    #[doc(hidden)]
    async fn send(&mut self, message: Self::Message) -> Result<(), Self::Error>;
}

/// The details of how a [`Server`] is implemented.
///
/// See [`AxumServer`] for an example.
///
/// [`AxumServer`]: crate::axum08::AxumServer
pub trait ServerImplementation
where
    Self: self::sealed::Sealed,
{
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    type Message;

    #[doc(hidden)]
    #[allow(private_bounds)]
    type Socket: Socket<Message = Self::Message, Error = Self::Error>;

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
    Musli {
        error: storage::Error,
    },
    FormatError,
    InvalidBufferFrame {
        frame: Range<usize>,
        size: usize,
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
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            #[cfg(feature = "axum-core05")]
            ErrorKind::AxumCore05 { .. } => write!(f, "Error in axum-core"),
            ErrorKind::Musli { .. } => write!(f, "Error in musli"),
            ErrorKind::FormatError => write!(f, "Error formatting error response"),
            ErrorKind::InvalidBufferFrame {
                frame: Range { start, end },
                size,
            } => write!(f, "Invalid buffer frame {start}..{end} is not in 0..{size}"),
        }
    }
}

impl core::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.kind {
            #[cfg(feature = "axum-core05")]
            ErrorKind::AxumCore05 { error } => Some(error),
            ErrorKind::Musli { error } => Some(error),
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

impl From<storage::Error> for Error {
    #[inline]
    fn from(error: storage::Error) -> Self {
        Self::new(ErrorKind::Musli { error })
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

/// The response meta from handling a request.
pub struct Response {
    handled: bool,
}

/// Trait governing how something can be converted into a response.
pub trait IntoResponse {
    /// The error variant being produced.
    type Error: fmt::Display;

    /// Convert self into a response.
    fn into_response(self) -> Result<Response, Self::Error>;
}

/// Implement [`IntoResponse`] for bool.
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

/// Implement [`IntoResponse`] for result types.
///
/// Note that this allows anything that implements [`fmt::Display`] to be used
/// as an [`Err`] variant. The exact message it's being formatted into will be
/// forwarded as an error to the client.
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

/// A handler for incoming requests.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub trait Handler {
    /// The response type returned by the handler.
    type Response: IntoResponse;

    /// Handle a request.
    fn handle<'this>(
        &'this mut self,
        kind: &'this str,
        incoming: &'this mut Incoming<'_>,
        outgoing: &'this mut Outgoing<'_>,
    ) -> impl Future<Output = Self::Response> + Send + 'this;
}

/// The server side handle of the websocket protocol.
///
/// See [`server()`] for how to use with `axum`.
///
/// [`server()`]: crate::axum08::server
pub struct Server<S, H>
where
    S: ServerImplementation,
{
    buf: Buf,
    error: String,
    socket: S::Socket,
    handler: H,
    last_ping: Option<u32>,
    rng: SmallRng,
    close_sleep: Sleep,
    ping_sleep: Sleep,
    max_capacity: usize,
}

impl<S, H> Server<S, H>
where
    S: ServerImplementation,
{
    /// Construct a new server with the specified handler.
    #[inline]
    pub(crate) fn new(socket: S::Socket, handler: H) -> Self {
        let now = Instant::now();

        Self {
            buf: Buf::default(),
            error: String::new(),
            socket,
            handler,
            last_ping: None::<u32>,
            rng: SmallRng::seed_from_u64(DEFAULT_SEED),
            close_sleep: tokio::time::sleep_until(now + CLOSE_TIMEOUT),
            ping_sleep: tokio::time::sleep_until(now + PING_TIMEOUT),
            max_capacity: MAX_CAPACITY,
        }
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

impl<S, H> Server<S, H>
where
    S: ServerImplementation,
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

    fn project(self: Pin<&mut Self>) -> (Pin<&mut Sleep>, Pin<&mut Sleep>, Pin<&mut S::Socket>) {
        // SAFETY: Deal with running the server.
        unsafe {
            let this = Pin::get_unchecked_mut(self);
            let close = Pin::new_unchecked(&mut this.close_sleep);
            let ping = Pin::new_unchecked(&mut this.ping_sleep);
            let socket = Pin::new_unchecked(&mut this.socket);
            (close, ping, socket)
        }
    }
}

impl<S, H> Server<S, H>
where
    S: ServerImplementation,
    Error: From<S::Error>,
    H: Handler,
{
    fn format_error(self: Pin<&mut Self>, error: impl fmt::Display) -> Result<(), Error> {
        let this = unsafe { Pin::get_unchecked_mut(self) };
        this.error.clear();

        if write!(this.error, "{error}").is_err() {
            this.error.clear();
            return Err(Error::new(ErrorKind::FormatError));
        }

        Ok(())
    }

    /// Run the server.
    ///
    /// This must be called to handle buffered outgoing and incoming messages.
    pub async fn run(mut self: Pin<&mut Self>) -> Result<(), Error> {
        let close_here = loop {
            if !self.buf.is_empty() {
                self.as_mut().flush().await?;
            }

            let result = {
                let (close, ping, socket) = self.as_mut().project();

                let inner = Inner {
                    close,
                    ping,
                    socket,
                };

                inner.await
            };

            match result {
                InnerOutput::Close => {
                    break Some((CLOSE_NORMAL, "connection timed out"));
                }
                InnerOutput::Ping => {
                    // SAFETY: Deal with running the server.
                    let this = unsafe { Pin::get_unchecked_mut(self.as_mut()) };
                    let mut ping = unsafe { Pin::new_unchecked(&mut this.ping_sleep) };

                    let payload = this.rng.random::<u32>();
                    this.last_ping = Some(payload);
                    let data = payload.to_ne_bytes().into_iter().collect::<Vec<_>>();
                    tracing::trace!(data = ?&data[..], "Sending ping");
                    this.socket.send(S::ping(data.to_vec().into())).await?;
                    let now = Instant::now();
                    ping.as_mut().reset(now + PING_TIMEOUT);
                }
                InnerOutput::Output(message) => {
                    let Some(message) = message else {
                        break None;
                    };

                    match message? {
                        Message::Text => break Some((CLOSE_PROTOCOL_ERROR, "unsupported message")),
                        Message::Binary(bytes) => {
                            let mut reader = SliceReader::new(&bytes);

                            let header = match storage::decode(&mut reader) {
                                Ok(header) => header,
                                Err(error) => {
                                    tracing::warn!(?error, "Failed to decode request header");
                                    break Some((CLOSE_PROTOCOL_ERROR, "invalid request"));
                                }
                            };

                            let errored = match self.as_mut().handle_request(reader, header).await {
                                Ok(response) => match response.into_response() {
                                    Err(error) => {
                                        self.as_mut().format_error(error)?;
                                        false
                                    }
                                    Ok(response) => {
                                        if response.handled {
                                            true
                                        } else {
                                            self.as_mut().format_error(format_args!(
                                                "No support for request {}",
                                                header.kind
                                            ))?;
                                            false
                                        }
                                    }
                                },
                                Err(error) => {
                                    self.as_mut().format_error(error)?;
                                    false
                                }
                            };

                            if errored {
                                let this = unsafe { Pin::get_unchecked_mut(self.as_mut()) };

                                // Reset the buffer to the previous start point.
                                this.buf.reset();

                                this.buf.write(api::ResponseHeader {
                                    index: header.index,
                                    serial: header.serial,
                                    broadcast: None,
                                    error: Some(this.error.as_str()),
                                })?;

                                this.buf.done();
                            }
                        }
                        Message::Ping(payload) => {
                            // SAFETY: Deal with running the server.
                            let this = unsafe { Pin::get_unchecked_mut(self.as_mut()) };
                            this.socket.send(S::pong(payload)).await?;
                            continue;
                        }
                        Message::Pong(data) => {
                            // SAFETY: Deal with running the server.
                            let this = unsafe { Pin::get_unchecked_mut(self.as_mut()) };
                            let close = unsafe { Pin::new_unchecked(&mut this.close_sleep) };
                            let ping = unsafe { Pin::new_unchecked(&mut this.ping_sleep) };

                            tracing::trace!(data = ?&data[..], "Pong");

                            let Some(expected) = this.last_ping else {
                                continue;
                            };

                            if expected.to_ne_bytes()[..] != data[..] {
                                continue;
                            }

                            let now = Instant::now();

                            close.reset(now + CLOSE_TIMEOUT);
                            ping.reset(now + PING_TIMEOUT);
                            this.last_ping = None;
                        }
                        Message::Close => break None,
                    }
                }
            }
        };

        if let Some((code, reason)) = close_here {
            tracing::trace!(code, reason, "Closing websocket with reason");
            let this = unsafe { Pin::get_unchecked_mut(self) };
            this.socket.send(S::close(code, reason)).await?;
        } else {
            tracing::trace!("Closing websocket");
        };

        Ok(())
    }

    /// Write a broadcast message.
    ///
    /// Note that the written message is buffered, and will be sent when
    /// [`Server::run`] is called.
    pub fn broadcast<'de, T>(self: Pin<&mut Self>, message: T) -> Result<(), Error>
    where
        T: api::Broadcast<'de>,
    {
        let this = unsafe { Pin::get_unchecked_mut(self) };

        this.buf.write(api::ResponseHeader {
            index: 0,
            serial: 0,
            broadcast: Some(<T::Endpoint as api::Listener>::KIND),
            error: None,
        })?;

        this.buf.write(message)?;
        this.buf.done();
        Ok(())
    }

    /// Flush outgoing messages that have been written to the outgoing buffer.
    ///
    /// This will block until the messages have been sent over the network.
    pub async fn flush(self: Pin<&mut Self>) -> Result<()> {
        tracing::trace!("flushing outbound buffer");

        let this = unsafe { Pin::get_unchecked_mut(self) };

        for frame in this.buf.frames() {
            let frame =
                frame.map_err(|(frame, size)| ErrorKind::InvalidBufferFrame { frame, size })?;

            this.socket.send(S::binary(frame)).await?;
        }

        this.error.clear();
        this.buf.clear();
        this.buf.shrink_to(MAX_CAPACITY);
        Ok(())
    }

    async fn handle_request(
        self: Pin<&mut Self>,
        reader: SliceReader<'_>,
        header: api::RequestHeader<'_>,
    ) -> Result<H::Response, storage::Error> {
        tracing::trace!("Got request: {header:?}");

        // SAFETY: The server is pinned.
        let this = unsafe { Pin::get_unchecked_mut(self) };

        this.buf.write(api::ResponseHeader {
            index: header.index,
            serial: header.serial,
            broadcast: None,
            error: None,
        })?;

        let mut incoming = Incoming {
            error: None,
            reader,
        };

        let mut outgoing = Outgoing {
            error: None,
            buf: &mut this.buf,
        };

        let response = this
            .handler
            .handle(header.kind, &mut incoming, &mut outgoing)
            .await;

        if let Some(error) = incoming.error.take() {
            return Err(error);
        }

        if let Some(error) = outgoing.error.take() {
            return Err(error);
        }

        outgoing.buf.done();
        Ok(response)
    }
}

enum InnerOutput<E> {
    /// The connection should be closed.
    Close,
    /// A ping message was received.
    Ping,
    /// A message was received.
    Output(Option<Result<Message, E>>),
}

struct Inner<'a, C, P, S> {
    close: Pin<&'a mut C>,
    ping: Pin<&'a mut P>,
    socket: Pin<&'a mut S>,
}

impl<C, P, S> Future for Inner<'_, C, P, S>
where
    C: Future,
    P: Future,
    S: Socket,
{
    type Output = InnerOutput<S::Error>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: This type is not Unpin.
        let (close, ping, socket) = unsafe {
            let this = Pin::get_unchecked_mut(self);
            let close = this.close.as_mut();
            let ping = this.ping.as_mut();
            let socket = this.socket.as_mut();
            (close, ping, socket)
        };

        if close.poll(cx).is_ready() {
            cx.waker().wake_by_ref();
            return Poll::Ready(InnerOutput::Close);
        }

        if ping.poll(cx).is_ready() {
            cx.waker().wake_by_ref();
            return Poll::Ready(InnerOutput::Ping);
        }

        if let Poll::Ready(output) = socket.poll_next(cx) {
            cx.waker().wake_by_ref();
            return Poll::Ready(InnerOutput::Output(output));
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
}

impl<'de> Incoming<'de> {
    /// Read a request.
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
    error: Option<storage::Error>,
    buf: &'a mut Buf,
}

impl Outgoing<'_> {
    /// Write a response.
    ///
    /// See [`server()`] for how to use with `axum`.
    ///
    /// [`server()`]: crate::axum08::server
    pub fn write<T>(&mut self, value: T)
    where
        T: Encode<Binary>,
    {
        if let Err(error) = self.buf.write(value) {
            self.error = Some(error);
        }
    }
}
