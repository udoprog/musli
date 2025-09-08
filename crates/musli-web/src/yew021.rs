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
//!         impl Endpoint for Hello {
//!             impl<'de> Request for HelloRequest<'de>;
//!             type Response<'de> = HelloResponse<'de>;
//!         }
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

use yew021::Callback;
use yew021::html::ImplicitClone;

use crate::web::{Handle, WebImpl};

impl<H> ImplicitClone for Handle<H>
where
    H: WebImpl,
{
    #[inline]
    fn implicit_clone(&self) -> Self {
        self.clone()
    }
}

impl<I> crate::web::Callback<I> for Callback<I>
where
    I: 'static,
{
    #[inline]
    fn call(&self, result: I) {
        self.emit(result);
    }
}
