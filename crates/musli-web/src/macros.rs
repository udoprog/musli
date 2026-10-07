/// Defines the implementation of a `tokio-tungstenite` integration.
///
/// This is shared between the supported versions, which only differ in the
/// crate they are linked against, the name of the generated implementation
/// type, and the documented version.
#[cfg(any(feature = "tungstenite029", feature = "tungstenite030"))]
macro_rules! tungstenite_impl {
    ($krate:ident, $impl_ty:ident, $version:literal) => {
        use alloc::boxed::Box;
        use alloc::string::{String, ToString};

        use core::future::{Future, poll_fn};
        use core::pin::Pin;
        use core::task::{Context, Poll};

        use std::io;

        use bytes::Bytes;
        use futures_core03::Stream;
        use futures_sink03::Sink;
        use tokio::io::{AsyncRead, AsyncWrite};
        use tokio::net::TcpStream;
        use $krate::tungstenite::protocol::Message as WsMessage;
        use $krate::tungstenite::{Error, Utf8Bytes};
        use $krate::{MaybeTlsStream, WebSocketStream, client_async, connect_async};

        use crate::api::Mode;
        use crate::client::{ClientImpl, EmptyCallback, Message, ServiceBuilder, SocketImpl};

        /// A byte stream a websocket can be established over, see
        /// [`connect_with()`].
        ///
        /// Implemented for anything which is [`AsyncRead`] and [`AsyncWrite`],
        /// such as a `TcpStream`, a `UnixStream`, or one half of
        /// `tokio::io::duplex`.
        ///
        /// [`AsyncRead`]: <https://docs.rs/tokio/1/tokio/io/trait.AsyncRead.html>
        /// [`AsyncWrite`]: <https://docs.rs/tokio/1/tokio/io/trait.AsyncWrite.html>
        pub trait ByteStream: 'static + Send + Unpin + AsyncRead + AsyncWrite {}

        impl<S> ByteStream for S where S: 'static + Send + Unpin + AsyncRead + AsyncWrite {}

        /// A boxed future opening a [`ByteStream`].
        type OpenFuture = Pin<Box<dyn Future<Output = io::Result<Box<dyn ByteStream>>> + Send>>;

        /// The socket type used by this implementation.
        #[doc(hidden)]
        pub enum Socket {
            /// A socket connected to a url.
            Url(Box<WebSocketStream<MaybeTlsStream<TcpStream>>>),
            /// A socket established over a caller-supplied stream.
            Stream(Box<WebSocketStream<Box<dyn ByteStream>>>),
        }

        /// A caller-supplied way of opening the stream a websocket is
        /// established over, see [`connect_with()`].
        #[doc(hidden)]
        pub struct Connector {
            url: String,
            open: Box<dyn FnMut() -> OpenFuture + Send>,
        }

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

                use core::future::Future;

                use super::super::$impl_ty;

                /// Implementation alias for [`connect`].
                ///
                /// [`connect`]: super::super::connect
                #[inline]
                pub fn connect(url: impl AsRef<str>) -> ServiceBuilder<EmptyCallback> {
                    super::super::connect(url)
                }

                /// Implementation alias for [`connect_with`].
                ///
                /// [`connect_with`]: super::super::connect_with
                #[inline]
                pub fn connect_with<F, O, S>(
                    url: impl AsRef<str>,
                    open: F,
                ) -> ServiceBuilder<EmptyCallback>
                where
                    F: 'static + Send + FnMut() -> O,
                    O: 'static + Send + Future<Output = std::io::Result<S>>,
                    S: super::super::ByteStream,
                {
                    super::super::connect_with(url, open)
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
        /// Note that no connection is established until [`Service::run`] or
        /// [`Service::try_connect`] is called.
        ///
        /// [`Service::run`]: crate::client::Service::run
        /// [`Service::try_connect`]: crate::client::Service::try_connect
        #[inline]
        pub fn connect(url: impl AsRef<str>) -> ServiceBuilder<$impl_ty, EmptyCallback> {
            crate::client::connect(url)
        }

        /// Construct a new [`ServiceBuilder`] which establishes its websocket
        /// over a stream opened by `open`, instead of connecting to `url`.
        ///
        /// `open` is called for every connection attempt, including
        /// reconnects, so it can for example connect a `tokio::net::UnixStream`.
        /// An error it returns is treated like any other failure to connect:
        /// retried with a backoff by [`Service::run`], or returned by
        /// [`Service::try_connect`].
        ///
        /// `url` is the request the websocket handshake is performed with, such
        /// as `ws://localhost/ws`. It names the host and path the server sees,
        /// but is never connected to.
        ///
        /// Note that no connection is established until [`Service::run`] or
        /// [`Service::try_connect`] is called.
        ///
        /// [`Service::run`]: crate::client::Service::run
        /// [`Service::try_connect`]: crate::client::Service::try_connect
        pub fn connect_with<F, O, S>(
            url: impl AsRef<str>,
            mut open: F,
        ) -> ServiceBuilder<$impl_ty, EmptyCallback>
        where
            F: 'static + Send + FnMut() -> O,
            O: 'static + Send + Future<Output = io::Result<S>>,
            S: ByteStream,
        {
            let url = url.as_ref();

            let connector = Connector {
                url: url.to_string(),
                open: Box::new(move || {
                    let future = open();

                    Box::pin(async move {
                        let stream = future.await?;
                        Ok(Box::new(stream) as Box<dyn ByteStream>)
                    })
                }),
            };

            crate::client::connect_with(url, connector)
        }

        impl crate::client::sealed_client::Sealed for $impl_ty {}

        impl ClientImpl for $impl_ty {
            type Error = Error;
            type Socket = Socket;
            type Connector = Connector;

            #[inline]
            async fn connect(url: &str) -> Result<Self::Socket, Self::Error> {
                let (socket, _) = connect_async(url).await?;
                Ok(Socket::Url(Box::new(socket)))
            }

            #[inline]
            fn connect_with(
                connector: &mut Self::Connector,
            ) -> impl Future<Output = Result<Self::Socket, Self::Error>> + Send + '_ {
                let open = (connector.open)();

                async move {
                    let stream = open.await.map_err(Error::Io)?;
                    let (socket, _) = client_async(connector.url.as_str(), stream).await?;
                    Ok(Socket::Stream(Box::new(socket)))
                }
            }
        }

        impl Socket {
            #[inline]
            fn poll_next_message(
                &mut self,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Result<WsMessage, Error>>> {
                match self {
                    Socket::Url(socket) => Pin::new(&mut **socket).poll_next(cx),
                    Socket::Stream(socket) => Pin::new(&mut **socket).poll_next(cx),
                }
            }

            #[inline]
            fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
                match self {
                    Socket::Url(socket) => Pin::new(&mut **socket).poll_ready(cx),
                    Socket::Stream(socket) => Pin::new(&mut **socket).poll_ready(cx),
                }
            }

            #[inline]
            fn start_send(&mut self, message: WsMessage) -> Result<(), Error> {
                match self {
                    Socket::Url(socket) => Pin::new(&mut **socket).start_send(message),
                    Socket::Stream(socket) => Pin::new(&mut **socket).start_send(message),
                }
            }

            #[inline]
            fn poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
                match self {
                    Socket::Url(socket) => Pin::new(&mut **socket).poll_flush(cx),
                    Socket::Stream(socket) => Pin::new(&mut **socket).poll_flush(cx),
                }
            }

            #[inline]
            fn poll_close(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
                match self {
                    Socket::Url(socket) => Pin::new(&mut **socket).poll_close(cx),
                    Socket::Stream(socket) => Pin::new(&mut **socket).poll_close(cx),
                }
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
                    self.poll_next_message(cx)
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
                    poll_fn(|cx| self.poll_ready(cx)).await?;
                    self.start_send(message)?;
                    poll_fn(|cx| self.poll_flush(cx)).await
                }
            }

            #[inline]
            fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
                poll_fn(move |cx| self.poll_close(cx))
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
