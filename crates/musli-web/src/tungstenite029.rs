//! Client side implementation for [`tokio-tungstenite`] `0.29.x`.
//!
//! This allows non-browser clients to talk to the same websocket API which is
//! served by [`ws::Server`], such as through the [`axum08`] integration.
//!
//! [`axum08`]: <https://docs.rs/musli-web/latest/musli_web/axum08/>
//! [`tokio-tungstenite`]: <https://docs.rs/tokio-tungstenite/0.29>
//! [`ws::Server`]: <https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html>
//!
//! # Examples
//!
//! ```no_run
//! use musli_web::tungstenite029::prelude::*;
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
//!     api::define! {
//!         pub type Hello;
//!
//!         impl Endpoint for Hello {
//!             impl<'de> Request for HelloRequest<'de>;
//!             type Response<'de> = HelloResponse<'de>;
//!         }
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn core::error::Error>> {
//! let mut service = ws::connect("ws://localhost:3000/ws")
//!     .on_error(|error| {
//!         tracing::error!("WebSocket error: {error}");
//!     })
//!     .build();
//!
//! let handle = service.handle().clone();
//!
//! tokio::spawn(async move {
//!     if let Err(error) = service.run().await {
//!         tracing::error!("WebSocket service error: {error}");
//!     }
//! });
//!
//! handle.wait_until_open().await?;
//!
//! let packet = handle
//!     .request()
//!     .body(api::HelloRequest { message: "Hello!" })
//!     .send()
//!     .await?;
//!
//! let response = packet.decode()?;
//! println!("Response: {}", response.message);
//! # Ok(())
//! # }
//! ```

use alloc::string::String;

use core::future::{Future, poll_fn};
use core::pin::Pin;

use bytes::Bytes;
use futures_core03::Stream;
use futures_sink03::Sink;
use tokio::net::TcpStream;
use tokio_tungstenite029::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite029::tungstenite::{Error, Utf8Bytes};
use tokio_tungstenite029::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::api::Mode;
use crate::client::{ClientImpl, EmptyCallback, Message, ServiceBuilder, SocketImpl};

/// The socket type used by this implementation.
#[doc(hidden)]
pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub mod prelude {
    //! The public facing API for use with `tokio-tungstenite` `0.29.x`.

    pub mod ws {
        //! Organization module prefixing all exported items with `ws` for
        //! convenient namespacing.

        pub use crate::api::ChannelId;
        pub use crate::client::{
            Channel, EmptyCallback, Error, Handle, Listener, Packet, RawPacket, RequestBuilder,
            State, StateListener,
        };

        use crate::tungstenite029::Tungstenite029Impl;

        /// Implementation alias for [`connect`].
        ///
        /// [`connect`]: crate::tungstenite029::connect
        #[inline]
        pub fn connect(url: impl AsRef<str>) -> ServiceBuilder<EmptyCallback> {
            crate::tungstenite029::connect(url)
        }

        /// Implementation alias for [`Service`].
        ///
        /// [`Service`]: crate::client::Service
        pub type Service = crate::client::Service<Tungstenite029Impl>;

        /// Implementation alias for [`ServiceBuilder`].
        ///
        /// [`ServiceBuilder`]: crate::client::ServiceBuilder
        pub type ServiceBuilder<C> = crate::client::ServiceBuilder<Tungstenite029Impl, C>;
    }
}

/// Client implementation for `tokio-tungstenite` `0.29.x`.
///
/// See [`connect()`].
#[derive(Clone, Copy)]
pub enum Tungstenite029Impl {}

/// Construct a new [`ServiceBuilder`] which will connect to `url`.
///
/// Note that no connection is established until [`Service::run`] is called.
///
/// [`Service::run`]: crate::client::Service::run
#[inline]
pub fn connect(url: impl AsRef<str>) -> ServiceBuilder<Tungstenite029Impl, EmptyCallback> {
    crate::client::connect(url)
}

impl crate::client::sealed_client::Sealed for Tungstenite029Impl {}

impl ClientImpl for Tungstenite029Impl {
    type Error = Error;
    type Socket = Socket;

    #[inline]
    async fn connect(url: &str) -> Result<Self::Socket, Self::Error> {
        let (socket, _) = connect_async(url).await?;
        Ok(socket)
    }
}

impl crate::client::sealed_socket::Sealed for Socket {}

impl SocketImpl for Socket {
    type Error = Error;

    #[inline]
    fn recv(&mut self) -> impl Future<Output = Option<Result<Message, Self::Error>>> + Send + '_ {
        poll_fn(move |cx| {
            Pin::new(&mut *self)
                .poll_next(cx)
                .map(|message| message.map(|message| message.map(convert)))
        })
    }

    #[inline]
    fn send(
        &mut self,
        mode: Mode,
        data: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        let message = match mode {
            Mode::Binary => WsMessage::Binary(Bytes::copy_from_slice(data)),
            // NB: A text frame is only ever written in a mode which guarantees
            // that everything in it is valid UTF-8, so the lossy path is never
            // taken.
            Mode::Text => WsMessage::Text(Utf8Bytes::from(&*String::from_utf8_lossy(data))),
        };

        async move {
            poll_fn(|cx| Pin::new(&mut *self).poll_ready(cx)).await?;
            Pin::new(&mut *self).start_send(message)?;
            poll_fn(|cx| Pin::new(&mut *self).poll_flush(cx)).await
        }
    }

    #[inline]
    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        poll_fn(move |cx| Pin::new(&mut *self).poll_close(cx))
    }
}

/// Convert a tungstenite message into a message understood by the client.
#[inline]
fn convert(message: WsMessage) -> Message {
    match message {
        WsMessage::Binary(data) => Message::Binary(data),
        WsMessage::Text(data) => Message::Text(Bytes::from(data)),
        WsMessage::Ping(..) => Message::Ping,
        WsMessage::Pong(..) => Message::Pong,
        WsMessage::Close(..) => Message::Close,
        // NB: Raw frames are never produced while reading, so this is a
        // protocol violation which tears the connection down.
        WsMessage::Frame(..) => Message::Unsupported,
    }
}
