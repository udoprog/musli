//! Integration with yew `0.21.x`.
//!
//! # Examples
//!
//! This example uses [`yew021`]:
//!
//! [`yew021`]: crate::yew021
//!
//! ```
//! # extern crate yew021 as yew;
//! # extern crate web_sys03 as web_sys;
//! use web_sys::HtmlInputElement;
//! use yew::prelude::*;
//! use musli_web::yew021::prelude::*;
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
//!     #[derive(Encode, Decode)]
//!     pub struct TickEvent<'de> {
//!         pub message: &'de str,
//!         pub tick: u32,
//!     }
//!  
//!     api::define! {
//!         endpoint Hello {
//!             request<'de> = HelloRequest<'de>;
//!             response<'de> = HelloResponse<'de>;
//!         }
//!    
//!         broadcast Tick {
//!             body<'de> = TickEvent<'de>;
//!         }
//!     }
//! }
//!
//! enum Msg {
//!     Error(ws::Error),
//!     Change(String),
//!     Send,
//!     HelloResponse(Result<ws::Packet<api::Hello>, ws::Error>),
//!     Tick(Result<ws::Packet<api::Tick>, ws::Error>),
//! }
//!
//! struct App {
//!     service: ws::Service,
//!     _listen: ws::Listener,
//!     request: ws::Request,
//!     text: String,
//!     tick: u32,
//!     responses: Vec<String>,
//! }
//!
//! impl Component for App {
//!     type Message = Msg;
//!     type Properties = ();
//!
//!     fn create(ctx: &Context<Self>) -> Self {
//!         let service = ws::connect(ws::Connect::location("ws"))
//!             .on_error(ctx.link().callback(Msg::Error))
//!             .build();
//!
//!         service.connect();
//!
//!         let listen = service.handle().listen(ctx.link().callback(Msg::Tick));
//!
//!         Self {
//!             service,
//!             _listen: listen,
//!             request: ws::Request::new(),
//!             text: String::new(),
//!             tick: 0,
//!             responses: Vec::new(),
//!         }
//!     }
//!
//!     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//!         match msg {
//!             Msg::Error(error) => {
//!                 tracing::error!("WebSocket error: {:?}", error);
//!                 false
//!             }
//!             Msg::Change(text) => {
//!                 self.text = text;
//!                 true
//!             }
//!             Msg::Send => {
//!                 self.request = self
//!                     .service
//!                     .handle()
//!                     .request()
//!                     .body(api::HelloRequest {
//!                         message: self.text.as_str(),
//!                     })
//!                     .on_packet(
//!                         ctx.link()
//!                             .callback(|result: Result<_, _>| Msg::HelloResponse(result)),
//!                     )
//!                     .send();
//!
//!                 self.text.clear();
//!                 true
//!             }
//!             Msg::HelloResponse(Err(error)) => {
//!                 tracing::error!("Request error: {:?}", error);
//!                 false
//!             }
//!             Msg::HelloResponse(Ok(packet)) => {
//!                 tracing::debug!("Got response");
//!
//!                 while !packet.is_empty() {
//!                     let Ok(response) = packet.decode() else {
//!                         break;
//!                     };
//!
//!                     self.responses.push(response.message.to_owned());
//!                 }
//!
//!                 true
//!             }
//!             Msg::Tick(Err(error)) => {
//!                 tracing::error!("Tick error: {error}");
//!                 false
//!             }
//!             Msg::Tick(Ok(packet)) => {
//!                 tracing::debug!("Got tick");
//!
//!                 if let Ok(tick) = packet.decode_broadcast() {
//!                     self.tick = tick.tick;
//!                 }
//!
//!                 true
//!             }
//!         }
//!     }
//!
//!     fn view(&self, ctx: &Context<Self>) -> Html {
//!         let oninput = ctx.link().callback(|e: InputEvent| {
//!             let input = e.target_unchecked_into::<HtmlInputElement>();
//!             Msg::Change(input.value())
//!         });
//!
//!         let onclick = ctx.link().callback(|_: MouseEvent| {
//!             Msg::Send
//!         });
//!
//!         html! {
//!             <div class="container">
//!                 <input type="text" {oninput} value={self.text.clone()} />
//!                 <button {onclick}>{"Send Message"}</button>
//!                 {for self.responses.iter().enumerate().map(|(index, response)| html!(<div>{format!("Response #{index}: {response}")}</div>))}
//!                 <div>{format!("Global tick: {}", self.tick)}</div>
//!             </div>
//!         }
//!     }
//! }
//! ```

use yew021::Callback;
use yew021::html::ImplicitClone;

pub mod prelude {
    //! The public facing API for use with yew `0.2.1` and web-sys `0.3.x`.

    pub use crate::yew021::{HandleExt, RequestBuilderExt, ServiceBuilderExt};

    pub mod ws {
        #[doc(inline)]
        pub use crate::web03::prelude::ws::*;
    }
}

use crate::api;
use crate::web::{
    Error, Handle, Listener, Packet, RawPacket, RequestBuilder, ServiceBuilder, State,
    StateListener, WebImpl,
};

impl<H> ImplicitClone for Handle<H>
where
    H: WebImpl,
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
    H: WebImpl,
{
    #[inline]
    fn on_error(self, callback: Callback<Error>) -> Self {
        ServiceBuilder::on_error_cb(self, move |error| callback.emit(error))
    }
}

/// Request builder extension for interacting with handles in yew `0.21.x`.
pub trait HandleExt {
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
    ///     Tick(Result<ws::Packet<api::Tick>, ws::Error>),
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
    ///             Msg::Tick(Err(error)) => {
    ///                 tracing::error!("Tick error: {error}");
    ///                 false
    ///             }
    ///             Msg::Tick(Ok(packet)) => {
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
    fn listen<T>(&self, callback: Callback<Result<Packet<T>, Error>>) -> Listener
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
    fn on_state_change(&self, callback: Callback<State>) -> (State, StateListener);
}

impl<H> HandleExt for Handle<H>
where
    H: WebImpl,
{
    #[inline]
    fn listen<T>(&self, callback: Callback<Result<Packet<T>, Error>>) -> Listener
    where
        T: api::Listener,
    {
        Handle::listen_cb::<T>(self, move |result| match result {
            Ok(packet) => callback.emit(Ok(Packet::new(packet))),
            Err(error) => callback.emit(Err(error)),
        })
    }

    #[inline]
    fn on_state_change(&self, callback: Callback<State>) -> (State, StateListener) {
        Handle::on_state_change_cb(self, move |state| callback.emit(state))
    }
}

/// Request builder extension for interacting with request builders in yew `0.21.x`.
pub trait RequestBuilderExt<'a, H, B, C>
where
    Self: Sized,
    H: WebImpl,
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
    ///             .request()
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
    fn on_packet<E>(
        self,
        f: Callback<Result<Packet<E>, Error>>,
    ) -> RequestBuilder<'a, H, B, Callback<Result<Packet<E>, Error>>>
    where
        E: api::Endpoint;
}

impl<'a, H, B, C> RequestBuilderExt<'a, H, B, C> for RequestBuilder<'a, H, B, C>
where
    H: WebImpl,
{
    #[inline]
    fn on_packet<E>(
        self,
        callback: Callback<Result<Packet<E>, Error>>,
    ) -> RequestBuilder<'a, H, B, Callback<Result<Packet<E>, Error>>>
    where
        E: api::Endpoint,
    {
        RequestBuilder::on_raw_packet(self, callback)
    }
}

impl<E> crate::web::Callback<Result<RawPacket, Error>> for Callback<Result<Packet<E>, Error>>
where
    E: api::Endpoint,
{
    #[inline]
    fn call(&self, result: Result<RawPacket, Error>) {
        match result {
            Ok(raw) => {
                self.emit(Ok(Packet::new(raw)));
            }
            Err(error) => {
                self.emit(Err(error));
            }
        }
    }
}

impl crate::web::Callback<Result<RawPacket, Error>> for Callback<Result<RawPacket, Error>> {
    #[inline]
    fn call(&self, result: Result<RawPacket, Error>) {
        self.emit(result);
    }
}
