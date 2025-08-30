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
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::collections::VecDeque;
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
use crate::api::{Broadcast, Listener, RequestHeader, ResponseHeader};
use crate::buf::InvalidFrame;

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
    Musli {
        error: storage::Error,
    },
    FormatError,
    InvalidFrame {
        error: InvalidFrame,
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
            ErrorKind::InvalidFrame { error } => error.fmt(f),
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
pub trait IntoResponse {
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
    S: ServerImpl,
{
    closing: bool,
    buf: Buf,
    error: String,
    socket: S::Socket,
    handler: H,
    last_ping: Option<u32>,
    rng: SmallRng,
    close_sleep: Sleep,
    ping_sleep: Sleep,
    max_capacity: usize,
    out: VecDeque<S::Message>,
    socket_send: bool,
    socket_flush: bool,
}

impl<S, H> Server<S, H>
where
    S: ServerImpl,
{
    /// Construct a new server with the specified handler.
    #[inline]
    pub(crate) fn new(socket: S::Socket, handler: H) -> Self {
        let now = Instant::now();

        Self {
            closing: false,
            buf: Buf::default(),
            error: String::new(),
            socket,
            handler,
            last_ping: None::<u32>,
            rng: SmallRng::seed_from_u64(DEFAULT_SEED),
            close_sleep: tokio::time::sleep_until(now + CLOSE_TIMEOUT),
            ping_sleep: tokio::time::sleep_until(now + PING_TIMEOUT),
            max_capacity: MAX_CAPACITY,
            out: VecDeque::new(),
            socket_send: false,
            socket_flush: false,
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
    S: ServerImpl,
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

impl<S, H> Server<S, H>
where
    S: ServerImpl,
    Error: From<S::Error>,
    H: Handler,
{
    fn format_err(&mut self, error: impl fmt::Display) -> Result<(), Error> {
        self.error.clear();

        if write!(self.error, "{error}").is_err() {
            self.error.clear();
            return Err(Error::new(ErrorKind::FormatError));
        }

        Ok(())
    }

    /// Run the server.
    ///
    /// This must be called to handle buffered outgoing and incoming messages.
    pub async fn run(self: Pin<&mut Self>) -> Result<(), Error> {
        // SAFETY: This method doesn't violate pin.
        let this = unsafe { Pin::get_unchecked_mut(self) };

        loop {
            if this.closing && this.out.is_empty() && this.buf.is_empty() {
                break;
            }

            this.handle_send()?;

            let result = {
                let inner = Select {
                    close: &mut this.close_sleep,
                    ping: &mut this.ping_sleep,
                    socket: &mut this.socket,
                    wants_socket_send: !this.socket_send,
                    wants_socket_flush: this.socket_flush,
                };

                inner.await
            };

            tracing::debug!(?result);

            match result {
                Output::Close => {
                    this.out
                        .push_back(S::close(CLOSE_NORMAL, "connection timed out"));
                    this.closing = true;
                }
                Output::Ping => {
                    this.handle_ping()?;
                }
                Output::Recv(message) => {
                    let Some(message) = message else {
                        this.closing = true;
                        continue;
                    };

                    match message? {
                        Message::Text => {
                            this.out.push_back(S::close(
                                CLOSE_PROTOCOL_ERROR,
                                "Unsupported text message",
                            ));
                            this.closing = true;
                        }
                        Message::Binary(bytes) => {
                            this.handle_message(bytes).await?;
                        }
                        Message::Ping(payload) => {
                            this.out.push_back(S::pong(payload));
                        }
                        Message::Pong(data) => {
                            this.handle_pong(data)?;
                        }
                        Message::Close => {
                            this.closing = true;
                        }
                    }
                }
                Output::Send(result) => {
                    if let Err(err) = result {
                        return Err(Error::from(err));
                    };

                    this.socket_send = true;
                }
                Output::Flushed(result) => {
                    if let Err(err) = result {
                        return Err(Error::from(err));
                    };

                    this.socket_flush = false;
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, bytes))]
    async fn handle_message(&mut self, bytes: Bytes) -> Result<(), Error> {
        let mut reader = SliceReader::new(&bytes);

        let header = match storage::decode(&mut reader) {
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
            let res = match self.handle_request(reader, header).await {
                Ok(res) => res,
                Err(err) => {
                    self.format_err(err)?;
                    break 'err true;
                }
            };

            let res = match res.into_response() {
                Ok(res) => res,
                Err(err) => {
                    self.format_err(err)?;
                    break 'err true;
                }
            };

            if !res.handled {
                self.format_err(format_args!("No support for request `{}`", header.kind))?;
                break 'err true;
            }

            false
        };

        if err {
            // Reset the buffer to the previous start point.
            self.buf.reset();

            self.buf.write(ResponseHeader {
                serial: header.serial,
                broadcast: None,
                error: Some(self.error.as_str()),
            })?;
        }

        self.buf.done();
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_ping(&mut self) -> Result<(), Error> {
        let mut ping_sleep = unsafe { Pin::new_unchecked(&mut self.ping_sleep) };

        let payload = self.rng.random::<u32>();
        self.last_ping = Some(payload);
        let data = payload.to_ne_bytes().into_iter().collect::<Vec<_>>();

        tracing::debug!(data = ?&data[..], "Sending ping");
        self.out.push_back(S::ping(data.into()));

        let now = Instant::now();
        ping_sleep.as_mut().reset(now + PING_TIMEOUT);

        Ok(())
    }

    #[tracing::instrument(skip(self, data))]
    fn handle_pong(&mut self, data: Bytes) -> Result<(), Error> {
        let close_sleep = unsafe { Pin::new_unchecked(&mut self.close_sleep) };
        let ping_sleep = unsafe { Pin::new_unchecked(&mut self.ping_sleep) };

        tracing::debug!(data = ?&data[..], "Pong");

        let Some(expected) = self.last_ping else {
            tracing::debug!("No ping sent");
            return Ok(());
        };

        let expected = expected.to_ne_bytes();

        if expected[..] != data[..] {
            tracing::debug!(?expected, ?data, "Pong doesn't match");
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
        let mut socket = unsafe { Pin::new_unchecked(&mut self.socket) };

        if self.socket_send
            && let Some(message) = self.out.pop_front()
        {
            socket.as_mut().start_send(message)?;
            self.socket_flush = true;
            self.socket_send = false;
        }

        if self.socket_send
            && let Some(frame) = self.buf.read()?
        {
            socket.as_mut().start_send(S::binary(frame))?;

            if self.buf.is_empty() {
                self.buf.clear();
            }

            self.socket_flush = true;
            self.socket_send = false;
        }

        Ok(())
    }

    /// Write a broadcast message.
    ///
    /// Note that the written message is buffered, and will be sent when
    /// [`Server::run`] is called.
    pub fn broadcast<'de, T>(self: Pin<&mut Self>, message: T) -> Result<(), Error>
    where
        T: Broadcast<'de>,
    {
        let this = unsafe { Pin::get_unchecked_mut(self) };

        this.buf.write(ResponseHeader {
            serial: 0,
            broadcast: Some(<T::Endpoint as Listener>::KIND),
            error: None,
        })?;

        this.buf.write(message)?;
        this.buf.done();
        Ok(())
    }

    async fn handle_request(
        &mut self,
        reader: SliceReader<'_>,
        header: RequestHeader<'_>,
    ) -> Result<H::Response, storage::Error> {
        tracing::debug!(?header, "Got request");

        self.buf.write(ResponseHeader {
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
            buf: &mut self.buf,
        };

        let response = self
            .handler
            .handle(header.kind, &mut incoming, &mut outgoing)
            .await;

        if let Some(error) = incoming.error.take() {
            return Err(error);
        }

        if let Some(error) = outgoing.error.take() {
            return Err(error);
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

struct Select<'a, C, P, S> {
    close: &'a mut C,
    ping: &'a mut P,
    socket: &'a mut S,
    wants_socket_send: bool,
    wants_socket_flush: bool,
}

impl<C, P, S> Future for Select<'_, C, P, S>
where
    C: Future,
    P: Future,
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
            close = Pin::new_unchecked(&mut *this.close);
            ping = Pin::new_unchecked(&mut *this.ping);
            socket = Pin::new_unchecked(&mut *this.socket);
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
