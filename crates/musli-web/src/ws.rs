use core::fmt::{self, Write};
use core::future::Future;
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
use tokio::time::{Duration, Instant};

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
    type Error;

    #[doc(hidden)]
    type Message;

    #[doc(hidden)]
    async fn next(&mut self) -> Option<Result<Message, Self::Error>>;

    #[doc(hidden)]
    async fn send(&mut self, message: Self::Message) -> Result<(), Self::Error>;
}

/// The implementation of a server.
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
    fn binary(data: Bytes) -> Self::Message;

    #[doc(hidden)]
    fn close(code: u16, reason: &str) -> Self::Message;
}

enum OneOf<'a, E> {
    Handler { error: E },
    Musli { error: storage::Error },
    UnknownRequest { kind: &'a str },
}

impl<E> From<storage::Error> for OneOf<'_, E> {
    #[inline]
    fn from(error: storage::Error) -> Self {
        Self::Musli { error }
    }
}

impl<E> fmt::Display for OneOf<'_, E>
where
    E: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OneOf::Handler { error } => error.fmt(f),
            OneOf::Musli { error } => error.fmt(f),
            OneOf::UnknownRequest { kind } => {
                write!(f, "Unknown request kind: {kind}")
            }
        }
    }
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
}

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

type Result<T, E = Error> = core::result::Result<T, E>;

/// A handler for incoming requests.
pub trait Handler {
    /// Error returned by handler.
    type Error: fmt::Display;

    /// Handle a request.
    fn handle<'this>(
        &'this mut self,
        kind: &'this str,
        incoming: &'this mut Incoming<'_>,
        outgoing: &'this mut Outgoing<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'this;
}

/// The server side of the websocket connection.
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
    close_interval: tokio::time::Sleep,
    ping_interval: tokio::time::Sleep,
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
            close_interval: tokio::time::sleep_until(now + CLOSE_TIMEOUT),
            ping_interval: tokio::time::sleep_until(now + PING_TIMEOUT),
        }
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
}

impl<S, H> Server<S, H>
where
    S: ServerImplementation,
    Error: From<S::Error>,
    H: Handler,
{
    /// Run the server.
    pub async fn run(self: Pin<&mut Self>) -> Result<(), Error> {
        tracing::trace!("Accepted");

        // SAFETY: Deal with running the server.
        let this = unsafe { Pin::get_unchecked_mut(self) };

        let close_here = loop {
            // SAFETY: The server is pinned.
            let mut close = unsafe { Pin::new_unchecked(&mut this.close_interval) };
            let mut ping = unsafe { Pin::new_unchecked(&mut this.ping_interval) };

            let inner = Inner {
                close: close.as_mut(),
                ping: ping.as_mut(),
                socket: this.socket.next(),
            };

            match inner.await {
                InnerOutput::Close => {
                    break Some((CLOSE_NORMAL, "connection timed out"));
                }
                InnerOutput::Ping => {
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

                            match this.handle_request(reader, header).await {
                                Ok(()) => {
                                    this.flush().await?;
                                }
                                Err(error) => {
                                    if write!(this.error, "{error}").is_err() {
                                        return Err(Error::new(ErrorKind::FormatError));
                                    }

                                    this.buf.buffer.clear();

                                    this.buf.write(api::ResponseHeader {
                                        index: header.index,
                                        serial: header.serial,
                                        broadcast: None,
                                        error: Some(this.error.as_str()),
                                    })?;

                                    this.flush().await?;
                                }
                            }
                        }
                        Message::Ping(payload) => {
                            this.socket.send(S::pong(payload)).await?;
                            continue;
                        }
                        Message::Pong(data) => {
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
            this.socket.send(S::close(code, reason)).await?;
        } else {
            tracing::trace!("Closing websocket");
        };

        Ok(())
    }

    /// Send a broadcast message on the server.
    pub async fn broadcast<'de, T>(self: Pin<&mut Self>, message: T) -> Result<(), Error>
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
        this.flush().await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        self.socket
            .send(S::binary(self.buf.buffer.to_vec().into()))
            .await?;
        self.error.clear();
        self.buf.buffer.clear();
        self.buf.buffer.shrink_to(MAX_CAPACITY);
        Ok(())
    }

    async fn handle_request<'header>(
        &mut self,
        reader: SliceReader<'_>,
        header: api::RequestHeader<'header>,
    ) -> Result<(), OneOf<'header, H::Error>> {
        tracing::trace!("Got request: {header:?}");

        self.buf.write(api::ResponseHeader {
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
            written: false,
            buf: &mut self.buf,
        };

        let result = self
            .handler
            .handle(header.kind, &mut incoming, &mut outgoing)
            .await;

        if let Err(error) = result {
            return Err(OneOf::Handler { error });
        }

        if let Some(error) = incoming.error.take() {
            return Err(OneOf::Musli { error });
        }

        if !outgoing.written {
            return Err(OneOf::UnknownRequest { kind: header.kind });
        }

        Ok(())
    }
}

enum InnerOutput<O> {
    /// The connection should be closed.
    Close,
    /// A ping message was received.
    Ping,
    /// A message was received.
    Output(O),
}

struct Inner<C, P, S> {
    close: C,
    ping: P,
    socket: S,
}

impl<C, P, S> Future for Inner<C, P, S>
where
    C: Future,
    P: Future,
    S: Future,
{
    type Output = InnerOutput<S::Output>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: This type is not Unpin.
        let (close, ping, socket) = unsafe {
            let this = Pin::get_unchecked_mut(self);
            let close = Pin::new_unchecked(&mut this.close);
            let ping = Pin::new_unchecked(&mut this.ping);
            let socket = Pin::new_unchecked(&mut this.socket);
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

        if let Poll::Ready(output) = socket.poll(cx) {
            cx.waker().wake_by_ref();
            return Poll::Ready(InnerOutput::Output(output));
        }

        Poll::Pending
    }
}

/// An incoming request.
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

/// Handler for an outgoing buffer.
pub struct Outgoing<'a> {
    error: Option<storage::Error>,
    written: bool,
    buf: &'a mut Buf,
}

impl Outgoing<'_> {
    /// Write a response.
    pub fn write<T>(&mut self, value: T)
    where
        T: Encode<Binary>,
    {
        if let Err(error) = self.buf.write(value) {
            self.error = Some(error);
        } else {
            self.written = true;
        }
    }
}

#[derive(Default)]
struct Buf {
    buffer: Vec<u8>,
}

impl Buf {
    fn write<T>(&mut self, value: T) -> Result<(), storage::Error>
    where
        T: Encode<Binary>,
    {
        storage::to_writer(&mut self.buffer, &value)?;
        Ok(())
    }
}
