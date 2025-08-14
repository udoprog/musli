//! The server implementation for axum.
//!
//! Use [`server()`] to set up the server and feed it incoming requests.

use bytes::Bytes;
use tokio_stream::StreamExt;

use axum_core05::Error;
use axum08::extract::ws::{CloseFrame, Message, WebSocket};

use crate::ws::{self, Handler, Server, ServerImplementation, Socket};

/// Construct a new axum server with the specified handler.
#[inline]
pub fn server<H>(socket: WebSocket, handler: H) -> Server<AxumServer, H>
where
    H: Handler,
{
    Server::new(socket, handler)
}

#[non_exhaustive]
pub enum AxumServer {}

impl Socket for WebSocket {
    type Error = Error;
    type Message = Message;

    #[inline]
    #[allow(private_interfaces)]
    async fn next(&mut self) -> Option<Result<ws::Message, Self::Error>> {
        let result = StreamExt::next(self).await?;

        match result {
            Ok(Message::Text(..)) => Some(Ok(ws::Message::Text)),
            Ok(Message::Binary(data)) => Some(Ok(ws::Message::Binary(data))),
            Ok(Message::Ping(data)) => Some(Ok(ws::Message::Ping(data))),
            Ok(Message::Pong(data)) => Some(Ok(ws::Message::Pong(data))),
            Ok(Message::Close(..)) => Some(Ok(ws::Message::Close)),
            Err(err) => Some(Err(err)),
        }
    }

    #[inline]
    async fn send(&mut self, message: Self::Message) -> Result<(), Self::Error> {
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
