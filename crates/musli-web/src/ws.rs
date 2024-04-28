#[cfg(feature = "axum")]
mod axum;
#[cfg(feature = "axum")]
pub use axum::AxumServer;

use core::fmt::{self, Write};
use core::future::Future;
use core::pin::{pin, Pin};
use core::task::{Context, Poll};

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use bytes::Bytes;
use musli::alloc::System;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::storage;
use musli::{Decode, Encode};
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tokio_stream::Stream;

use crate::api;

const MAX_CAPACITY: usize = 1048576;
const CLOSE_NORMAL: u16 = 1000;
const CLOSE_PROTOCOL_ERROR: u16 = 1002;
const CLOSE_TIMEOUT: Duration = Duration::from_secs(30);
const PING_TIMEOUT: Duration = Duration::from_secs(10);

mod sealed {
    pub trait Sealed {}

    #[cfg(feature = "axum")]
    impl Sealed for super::axum::AxumServer {}
    #[cfg(feature = "axum")]
    impl Sealed for axum::extract::ws::WebSocket {}
}

/// Set up an unbounded channel that can be used to send broadcast messages.
pub fn unbounded_channel() -> (UnboundedSender, UnboundedReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();

    (
        UnboundedSender { inner: sender },
        UnboundedReceiver { inner: receiver },
    )
}

/// Set up an bounded channel that can be used to send broadcast messages with
/// the given capacity.
pub fn channel(capacity: usize) -> (Sender, Receiver) {
    let (sender, receiver) = mpsc::channel(capacity);

    (Sender { inner: sender }, Receiver { inner: receiver })
}

/// The broadcast message to send.
pub struct BroadcastMessage {
    kind: &'static str,
    payload: Box<[u8]>,
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
pub(crate) trait Socket: self::sealed::Sealed {
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
pub trait ServerImplementation: self::sealed::Sealed {
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

enum OneOf<E> {
    Error(Error),
    Handler(E),
}

impl<E> From<Error> for OneOf<E> {
    #[inline]
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

impl<E> From<storage::Error> for OneOf<E> {
    #[inline]
    fn from(error: storage::Error) -> Self {
        Self::Error(Error::from(error))
    }
}

impl<E> fmt::Display for OneOf<E>
where
    E: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OneOf::Error(error) => error.fmt(f),
            OneOf::Handler(error) => error.fmt(f),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    #[cfg(feature = "axum")]
    Axum {
        error: axum_core::Error,
    },
    Musli {
        error: storage::Error,
    },
    UnknownRequest {
        kind: Box<str>,
    },
    SendError,
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
            #[cfg(feature = "axum")]
            ErrorKind::Axum { .. } => write!(f, "Error in axum"),
            ErrorKind::Musli { .. } => write!(f, "Error in musli"),
            ErrorKind::UnknownRequest { kind } => {
                write!(f, "Unknown request kind `{kind}`")
            }
            ErrorKind::SendError => write!(f, "Error sending message"),
            ErrorKind::FormatError => write!(f, "Error formatting error response"),
        }
    }
}

impl core::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.kind {
            #[cfg(feature = "axum")]
            ErrorKind::Axum { error } => Some(error),
            ErrorKind::Musli { error } => Some(error),
            _ => None,
        }
    }
}

#[cfg(feature = "axum")]
impl From<axum_core::Error> for Error {
    #[inline]
    fn from(error: axum_core::Error) -> Self {
        Self::new(ErrorKind::Axum { error })
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
pub trait Handler: Send + Sync {
    /// Error returned by handler.
    type Error: 'static + Send + Sync + fmt::Display;

    /// Handle a request.
    fn handle<'this>(
        &'this mut self,
        incoming: &'this mut Incoming<'_>,
        outgoing: &'this mut Outgoing<'_>,
        kind: &'this str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'this;
}

/// The server side of the websocket connection.
pub struct Server<S, H, B = EmptyStream>
where
    S: ServerImplementation,
{
    buf: Buf,
    error: String,
    socket: S::Socket,
    handler: H,
    stream: B,
}

impl<S, H> Server<S, H>
where
    S: ServerImplementation,
{
    /// Construct a new server with the specified handler.
    #[inline]
    pub fn new(socket: S::Socket, handler: H) -> Self {
        Self {
            buf: Buf::default(),
            error: String::new(),
            socket,
            handler,
            stream: EmptyStream,
        }
    }
}

impl<S, H, B> Server<S, H, B>
where
    S: ServerImplementation,
{
    /// Associate a stream with the server.
    #[inline]
    pub fn with_stream<U>(self, stream: U) -> Server<S, H, U>
    where
        U: Stream<Item = BroadcastMessage>,
    {
        Server {
            buf: self.buf,
            error: self.error,
            socket: self.socket,
            handler: self.handler,
            stream,
        }
    }
}

impl<S, H, B> Server<S, H, B>
where
    S: ServerImplementation,
    Error: From<S::Error>,
    H: Handler,
    B: Stream<Item = BroadcastMessage> + Unpin,
{
    /// Run the server.
    pub async fn run(mut self) -> Result<(), Error> {
        tracing::trace!("Accepted");

        let now = Instant::now();
        let mut last_ping = None::<u32>;
        let mut rng = SmallRng::seed_from_u64(0x404241112);
        let mut close_interval = pin!(tokio::time::sleep_until(now + CLOSE_TIMEOUT));
        let mut ping_interval = pin!(tokio::time::sleep_until(now + PING_TIMEOUT));

        let close_here = loop {
            let inner = Inner {
                close: close_interval.as_mut(),
                ping: ping_interval.as_mut(),
                socket: self.socket.next(),
                stream: &mut self.stream,
            };

            match inner.await {
                InnerOutput::Close => {
                    break Some((CLOSE_NORMAL, "connection timed out"));
                }
                InnerOutput::Ping => {
                    let payload = rng.random::<u32>();
                    last_ping = Some(payload);
                    let data = payload.to_ne_bytes().into_iter().collect::<Vec<_>>();
                    tracing::trace!(data = ?&data[..], "Sending ping");
                    self.socket.send(S::ping(data.to_vec().into())).await?;
                    let now = Instant::now();
                    ping_interval.as_mut().reset(now + PING_TIMEOUT);
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

                            match self.handle_request(reader, header).await {
                                Ok(()) => {
                                    self.flush().await?;
                                }
                                Err(error) => {
                                    if write!(self.error, "{error}").is_err() {
                                        return Err(Error::new(ErrorKind::FormatError));
                                    }

                                    self.buf.buffer.clear();

                                    self.buf.write(api::ResponseHeader {
                                        index: header.index,
                                        serial: header.serial,
                                        broadcast: None,
                                        error: Some(self.error.as_str()),
                                    })?;

                                    self.flush().await?;
                                }
                            }
                        }
                        Message::Ping(payload) => {
                            self.socket.send(S::pong(payload)).await?;
                            continue;
                        }
                        Message::Pong(data) => {
                            tracing::trace!(data = ?&data[..], "Pong");

                            let Some(expected) = last_ping else {
                                continue;
                            };

                            if expected.to_ne_bytes()[..] != data[..] {
                                continue;
                            }

                            let now = Instant::now();

                            close_interval.as_mut().reset(now + CLOSE_TIMEOUT);
                            ping_interval.as_mut().reset(now + PING_TIMEOUT);
                            last_ping = None;
                        }
                        Message::Close => break None,
                    }
                }
                InnerOutput::Stream(message) => {
                    let Some(message) = message else {
                        continue;
                    };

                    self.buf.write(api::ResponseHeader {
                        index: 0,
                        serial: 0,
                        broadcast: Some(message.kind),
                        error: None,
                    })?;

                    self.buf.buffer.extend_from_slice(message.payload.as_ref());
                    self.flush().await?;
                }
            }
        };

        if let Some((code, reason)) = close_here {
            tracing::trace!(code, reason, "Closing websocket with reason");
            self.socket.send(S::close(code, reason)).await?;
        } else {
            tracing::trace!("Closing websocket");
        };

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

    async fn handle_request(
        &mut self,
        reader: SliceReader<'_>,
        header: api::RequestHeader<'_>,
    ) -> Result<(), OneOf<H::Error>> {
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

        if let Err(error) = self
            .handler
            .handle(&mut incoming, &mut outgoing, header.kind)
            .await
        {
            return Err(OneOf::Handler(error));
        }

        if let Some(error) = incoming.error.take() {
            return Err(OneOf::Error(Error::new(ErrorKind::Musli { error })));
        }

        if !outgoing.written {
            return Err(OneOf::Error(Error::new(ErrorKind::UnknownRequest {
                kind: header.kind.into(),
            })));
        }

        Ok(())
    }
}

enum InnerOutput<O, B> {
    /// The connection should be closed.
    Close,
    /// A ping message was received.
    Ping,
    /// A message was received.
    Output(O),
    /// The stream output.
    Stream(Option<B>),
}

struct Inner<'a, C, P, S, B> {
    close: C,
    ping: P,
    socket: S,
    stream: &'a mut B,
}

impl<'a, C, P, S, B> Future for Inner<'a, C, P, S, B>
where
    C: Future,
    P: Future,
    S: Future,
    B: Stream + Unpin,
{
    type Output = InnerOutput<S::Output, B::Item>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: This type is not Unpin.
        let (close, ping, socket, stream) = unsafe {
            let this = Pin::get_unchecked_mut(self);
            let close = Pin::new_unchecked(&mut this.close);
            let ping = Pin::new_unchecked(&mut this.ping);
            let socket = Pin::new_unchecked(&mut this.socket);
            let stream = Pin::new_unchecked(&mut *this.stream);
            (close, ping, socket, stream)
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

        if let Poll::Ready(output) = Stream::poll_next(stream, cx) {
            cx.waker().wake_by_ref();
            return Poll::Ready(InnerOutput::Stream(output));
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
        T: Decode<'de, Binary, System>,
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

/// An empty message.
pub enum EmptyMessage {}

/// An empty stream.
pub struct EmptyStream;

impl Stream for EmptyStream {
    type Item = EmptyMessage;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

/// A sender for broadcast messages.
#[derive(Clone)]
pub struct UnboundedSender {
    inner: mpsc::UnboundedSender<BroadcastMessage>,
}

impl UnboundedSender {
    /// Send a broadcast message.
    pub fn send<T>(&self, message: T) -> Result<(), Error>
    where
        T: Encode<Binary> + api::Broadcast,
    {
        let payload = match storage::to_vec(&message) {
            Ok(payload) => payload,
            Err(error) => return Err(Error::new(ErrorKind::Musli { error })),
        };

        let message = BroadcastMessage {
            kind: T::KIND,
            payload: payload.into(),
        };

        match self.inner.send(message) {
            Ok(()) => Ok(()),
            Err(..) => Err(Error::new(ErrorKind::SendError)),
        }
    }
}

/// A receiver for broadcast messages.
///
/// This implements `Stream` in a manner so that it can be used with
/// [`Server::with_stream`].
pub struct UnboundedReceiver {
    inner: mpsc::UnboundedReceiver<BroadcastMessage>,
}

impl Stream for UnboundedReceiver {
    type Item = BroadcastMessage;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_recv(cx)
    }
}

/// A sender for broadcast messages.
#[derive(Clone)]
pub struct Sender {
    inner: mpsc::Sender<BroadcastMessage>,
}

impl Sender {
    /// Send a broadcast message.
    ///
    /// This is asynchronous since it will block if the channel is full.
    pub async fn send<T>(&self, message: T) -> Result<(), Error>
    where
        T: Encode<Binary> + api::Broadcast,
    {
        let payload = match storage::to_vec(&message) {
            Ok(payload) => payload,
            Err(error) => return Err(Error::new(ErrorKind::Musli { error })),
        };

        let message = BroadcastMessage {
            kind: T::KIND,
            payload: payload.into(),
        };

        match self.inner.send(message).await {
            Ok(()) => Ok(()),
            Err(..) => Err(Error::new(ErrorKind::SendError)),
        }
    }
}

/// A receiver for broadcast messages.
///
/// This implements `Stream` in a manner so that it can be used with
/// [`Server::with_stream`].
pub struct Receiver {
    inner: mpsc::Receiver<BroadcastMessage>,
}

impl Stream for Receiver {
    type Item = BroadcastMessage;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_recv(cx)
    }
}
