/// Defines the implementation of a `tokio-tungstenite` integration.
///
/// This is shared between the supported versions, which only differ in the
/// crate they are linked against, the name of the generated implementation
/// type, and the documented version.
#[cfg(any(feature = "tungstenite029", feature = "tungstenite030"))]
macro_rules! tungstenite_impl {
    ($krate:ident, $impl_ty:ident, $version:literal) => {
        use alloc::string::String;

        use core::future::{Future, poll_fn};
        use core::pin::Pin;

        use bytes::Bytes;
        use futures_core03::Stream;
        use futures_sink03::Sink;
        use tokio::net::TcpStream;
        use $krate::tungstenite::protocol::Message as WsMessage;
        use $krate::tungstenite::{Error, Utf8Bytes};
        use $krate::{MaybeTlsStream, WebSocketStream, connect_async};

        use crate::api::Mode;
        use crate::client::{ClientImpl, EmptyCallback, Message, ServiceBuilder, SocketImpl};

        /// The socket type used by this implementation.
        #[doc(hidden)]
        pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

        #[doc = concat!("The public facing API for use with `tokio-tungstenite` `", $version, "`.")]
        pub mod prelude {
            pub mod ws {
                //! Organization module prefixing all exported items with `ws` for
                //! convenient namespacing.

                pub use crate::api::ChannelId;
                pub use crate::client::{
                    Channel, EmptyCallback, Error, Handle, Listener, Packet, RawPacket,
                    RequestBuilder, State, StateListener,
                };

                use super::super::$impl_ty;

                /// Implementation alias for [`connect`].
                ///
                /// [`connect`]: super::super::connect
                #[inline]
                pub fn connect(url: impl AsRef<str>) -> ServiceBuilder<EmptyCallback> {
                    super::super::connect(url)
                }

                /// Implementation alias for [`Service`].
                ///
                /// [`Service`]: crate::client::Service
                pub type Service = crate::client::Service<$impl_ty>;

                /// Implementation alias for [`ServiceBuilder`].
                ///
                /// [`ServiceBuilder`]: crate::client::ServiceBuilder
                pub type ServiceBuilder<C> = crate::client::ServiceBuilder<$impl_ty, C>;
            }
        }

        #[doc = concat!("Client implementation for `tokio-tungstenite` `", $version, "`.")]
        ///
        /// See [`connect()`].
        #[derive(Clone, Copy)]
        pub enum $impl_ty {}

        /// Construct a new [`ServiceBuilder`] which will connect to `url`.
        ///
        /// Note that no connection is established until [`Service::run`] is called.
        ///
        /// [`Service::run`]: crate::client::Service::run
        #[inline]
        pub fn connect(url: impl AsRef<str>) -> ServiceBuilder<$impl_ty, EmptyCallback> {
            crate::client::connect(url)
        }

        impl crate::client::sealed_client::Sealed for $impl_ty {}

        impl ClientImpl for $impl_ty {
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
            fn recv(
                &mut self,
            ) -> impl Future<Output = Option<Result<Message, Self::Error>>> + Send + '_ {
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
    };
}

/// Implements the callback integration for a version of `yew`.
///
/// This is shared between the supported versions, which only differ in the
/// crate they are linked against.
#[cfg(any(feature = "yew022", feature = "yew023"))]
macro_rules! yew_impl {
    ($krate:ident) => {
        use $krate::Callback;

        impl<I> crate::web::Callback<I> for Callback<I>
        where
            I: 'static,
        {
            #[inline]
            fn call(&self, result: I) {
                self.emit(result);
            }
        }
    };
}
