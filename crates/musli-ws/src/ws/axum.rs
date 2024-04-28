use bytes::Bytes;
use tokio_stream::StreamExt;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum_core::Error;

use super::{Server, ServerImplementation, Socket};

impl<H> Server<AxumServer, H> {
    /// Construct a new axum server with the specified handler.
    #[inline]
    pub fn axum(socket: axum::extract::ws::WebSocket, handler: H) -> Self {
        Self::new(socket, handler)
    }
}

#[non_exhaustive]
pub enum AxumServer {}

impl Socket for WebSocket {
    type Error = Error;
    type Message = Message;

    #[inline]
    #[allow(private_interfaces)]
    async fn next(&mut self) -> Option<Result<super::Message, Self::Error>> {
        let result = StreamExt::next(self).await?;

        match result {
            Ok(Message::Text(..)) => Some(Ok(super::Message::Text)),
            Ok(Message::Binary(data)) => Some(Ok(super::Message::Binary(data))),
            Ok(Message::Ping(data)) => Some(Ok(super::Message::Ping(data))),
            Ok(Message::Pong(data)) => Some(Ok(super::Message::Pong(data))),
            Ok(Message::Close(..)) => Some(Ok(super::Message::Close)),
            Err(err) => Some(Err(err)),
        }
    }

    #[inline]
    async fn send(&mut self, message: Self::Message) -> super::Result<(), Self::Error> {
        WebSocket::send(self, message).await
    }
}

impl ServerImplementation for AxumServer {
    type Error = Error;
    type Message = Message;
    type Socket = WebSocket;

    #[inline]
    fn ping(data: Bytes) -> Self::Message {
        Message::Ping(data)
    }

    #[inline]
    fn pong(data: Bytes) -> Self::Message {
        Message::Pong(data)
    }

    #[inline]
    fn binary(data: Bytes) -> Self::Message {
        Message::Binary(data)
    }

    #[inline]
    fn close(code: u16, reason: &str) -> Self::Message {
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))
    }
}
