//! Integration with yew `0.22.x`.
//!
//! [`Callback`] can be used wherever this crate takes a callback, and the
//! [`ws::Handle`] of a service can be provided to every component in an
//! application.
//!
//! # Providing the handle
//!
//! Build and connect the [`ws::Service`] in the root component and keep it for
//! as long as the application runs. Wrap the component tree in a
//! [`HandleProvider`] with its handle, and the components below it can get the
//! handle without it being passed through their properties:
//!
//! * [`use_handle`] gets the handle in a function component.
//! * [`use_request`] keeps a [`ws::Request`] for as long as the component is
//!   mounted, cancelling the previous one when a new one is stored.
//! * [`use_channel`] opens a [`ws::Channel`] for as long as the component is
//!   mounted, and renders the component again whenever it opens or closes.
//! * [`use_open`] follows whether the connection is open, so that what a
//!   component shows can be fetched again after a reconnect.
//! * [`ContextExt::handle`] gets the handle in a struct component.
//!
//! The hooks and [`ContextExt::handle`] panic with a message naming
//! [`HandleProvider`] if the component is not below one.
//!
//! [`Callback`]: yew022::Callback
//! [`ws::Handle`]: crate::web::Handle
//! [`ws::Service`]: crate::web::Service
//! [`ws::Request`]: crate::web::Request
//! [`ws::Channel`]: crate::web::Channel
//!
//! ```
//! # extern crate yew022 as yew;
//! use yew::prelude::*;
//! use musli_web::web03::prelude::*;
//! use musli_web::yew022::{HandleProvider, use_handle, use_open};
//!
//! #[function_component(App)]
//! fn app() -> Html {
//!     // The service is built once and kept for as long as the app runs.
//!     let service = use_memo((), |_| {
//!         let service = ws::connect(ws::Connect::location("ws"))
//!             .on_error(|error| tracing::error!("WebSocket error: {error}"))
//!             .build();
//!
//!         service.connect();
//!         service
//!     });
//!
//!     html! {
//!         <HandleProvider context={service.handle().clone()}>
//!             <Status />
//!         </HandleProvider>
//!     }
//! }
//!
//! #[function_component(Status)]
//! fn status() -> Html {
//!     let handle = use_handle();
//!     let open = use_open();
//!
//!     match open {
//!         Some(generation) => html!(<p>{format!("Connection #{generation} uses {:?}", handle.format())}</p>),
//!         None => html!(<p>{"Disconnected"}</p>),
//!     }
//! }
//! ```
//!
//! # Struct components
//!
//! This example uses [`yew022`] with a struct component which owns the
//! service itself:
//!
//! [`yew022`]: crate::yew022
//!
//! ```
//! # extern crate yew022 as yew;
//! # extern crate web_sys03 as web_sys;
//! use web_sys::HtmlInputElement;
//! use yew::prelude::*;
//! use musli_web::web03::prelude::*;
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
//!         pub type Hello;
//!
//!         impl Endpoint for Hello {
//!             impl<'de> Request for HelloRequest<'de>;
//!             type Response<'de> = HelloResponse<'de>;
//!         }
//!
//!         pub type Tick;
//!
//!         impl Broadcast for Tick {
//!             impl<'de> Event for TickEvent<'de>;
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
//!         let listen = service.handle().on_broadcast(ctx.link().callback(Msg::Tick));
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
//!                     .on_packet(ctx.link().callback(Msg::HelloResponse))
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
//!                 if let Ok(tick) = packet.decode_event() {
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

yew_impl!(yew022);
