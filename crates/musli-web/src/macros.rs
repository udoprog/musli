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

/// Implements the integration for a version of `yew`.
///
/// This is shared between the supported versions, which only differ in the
/// crate they are linked against.
///
/// NB: yew's own `#[hook]` and `#[derive(Properties)]` macros expand to paths
/// through `::yew`, which this crate does not link under that name, so the
/// hooks here implement [`Hook`] by hand.
#[cfg(any(feature = "yew022", feature = "yew023"))]
macro_rules! yew_impl {
    ($krate:ident) => {
        use alloc::rc::Rc;
        use core::cell::{Cell, RefCell};
        use core::fmt;
        use core::mem;

        use $krate::functional::{
            Hook, HookContext, use_context, use_force_update, use_memo, use_mut_ref,
        };
        use $krate::{Callback, Component, Context, ContextProvider};

        use crate::web03::prelude::ws::{
            Channel, ChannelEvent, Handle, KeyedRequests, Request, Requests, State,
        };

        impl<I> crate::web::Callback<I> for Callback<I>
        where
            I: 'static,
        {
            #[inline]
            fn call(&self, result: I) {
                self.emit(result);
            }
        }

        /// The message a missing [`HandleProvider`] panics with.
        const MISSING_PROVIDER: &str = concat!(
            "no ws::Handle in context: wrap the component tree in musli_web::",
            stringify!($krate),
            "::HandleProvider"
        );

        /// Provides a [`ws::Handle`] to every component below it.
        ///
        /// This is yew's [`ContextProvider`] for [`ws::Handle`], so it takes
        /// the handle as its `context` property. Components below it get the
        /// handle with [`use_handle`] in a function component, or
        /// [`ContextExt::handle`] in a struct component.
        ///
        /// The handle is usually that of a [`ws::Service`] the root component
        /// keeps for as long as the application runs, see the [module
        /// documentation](self) for an example.
        ///
        /// [`ws::Handle`]: crate::web::Handle
        /// [`ws::Service`]: crate::web::Service
        pub type HandleProvider = ContextProvider<Handle>;

        /// Get the [`ws::Handle`] provided by the closest [`HandleProvider`]
        /// above the component.
        ///
        /// The component renders again if the provided handle changes.
        ///
        /// # Panics
        ///
        /// Panics if the component is not below a [`HandleProvider`]. Use
        /// `use_context::<ws::Handle>()` to get an `Option` instead.
        ///
        /// [`ws::Handle`]: crate::web::Handle
        ///
        /// # Examples
        ///
        #[doc = concat!("```\n# extern crate ", stringify!($krate), " as yew;")]
        /// use yew::prelude::*;
        #[doc = concat!("use musli_web::", stringify!($krate), "::use_handle;")]
        ///
        /// #[function_component(Status)]
        /// fn status() -> Html {
        ///     let handle = use_handle();
        ///     html!(<p>{format!("Connected {} time(s)", handle.generation())}</p>)
        /// }
        /// ```
        pub fn use_handle() -> impl Hook<Output = Handle> {
            struct HookProvider;

            impl Hook for HookProvider {
                type Output = Handle;

                fn run(self, ctx: &mut HookContext) -> Self::Output {
                    match use_context::<Handle>().run(ctx) {
                        Some(handle) => handle,
                        None => panic!("{MISSING_PROVIDER}"),
                    }
                }
            }

            HookProvider
        }

        /// Keep a [`ws::Request`] for as long as the component is mounted.
        ///
        /// A request is cancelled when its [`ws::Request`] is dropped, so a
        /// function component needs somewhere to keep it between renders.
        /// Store a sent request with [`UseRequestHandle::set`], which cancels
        /// the request stored before it. The stored request is cancelled when
        /// the component is unmounted.
        ///
        /// [`ws::Request`]: crate::web::Request
        ///
        /// # Examples
        ///
        #[doc = concat!("```\n# extern crate ", stringify!($krate), " as yew;")]
        /// use yew::prelude::*;
        /// use musli_web::web03::prelude::*;
        #[doc = concat!("use musli_web::", stringify!($krate), "::{use_handle, use_request};")]
        ///
        /// mod api {
        ///     use musli::{Decode, Encode};
        ///     use musli_web::api;
        ///
        ///     #[derive(Encode, Decode)]
        ///     pub struct HelloRequest<'de> {
        ///         pub message: &'de str,
        ///     }
        ///
        ///     #[derive(Encode, Decode)]
        ///     pub struct HelloResponse<'de> {
        ///         pub message: &'de str,
        ///     }
        ///
        ///     api::define! {
        ///         pub type Hello;
        ///
        ///         impl Endpoint for Hello {
        ///             impl<'de> Request for HelloRequest<'de>;
        ///             type Response<'de> = HelloResponse<'de>;
        ///         }
        ///     }
        /// }
        ///
        /// #[function_component(Hello)]
        /// fn hello() -> Html {
        ///     let handle = use_handle();
        ///     let request = use_request();
        ///     let response = use_state(String::new);
        ///
        ///     let onclick = {
        ///         let response = response.clone();
        ///
        ///         move |_: MouseEvent| {
        ///             let response = response.clone();
        ///
        ///             request.set(
        ///                 handle
        ///                     .request()
        ///                     .body(api::HelloRequest { message: "Hello!" })
        ///                     .on_packet(move |packet: Result<ws::Packet<api::Hello>, ws::Error>| {
        ///                         if let Ok(r) = packet.and_then(|p| Ok(p.decode()?.message.to_owned())) {
        ///                             response.set(r);
        ///                         }
        ///                     })
        ///                     .send(),
        ///             );
        ///         }
        ///     };
        ///
        ///     html! {
        ///         <>
        ///             <button {onclick}>{"Say hello"}</button>
        ///             <p>{(*response).clone()}</p>
        ///         </>
        ///     }
        /// }
        /// ```
        pub fn use_request() -> impl Hook<Output = UseRequestHandle> {
            struct HookProvider;

            impl Hook for HookProvider {
                type Output = UseRequestHandle;

                fn run(self, ctx: &mut HookContext) -> Self::Output {
                    UseRequestHandle {
                        inner: use_mut_ref(Request::new).run(ctx),
                    }
                }
            }

            HookProvider
        }

        /// The handle returned by [`use_request`].
        ///
        /// Cloning it is cheap and every clone refers to the same stored
        /// request, so it can be moved into callbacks.
        #[derive(Clone)]
        pub struct UseRequestHandle {
            inner: Rc<RefCell<Request>>,
        }

        impl UseRequestHandle {
            /// Store `request`, cancelling the request stored before it.
            pub fn set(&self, request: Request) {
                let old = mem::replace(&mut *self.inner.borrow_mut(), request);
                // NB: Dropped after the borrow ends, since cancelling the old
                // request drops its callback.
                drop(old);
            }

            /// Cancel the stored request, if any.
            pub fn clear(&self) {
                self.set(Request::new());
            }

            /// Whether the stored request is still waiting for its response.
            ///
            /// This does not make the component render again by itself; keep
            /// track of what the response callback was told for that.
            pub fn is_pending(&self) -> bool {
                self.inner.borrow().is_pending()
            }
        }

        /// Handles are equal if they refer to the same stored request.
        impl PartialEq for UseRequestHandle {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                Rc::ptr_eq(&self.inner, &other.inner)
            }
        }

        impl fmt::Debug for UseRequestHandle {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("UseRequestHandle")
                    .field("is_pending", &self.is_pending())
                    .finish()
            }
        }

        /// Keep a set of [`ws::Requests`] for as long as the component is
        /// mounted.
        ///
        /// This is [`use_request`] for a component which keeps several
        /// requests in flight at once: storing a request with
        /// [`UseRequestsHandle::push`] leaves the requests stored before it
        /// alone, and each one is forgotten once it completes. Every request
        /// still pending is cancelled when the component is unmounted.
        ///
        /// [`ws::Requests`]: crate::web::Requests
        ///
        /// # Examples
        ///
        #[doc = concat!("```\n# extern crate ", stringify!($krate), " as yew;")]
        /// use yew::prelude::*;
        /// use musli_web::web03::prelude::*;
        #[doc = concat!("use musli_web::", stringify!($krate), "::{use_handle, use_requests};")]
        ///
        /// mod api {
        ///     use musli::{Decode, Encode};
        ///     use musli_web::api;
        ///
        ///     #[derive(Encode, Decode)]
        ///     pub struct LikeRequest {
        ///         pub id: u32,
        ///     }
        ///
        ///     api::define! {
        ///         pub type Like;
        ///
        ///         impl Endpoint for Like {
        ///             impl Request for LikeRequest;
        ///             type Response<'de> = ();
        ///         }
        ///     }
        /// }
        ///
        /// #[function_component(Like)]
        /// fn like() -> Html {
        ///     let handle = use_handle();
        ///     let requests = use_requests();
        ///
        ///     let onclick = move |_: MouseEvent| {
        ///         // Clicking again while an earlier like is in flight sends
        ///         // another one, without cancelling the first.
        ///         requests.push(
        ///             handle
        ///                 .request()
        ///                 .body(api::LikeRequest { id: 42 })
        ///                 .on_packet(|packet: Result<ws::Packet<api::Like>, ws::Error>| {
        ///                     if let Err(error) = packet {
        ///                         tracing::error!("Like failed: {error}");
        ///                     }
        ///                 })
        ///                 .send(),
        ///         );
        ///     };
        ///
        ///     html! {
        ///         <button {onclick}>{"Like"}</button>
        ///     }
        /// }
        /// ```
        pub fn use_requests() -> impl Hook<Output = UseRequestsHandle> {
            struct HookProvider;

            impl Hook for HookProvider {
                type Output = UseRequestsHandle;

                fn run(self, ctx: &mut HookContext) -> Self::Output {
                    UseRequestsHandle {
                        inner: use_mut_ref(Requests::new).run(ctx),
                    }
                }
            }

            HookProvider
        }

        /// The handle returned by [`use_requests`].
        ///
        /// Cloning it is cheap and every clone refers to the same set of
        /// requests, so it can be moved into callbacks.
        #[derive(Clone)]
        pub struct UseRequestsHandle {
            inner: Rc<RefCell<Requests>>,
        }

        impl UseRequestsHandle {
            /// Keep `request` until it completes, see [`Requests::push`].
            ///
            /// [`Requests::push`]: crate::web::Requests::push
            pub fn push(&self, request: Request) {
                self.inner.borrow_mut().push(request);
            }

            /// Cancel every pending request.
            pub fn clear(&self) {
                let old = mem::take(&mut *self.inner.borrow_mut());
                // NB: Dropped after the borrow ends, since cancelling the old
                // requests drops their callbacks.
                drop(old);
            }

            /// The number of requests which are still pending.
            ///
            /// This does not make the component render again by itself; keep
            /// track of what the response callbacks were told for that.
            pub fn len(&self) -> usize {
                self.inner.borrow().len()
            }

            /// Whether no request is pending.
            pub fn is_empty(&self) -> bool {
                self.inner.borrow().is_empty()
            }
        }

        /// Handles are equal if they refer to the same set of requests.
        impl PartialEq for UseRequestsHandle {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                Rc::ptr_eq(&self.inner, &other.inner)
            }
        }

        impl fmt::Debug for UseRequestsHandle {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("UseRequestsHandle")
                    .field("len", &self.len())
                    .finish()
            }
        }

        /// Keep a set of [`ws::KeyedRequests`] for as long as the component
        /// is mounted.
        ///
        /// This keeps at most one request in flight for each key: storing a
        /// request with [`UseKeyedRequestsHandle::insert`] cancels the request
        /// stored under the same key before it, and leaves the other keys
        /// alone. Each request is forgotten once it completes, and every
        /// request still pending is cancelled when the component is
        /// unmounted.
        ///
        /// [`ws::KeyedRequests`]: crate::web::KeyedRequests
        ///
        /// # Examples
        ///
        #[doc = concat!("```\n# extern crate ", stringify!($krate), " as yew;")]
        /// # extern crate web_sys03 as web_sys;
        /// use web_sys::HtmlInputElement;
        /// use yew::prelude::*;
        /// use musli_web::web03::prelude::*;
        #[doc = concat!("use musli_web::", stringify!($krate), "::{use_handle, use_keyed_requests};")]
        ///
        /// mod api {
        ///     use musli::{Decode, Encode};
        ///     use musli_web::api;
        ///
        ///     #[derive(Encode, Decode)]
        ///     pub struct ToggleRequest {
        ///         pub id: u32,
        ///         pub on: bool,
        ///     }
        ///
        ///     api::define! {
        ///         pub type Toggle;
        ///
        ///         impl Endpoint for Toggle {
        ///             impl Request for ToggleRequest;
        ///             type Response<'de> = ();
        ///         }
        ///     }
        /// }
        ///
        /// #[derive(Properties, PartialEq)]
        /// struct Props {
        ///     ids: Vec<u32>,
        /// }
        ///
        /// #[function_component(Toggles)]
        /// fn toggles(props: &Props) -> Html {
        ///     let handle = use_handle();
        ///     let requests = use_keyed_requests::<u32>();
        ///
        ///     let toggle = move |id: u32, on: bool| {
        ///         // Toggling an id again cancels its earlier toggle, but a
        ///         // toggle of another id stays in flight.
        ///         requests.insert(
        ///             id,
        ///             handle
        ///                 .request()
        ///                 .body(api::ToggleRequest { id, on })
        ///                 .on_packet(|packet: Result<ws::Packet<api::Toggle>, ws::Error>| {
        ///                     if let Err(error) = packet {
        ///                         tracing::error!("Toggle failed: {error}");
        ///                     }
        ///                 })
        ///                 .send(),
        ///         );
        ///     };
        ///
        ///     html! {
        ///         {for props.ids.iter().copied().map(|id| {
        ///             let toggle = toggle.clone();
        ///
        ///             let onchange = move |e: Event| {
        ///                 toggle(id, e.target_unchecked_into::<HtmlInputElement>().checked())
        ///             };
        ///
        ///             html!(<input type="checkbox" {onchange} />)
        ///         })}
        ///     }
        /// }
        /// ```
        pub fn use_keyed_requests<K>() -> impl Hook<Output = UseKeyedRequestsHandle<K>>
        where
            K: 'static,
        {
            struct HookProvider<K>(core::marker::PhantomData<K>);

            impl<K> Hook for HookProvider<K>
            where
                K: 'static,
            {
                type Output = UseKeyedRequestsHandle<K>;

                fn run(self, ctx: &mut HookContext) -> Self::Output {
                    UseKeyedRequestsHandle {
                        inner: use_mut_ref(KeyedRequests::new).run(ctx),
                    }
                }
            }

            HookProvider(core::marker::PhantomData)
        }

        /// The handle returned by [`use_keyed_requests`].
        ///
        /// Cloning it is cheap and every clone refers to the same set of
        /// requests, so it can be moved into callbacks.
        pub struct UseKeyedRequestsHandle<K> {
            inner: Rc<RefCell<KeyedRequests<K>>>,
        }

        impl<K> UseKeyedRequestsHandle<K>
        where
            K: Eq + core::hash::Hash,
        {
            /// Keep `request` under `key` until it completes, cancelling the
            /// request stored under `key` before it, see
            /// [`KeyedRequests::insert`].
            ///
            /// [`KeyedRequests::insert`]: crate::web::KeyedRequests::insert
            pub fn insert(&self, key: K, request: Request) {
                self.inner.borrow_mut().insert(key, request);
            }

            /// Cancel the request stored under `key`, returning whether it
            /// was still pending.
            pub fn remove<Q>(&self, key: &Q) -> bool
            where
                K: core::borrow::Borrow<Q>,
                Q: ?Sized + Eq + core::hash::Hash,
            {
                self.inner.borrow_mut().remove(key)
            }

            /// Whether the request stored under `key` is still pending.
            ///
            /// This does not make the component render again by itself; keep
            /// track of what the response callbacks were told for that.
            pub fn is_pending<Q>(&self, key: &Q) -> bool
            where
                K: core::borrow::Borrow<Q>,
                Q: ?Sized + Eq + core::hash::Hash,
            {
                self.inner.borrow().is_pending(key)
            }
        }

        impl<K> UseKeyedRequestsHandle<K> {
            /// Cancel every pending request.
            pub fn clear(&self) {
                let old = mem::take(&mut *self.inner.borrow_mut());
                // NB: Dropped after the borrow ends, since cancelling the old
                // requests drops their callbacks.
                drop(old);
            }

            /// The number of keys whose request is still pending.
            pub fn len(&self) -> usize {
                self.inner.borrow().len()
            }

            /// Whether no request is pending.
            pub fn is_empty(&self) -> bool {
                self.inner.borrow().is_empty()
            }
        }

        impl<K> Clone for UseKeyedRequestsHandle<K> {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    inner: self.inner.clone(),
                }
            }
        }

        /// Handles are equal if they refer to the same set of requests.
        impl<K> PartialEq for UseKeyedRequestsHandle<K> {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                Rc::ptr_eq(&self.inner, &other.inner)
            }
        }

        impl<K> fmt::Debug for UseKeyedRequestsHandle<K>
        where
            K: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple("UseKeyedRequestsHandle")
                    .field(&*self.inner.borrow())
                    .finish()
            }
        }

        /// Open a [`ws::Channel`] through the [`ws::Handle`] from
        /// [`use_handle`], for as long as the component is mounted.
        ///
        /// The channel opens again by itself every time the connection does,
        /// and the component renders again every time it opens, closes or
        /// fails to open. Since channels [compare equal] by their current id, an
        /// effect which depends on the channel runs again every time it opens,
        /// which is the point at which to send the requests which need it:
        ///
        /// ```text
        /// use_effect_with(channel.clone(), move |channel| {
        ///     if channel.is_open() {
        ///         request.set(channel.request()./* ... */.send());
        ///     }
        /// });
        /// ```
        ///
        /// A request made in response to the user can instead be queued with
        /// [`Channel::when_open`], so that it is sent even if the channel
        /// happens to be closed at the time.
        ///
        /// A new channel is opened if the provided handle changes. The channel
        /// is closed when the component is unmounted and every clone of it has
        /// been dropped. To be told why a channel failed to open, open it with
        /// [`Handle::channel`] and [`ChannelBuilder::on_change`] instead.
        ///
        /// # Panics
        ///
        /// Panics if the component is not below a [`HandleProvider`].
        ///
        /// [`ws::Channel`]: crate::web::Channel
        /// [`ws::Handle`]: crate::web::Handle
        /// [compare equal]: crate::web::Channel#impl-PartialEq-for-Channel%3CH%3E
        /// [`Channel::when_open`]: crate::web::Channel::when_open
        /// [`Handle::channel`]: crate::web::Handle::channel
        /// [`ChannelBuilder::on_change`]: crate::web::ChannelBuilder::on_change
        ///
        /// # Examples
        ///
        #[doc = concat!("```\n# extern crate ", stringify!($krate), " as yew;")]
        /// use yew::prelude::*;
        /// use musli_web::web03::prelude::*;
        #[doc = concat!("use musli_web::", stringify!($krate), "::{use_channel, use_request};")]
        ///
        /// mod api {
        ///     use musli::{Decode, Encode};
        ///     use musli_web::api;
        ///
        ///     #[derive(Encode, Decode)]
        ///     pub struct Subscribe;
        ///
        ///     #[derive(Encode, Decode)]
        ///     pub struct Subscribed {
        ///         pub count: u32,
        ///     }
        ///
        ///     api::define! {
        ///         pub type Counter;
        ///
        ///         impl Endpoint for Counter {
        ///             impl Request for Subscribe;
        ///             type Response<'de> = Subscribed;
        ///         }
        ///     }
        /// }
        ///
        /// #[function_component(Counter)]
        /// fn counter() -> Html {
        ///     let channel = use_channel();
        ///     let request = use_request();
        ///     let count = use_state(|| None::<u32>);
        ///
        ///     {
        ///         let count = count.clone();
        ///
        ///         use_effect_with(channel.clone(), move |channel| {
        ///             if channel.is_open() {
        ///                 request.set(
        ///                     channel
        ///                         .request()
        ///                         .body(api::Subscribe)
        ///                         .on_packet(move |packet: Result<ws::Packet<api::Counter>, ws::Error>| {
        ///                             if let Ok(response) = packet.and_then(|p| p.decode()) {
        ///                                 count.set(Some(response.count));
        ///                             }
        ///                         })
        ///                         .send(),
        ///                 );
        ///             }
        ///         });
        ///     }
        ///
        ///     match (channel.is_open(), *count) {
        ///         (false, _) => html!(<p>{"Connecting..."}</p>),
        ///         (true, None) => html!(<p>{"Loading..."}</p>),
        ///         (true, Some(count)) => html!(<p>{format!("Count: {count}")}</p>),
        ///     }
        /// }
        /// ```
        pub fn use_channel() -> impl Hook<Output = Channel> {
            struct HookProvider;

            impl Hook for HookProvider {
                type Output = Channel;

                fn run(self, ctx: &mut HookContext) -> Self::Output {
                    let handle = use_handle().run(ctx);
                    let update = use_force_update().run(ctx);

                    let channel = use_memo(handle, move |handle| {
                        handle
                            .channel()
                            .on_change(move |_: ChannelEvent| update.force_update())
                            .open()
                    })
                    .run(ctx);

                    Channel::clone(&channel)
                }
            }

            HookProvider
        }

        /// Follow whether the connection of the [`ws::Handle`] from
        /// [`use_handle`] is open.
        ///
        /// This is `Some` with the connection's [generation] while it is open,
        /// and `None` while it is not. The component renders again when it
        /// changes, so an effect which depends on it runs again every time the
        /// connection opens, which is the point at which to fetch what the
        /// component shows again, see [refetching after a reconnect].
        ///
        /// # Panics
        ///
        /// Panics if the component is not below a [`HandleProvider`].
        ///
        /// [`ws::Handle`]: crate::web::Handle
        /// [generation]: crate::web::Handle::generation
        /// [refetching after a reconnect]: crate::web#refetching-after-a-reconnect
        ///
        /// # Examples
        ///
        #[doc = concat!("```\n# extern crate ", stringify!($krate), " as yew;")]
        /// use yew::prelude::*;
        #[doc = concat!("use musli_web::", stringify!($krate), "::{use_handle, use_open, use_request};")]
        ///
        /// #[function_component(Status)]
        /// fn status() -> Html {
        ///     let handle = use_handle();
        ///     let open = use_open();
        ///     let request = use_request();
        ///
        ///     use_effect_with(open, move |open| {
        ///         if open.is_some() {
        ///             // Send the requests this component shows the result of
        ///             // with `handle.request()` and store them in `request`.
        ///         }
        ///     });
        ///
        ///     match open {
        ///         Some(generation) => html!(<p>{format!("Connection #{generation}")}</p>),
        ///         None => html!(<p>{"Disconnected"}</p>),
        ///     }
        /// }
        /// ```
        pub fn use_open() -> impl Hook<Output = Option<u64>> {
            struct HookProvider;

            impl Hook for HookProvider {
                type Output = Option<u64>;

                fn run(self, ctx: &mut HookContext) -> Self::Output {
                    let handle = use_handle().run(ctx);
                    let update = use_force_update().run(ctx);

                    let state = use_memo(handle, move |handle| {
                        let open = Rc::new(Cell::new(None));

                        let (state, listener) = handle.on_state_change({
                            let open = open.clone();
                            let handle = handle.clone();

                            move |state: State| {
                                let new = state.is_open().then(|| handle.generation());

                                if open.replace(new) != new {
                                    update.force_update();
                                }
                            }
                        });

                        open.set(state.is_open().then(|| handle.generation()));
                        (open, listener)
                    })
                    .run(ctx);

                    state.0.get()
                }
            }

            HookProvider
        }

        mod sealed {
            pub trait Sealed {}
            impl<C> Sealed for super::Context<C> where C: super::Component {}
        }

        /// Extension methods to get the [`ws::Handle`] provided by a
        /// [`HandleProvider`] in a struct component.
        ///
        /// These look the handle up when called and are not told if the
        /// provided handle changes, which suits the usual case of a handle
        /// which the root component keeps for as long as the application runs.
        /// To be told about changes, use [`Scope::context`] instead.
        ///
        /// [`ws::Handle`]: crate::web::Handle
        #[doc = concat!("[`Scope::context`]: ", stringify!($krate), "::html::Scope::context")]
        ///
        /// # Examples
        ///
        #[doc = concat!("```\n# extern crate ", stringify!($krate), " as yew;")]
        /// use yew::prelude::*;
        /// use musli_web::web03::prelude::*;
        #[doc = concat!("use musli_web::", stringify!($krate), "::ContextExt;")]
        ///
        /// struct Status {
        ///     handle: ws::Handle,
        /// }
        ///
        /// impl Component for Status {
        ///     type Message = ();
        ///     type Properties = ();
        ///
        ///     fn create(ctx: &Context<Self>) -> Self {
        ///         Self {
        ///             handle: ctx.handle(),
        ///         }
        ///     }
        ///
        ///     fn view(&self, _: &Context<Self>) -> Html {
        ///         html!(<p>{format!("Connected {} time(s)", self.handle.generation())}</p>)
        ///     }
        /// }
        /// ```
        pub trait ContextExt: sealed::Sealed {
            /// Get the [`ws::Handle`] provided by the closest
            /// [`HandleProvider`] above the component.
            ///
            /// # Panics
            ///
            /// Panics if the component is not below a [`HandleProvider`]. Use
            /// [`ContextExt::try_handle`] to get an `Option` instead.
            ///
            /// [`ws::Handle`]: crate::web::Handle
            fn handle(&self) -> Handle;

            /// Get the [`ws::Handle`] provided by the closest
            /// [`HandleProvider`] above the component, or `None` if there is
            /// none.
            ///
            /// [`ws::Handle`]: crate::web::Handle
            fn try_handle(&self) -> Option<Handle>;
        }

        impl<C> ContextExt for Context<C>
        where
            C: Component,
        {
            #[inline]
            fn handle(&self) -> Handle {
                match self.try_handle() {
                    Some(handle) => handle,
                    None => panic!("{MISSING_PROVIDER}"),
                }
            }

            #[inline]
            fn try_handle(&self) -> Option<Handle> {
                let (handle, _) = self.link().context::<Handle>(Callback::noop())?;
                Some(handle)
            }
        }
    };
}
