//! The server implementation for axum.
//!
//! Use [`server()`] to set up the server and feed it incoming requests.

use core::pin::Pin;
use core::task::Poll;
use core::task::{Context, ready};

use bytes::Bytes;

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
    type Message = Message;
    type Error = Error;

    #[inline]
    #[allow(private_interfaces)]
    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Result<ws::Message, Self::Error>>> {
        let Some(result) = ready!(futures_core03::Stream::poll_next(self, ctx)) else {
            return Poll::Ready(None);
        };

        let message = match result {
            Ok(message) => message,
            Err(err) => return Poll::Ready(Some(Err(err))),
        };

        let message = match message {
            Message::Text(..) => ws::Message::Text,
            Message::Binary(data) => ws::Message::Binary(data),
            Message::Ping(data) => ws::Message::Ping(data),
            Message::Pong(data) => ws::Message::Pong(data),
            Message::Close(..) => ws::Message::Close,
        };

        Poll::Ready(Some(Ok(message)))
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
    fn binary(data: &[u8]) -> Self::Message {
        Message::Binary(Bytes::from(data.to_vec()))
    }

    #[inline]
    fn close(code: u16, reason: &str) -> Self::Message {
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))
    }
}
