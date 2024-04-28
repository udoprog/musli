use core::fmt::{self, Write};
use core::future::Future;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use musli::alloc::System;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::{Decode, Encode};
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::time::Duration;
use tokio_stream::StreamExt;

use crate::api;

const MAX_CAPACITY: usize = 1048576;

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

impl<E> From<musli::storage::Error> for OneOf<E> {
    #[inline]
    fn from(error: musli::storage::Error) -> Self {
        Self::Error(Error::from(error))
    }
}

impl<E> fmt::Display for OneOf<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OneOf::Error(error) => error.fmt(f),
            OneOf::Handler(error) => error.fmt(f),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    Axum { error: axum::Error },
    Musli { error: musli::storage::Error },
    UnknownRequest { kind: Box<str> },
    FormatError,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Axum { .. } => write!(f, "Error in axum"),
            ErrorKind::Musli { .. } => write!(f, "Error in musli"),
            ErrorKind::UnknownRequest { kind } => {
                write!(f, "Unknown request kind: {kind}")
            }
            ErrorKind::FormatError => write!(f, "Error formatting error response"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Axum { error } => Some(error),
            ErrorKind::Musli { error } => Some(error),
            _ => None,
        }
    }
}

impl From<axum::Error> for Error {
    #[inline]
    fn from(error: axum::Error) -> Self {
        Self::new(ErrorKind::Axum { error })
    }
}

impl From<musli::storage::Error> for Error {
    #[inline]
    fn from(error: musli::storage::Error) -> Self {
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
pub struct Server<H> {
    buf: Buf,
    error: String,
    socket: WebSocket,
    handler: H,
}

impl<H> Server<H> {
    /// Construct a new server with the specified handler.
    pub fn new(socket: WebSocket, handler: H) -> Self {
        Self {
            buf: Buf::default(),
            error: String::new(),
            socket,
            handler,
        }
    }
}

impl<H> Server<H>
where
    H: Handler,
{
    /// Run the server.
    pub async fn run(&mut self) -> Result<(), Error> {
        tracing::trace!("Accepted");

        const CLOSE_NORMAL: u16 = 1000;
        const CLOSE_PROTOCOL_ERROR: u16 = 1002;
        const CLOSE_TIMEOUT: Duration = Duration::from_secs(30);
        const PING_TIMEOUT: Duration = Duration::from_secs(10);

        let mut last_ping = None::<u32>;
        let mut rng = SmallRng::seed_from_u64(0x404241112);
        let mut close_interval = tokio::time::interval(CLOSE_TIMEOUT);
        close_interval.reset();

        let mut ping_interval = tokio::time::interval(PING_TIMEOUT);
        ping_interval.reset();

        let close_here = loop {
            tokio::select! {
                _ = close_interval.tick() => {
                    break Some((CLOSE_NORMAL, "connection timed out"));
                }
                _ = ping_interval.tick() => {
                    let payload = rng.gen::<u32>();
                    last_ping = Some(payload);
                    let data = payload.to_ne_bytes().into_iter().collect::<Vec<_>>();
                    tracing::trace!(data = ?&data[..], "Sending ping");
                    self.socket.send(Message::Ping(data.to_vec().into())).await?;
                    ping_interval.reset();
                }
                message = self.socket.next() => {
                    let Some(message) = message else {
                        break None;
                    };

                    match message? {
                        Message::Text(_) => break Some((CLOSE_PROTOCOL_ERROR, "unsupported message")),
                        Message::Binary(bytes) => {
                            let mut reader = SliceReader::new(&bytes);

                            let header = match musli::storage::decode(&mut reader) {
                                Ok(header) => header,
                                Err(error) => {
                                    tracing::warn!(?error, "Failed to decode request header");
                                    break Some((CLOSE_PROTOCOL_ERROR, "invalid request"));
                                }
                            };

                            match self.handle_request(reader, header).await {
                                Ok(()) => {
                                    self.flush().await?;
                                },
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
                        },
                        Message::Ping(payload) => {
                            self.socket.send(Message::Pong(payload)).await?;
                            continue;
                        },
                        Message::Pong(data) => {
                            tracing::trace!(data = ?&data[..], "Pong");

                            let Some(expected) = last_ping else {
                                continue;
                            };

                            if expected.to_ne_bytes()[..] != data[..] {
                                continue;
                            }

                            close_interval.reset();
                            ping_interval.reset();
                            last_ping = None;
                        },
                        Message::Close(_) => break None,
                    }
                }
            }
        };

        if let Some((code, reason)) = close_here {
            tracing::trace!(code, reason, "Closing websocket with reason");

            self.socket
                .send(Message::Close(Some(CloseFrame {
                    code,
                    reason: reason.into(),
                })))
                .await?;
        } else {
            tracing::trace!("Closing websocket");
        };

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        self.socket
            .send(Message::Binary(self.buf.buffer.to_vec().into()))
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

/// An incoming request.
pub struct Incoming<'de> {
    error: Option<musli::storage::Error>,
    reader: SliceReader<'de>,
}

impl<'de> Incoming<'de> {
    /// Read a request.
    #[inline]
    pub fn read<T>(&mut self) -> Option<T>
    where
        T: Decode<'de, Binary, System>,
    {
        match musli::storage::decode(&mut self.reader) {
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
    error: Option<musli::storage::Error>,
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
pub struct Buf {
    buffer: Vec<u8>,
}

impl Buf {
    fn write<T>(&mut self, value: T) -> Result<(), musli::storage::Error>
    where
        T: Encode<Binary>,
    {
        musli::storage::to_writer(&mut self.buffer, &value)?;
        Ok(())
    }
}
