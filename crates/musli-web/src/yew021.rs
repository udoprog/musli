//! Prelude to import when integrating with yew `0.21.x`.

use yew021::Callback;
use yew021::html::ImplicitClone;

pub mod prelude {
    //! The public facing API for use with yew `0.2.1` and web-sys `0.3.x`.

    pub use crate::yew021::{HandleExt, RequestBuilderExt, ServiceBuilderExt};

    pub mod ws {
        //! Organization module prefixing all exported items with `ws` for
        //! convenient namespacing.

        pub use crate::web::{Connect, Error, State};
        use crate::web03::Implementation;

        /// Implementation alias for [`connect`].
        ///
        /// [`connect`]: crate::web03::connect
        pub fn connect(connect: Connect) -> ServiceBuilder {
            crate::web03::connect(connect)
        }

        /// Implementation alias for [`Service`].
        ///
        /// [`Service`]: crate::web::Service
        pub type Service = crate::web::Service<Implementation>;

        /// Implementation alias for [`Request`].
        ///
        /// [`Request`]: crate::web::Request
        pub type Request = crate::web::Request<Implementation>;

        /// Implementation alias for [`Handle`].
        ///
        /// [`Handle`]: crate::web::Handle
        pub type Handle = crate::web::Handle<Implementation>;

        /// Implementation alias for [`Listener`].
        ///
        /// [`Listener`]: crate::web::Listener
        pub type Listener = crate::web::Listener<Implementation>;

        /// Implementation alias for [`Packet`].
        ///
        /// [`Packet`]: crate::web::Packet
        pub type Packet<T> = crate::web::Packet<T, Implementation>;

        /// Implementation alias for [`RawPacket`].
        ///
        /// [`RawPacket`]: crate::web::RawPacket
        pub type RawPacket = crate::web::RawPacket<Implementation>;

        /// Implementation alias for [`RequestBuilder`].
        ///
        /// [`RequestBuilder`]: crate::web::RequestBuilder
        pub type RequestBuilder<'a, E, T> = crate::web::RequestBuilder<'a, E, T, Implementation>;

        /// Implementation alias for [`ServiceBuilder`].
        ///
        /// [`ServiceBuilder`]: crate::web::ServiceBuilder
        pub type ServiceBuilder = crate::web::ServiceBuilder<Implementation>;

        /// Implementation alias for [`StateListener`].
        ///
        /// [`StateListener`]: crate::web::StateListener
        pub type StateListener = crate::web::StateListener<Implementation>;
    }
}

use crate::api;
use crate::web::{
    Error, Handle, Listener, Packet, RawPacket, RequestBuilder, ServiceBuilder, State,
    StateListener, WebImplementation,
};

impl<H> ImplicitClone for Handle<H>
where
    H: WebImplementation,
{
    #[inline]
    fn implicit_clone(&self) -> Self {
        self.clone()
    }
}

/// Service builder extension for interacting with handles in yew `0.21.x`.
pub trait ServiceBuilderExt {
    /// Set the error handler to use for the service.
    fn on_error(self, callback: Callback<Error>) -> Self;
}

impl<H> ServiceBuilderExt for ServiceBuilder<H>
where
    H: WebImplementation,
{
    #[inline]
    fn on_error(self, callback: Callback<Error>) -> Self {
        ServiceBuilder::on_error_cb(self, move |error| callback.emit(error))
    }
}

/// Request builder extension for interacting with handles in yew `0.21.x`.
pub trait HandleExt<H>
where
    H: WebImplementation,
{
    /// Listen for broadcasts of type `T`.
    ///
    /// Returns a handle for the listener that will cancel the listener if
    /// dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::yew021::prelude::*;
    ///
    /// mod api {
    ///     use musli::{Decode, Encode};
    ///     use musli_web::api;
    ///
    ///     #[derive(Encode, Decode)]
    ///     pub struct TickEvent<'de> {
    ///         pub message: &'de str,
    ///         pub tick: u32,
    ///     }
    ///
    ///     api::define! {
    ///         broadcast Tick {
    ///             body<'de> = TickEvent<'de>;
    ///         }
    ///     }
    /// }
    ///
    /// enum Msg {
    ///     Tick(ws::Packet<api::Tick>),
    /// }
    ///
    /// #[derive(Properties, PartialEq)]
    /// struct Props {
    ///     ws: ws::Handle,
    /// }
    ///
    /// struct App {
    ///     tick: u32,
    ///     _listen: ws::Listener,
    /// }
    ///
    /// impl Component for App {
    ///     type Message = Msg;
    ///     type Properties = Props;
    ///
    ///     fn create(ctx: &Context<Self>) -> Self {
    ///         let listen = ctx.props().ws.listen(ctx.link().callback(Msg::Tick));
    ///
    ///         Self {
    ///             tick: 0,
    ///             _listen: listen,
    ///         }
    ///     }
    ///
    ///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    ///         match msg {
    ///             Msg::Tick(packet) => {
    ///                 if let Ok(tick) = packet.decode_broadcast() {
    ///                     self.tick = tick.tick;
    ///                 }
    ///
    ///                 true
    ///             }
    ///         }
    ///     }
    ///
    ///     fn view(&self, ctx: &Context<Self>) -> Html {
    ///         html! {
    ///             <div>
    ///                 <h1>{"WebSocket Example"}</h1>
    ///                 <p>{format!("Tick: {}", self.tick)}</p>
    ///             </div>
    ///         }
    ///     }
    /// }
    /// ```
    fn listen<T>(&self, callback: Callback<Packet<T, H>>) -> Listener<H>
    where
        T: api::Listener;

    /// Listen for state changes to the underlying connection.
    ///
    /// This indicates when the connection is open and ready to receive requests
    /// through [`State::Open`], or if it's closed and requests will be queued
    /// through [`State::Closed`].
    ///
    /// Dropping the returned handle will cancel the listener.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::yew021::prelude::*;
    ///
    /// enum Msg {
    ///     StateChange(ws::State),
    /// }
    ///
    /// #[derive(Properties, PartialEq)]
    /// struct Props {
    ///     ws: ws::Handle,
    /// }
    ///
    /// struct App {
    ///     state: ws::State,
    ///     _listen: ws::StateListener,
    /// }
    ///
    /// impl Component for App {
    ///     type Message = Msg;
    ///     type Properties = Props;
    ///
    ///     fn create(ctx: &Context<Self>) -> Self {
    ///         let (state, listen) = ctx.props().ws.on_state_change(ctx.link().callback(Msg::StateChange));
    ///
    ///         Self {
    ///             state,
    ///             _listen: listen,
    ///         }
    ///     }
    ///
    ///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    ///         match msg {
    ///             Msg::StateChange(state) => {
    ///                 self.state = state;
    ///                 true
    ///             }
    ///         }
    ///     }
    ///
    ///     fn view(&self, ctx: &Context<Self>) -> Html {
    ///         html! {
    ///             <div>
    ///                 <h1>{"WebSocket Example"}</h1>
    ///                 <p>{format!("State: {:?}", self.state)}</p>
    ///             </div>
    ///         }
    ///     }
    /// }
    /// ```
    fn on_state_change(&self, callback: Callback<State>) -> (State, StateListener<H>);
}

impl<H> HandleExt<H> for Handle<H>
where
    H: WebImplementation,
{
    #[inline]
    fn listen<T>(&self, callback: Callback<Packet<T, H>>) -> Listener<H>
    where
        T: api::Listener,
    {
        Handle::listen_cb::<T>(self, move |packet| callback.emit(Packet::new(packet)))
    }

    #[inline]
    fn on_state_change(&self, callback: Callback<State>) -> (State, StateListener<H>) {
        Handle::on_state_change_cb(self, move |state| callback.emit(state))
    }
}

/// Request builder extension for interacting with request builders in yew `0.21.x`.
pub trait RequestBuilderExt<E, H>
where
    Self: Sized,
    E: api::Endpoint,
    H: WebImplementation,
{
    /// Handle the response using the specified callback.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::yew021::prelude::*;
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
    ///         endpoint Hello {
    ///             request<'de> = HelloRequest<'de>;
    ///             response<'de> = HelloResponse<'de>;
    ///         }
    ///     }
    /// }
    ///
    /// enum Msg {
    ///     OnHello(Result<ws::Packet<api::Hello>, ws::Error>),
    /// }
    ///
    /// #[derive(Properties, PartialEq)]
    /// struct Props {
    ///     ws: ws::Handle,
    /// }
    ///
    /// struct App {
    ///     message: String,
    ///     _hello: ws::Request,
    /// }
    ///
    /// impl Component for App {
    ///     type Message = Msg;
    ///     type Properties = Props;
    ///
    ///     fn create(ctx: &Context<Self>) -> Self {
    ///         let hello = ctx.props().ws
    ///             .request::<api::Hello>()
    ///             .body(api::HelloRequest { message: "Hello!"})
    ///             .on_packet(ctx.link().callback(Msg::OnHello))
    ///             .send();
    ///
    ///         Self {
    ///             message: String::from("No Message :("),
    ///             _hello: hello,
    ///         }
    ///     }
    ///
    ///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    ///         match msg {
    ///             Msg::OnHello(Err(error)) => {
    ///                 tracing::error!("Request error: {:?}", error);
    ///                 false
    ///             }
    ///             Msg::OnHello(Ok(packet)) => {
    ///                 if let Ok(response) = packet.decode() {
    ///                     self.message = response.message.to_owned();
    ///                 }
    ///
    ///                 true
    ///             }
    ///         }
    ///     }
    ///
    ///     fn view(&self, ctx: &Context<Self>) -> Html {
    ///         html! {
    ///             <div>
    ///                 <h1>{"WebSocket Example"}</h1>
    ///                 <p>{format!("Message: {}", self.message)}</p>
    ///             </div>
    ///         }
    ///     }
    /// }
    /// ```
    fn on_packet(self, f: Callback<Result<Packet<E, H>, Error>>) -> Self;

    /// Handle the raw response using the specified callback.
    ///
    /// This can be useful if all of your responses are expected to return the
    /// same response type and you only want to have one message to handle all
    /// of them.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::yew021::prelude::*;
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
    ///         endpoint Hello {
    ///             request<'de> = HelloRequest<'de>;
    ///             response<'de> = HelloResponse<'de>;
    ///         }
    ///     }
    /// }
    ///
    /// enum Msg {
    ///     OnHello(Result<ws::RawPacket, ws::Error>),
    /// }
    ///
    /// #[derive(Properties, PartialEq)]
    /// struct Props {
    ///     ws: ws::Handle,
    /// }
    ///
    /// struct App {
    ///     message: String,
    ///     _hello: ws::Request,
    /// }
    ///
    /// impl Component for App {
    ///     type Message = Msg;
    ///     type Properties = Props;
    ///
    ///     fn create(ctx: &Context<Self>) -> Self {
    ///         let hello = ctx.props().ws
    ///             .request::<api::Hello>()
    ///             .body(api::HelloRequest { message: "Hello!"})
    ///             .on_raw_packet(ctx.link().callback(Msg::OnHello))
    ///             .send();
    ///
    ///         Self {
    ///             message: String::from("No Message :("),
    ///             _hello: hello,
    ///         }
    ///     }
    ///
    ///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    ///         match msg {
    ///             Msg::OnHello(Err(error)) => {
    ///                 tracing::error!("Request error: {:?}", error);
    ///                 false
    ///             }
    ///             Msg::OnHello(Ok(packet)) => {
    ///                 if let Ok(response) = packet.decode::<api::HelloResponse>() {
    ///                     self.message = response.message.to_owned();
    ///                 }
    ///
    ///                 true
    ///             }
    ///         }
    ///     }
    ///
    ///     fn view(&self, ctx: &Context<Self>) -> Html {
    ///         html! {
    ///             <div>
    ///                 <h1>{"WebSocket Example"}</h1>
    ///                 <p>{format!("Message: {}", self.message)}</p>
    ///             </div>
    ///         }
    ///     }
    /// }
    /// ```
    fn on_raw_packet(self, f: Callback<Result<RawPacket<H>, Error>>) -> Self;
}

impl<E, T, H> RequestBuilderExt<E, H> for RequestBuilder<'_, E, T, H>
where
    E: api::Endpoint,
    H: WebImplementation,
{
    #[inline]
    fn on_packet(self, callback: Callback<Result<Packet<E, H>, Error>>) -> Self {
        RequestBuilder::on_raw_packet_cb(self, move |result| match result {
            Ok(raw) => {
                callback.emit(Ok(Packet::new(raw)));
            }
            Err(error) => {
                callback.emit(Err(error));
            }
        })
    }

    #[inline]
    fn on_raw_packet(self, callback: Callback<Result<RawPacket<H>, Error>>) -> Self {
        RequestBuilder::on_raw_packet_cb(self, move |result| match result {
            Ok(raw) => {
                callback.emit(Ok(raw));
            }
            Err(error) => {
                callback.emit(Err(error));
            }
        })
    }
}
