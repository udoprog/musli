//! Client side implementation for [`web-sys`] `0.3.x`.
//!
//! [`web-sys`]: <https://docs.rs/web-sys/0.3>
//!
//! # Examples
//!
//! This example uses yew:
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
//!     Send,
//!     HelloResponse(Result<ws::Packet<api::Hello>, ws::Error>),
//!     Tick(ws::Packet<api::Tick>),
//! }
//!
//! struct App {
//!     service: ws::Service,
//!     input: NodeRef,
//!     _listen: ws::Listener,
//!     request: ws::Request,
//!     response: String,
//!     tick: u32,
//! }
//!
//! impl Component for App {
//!     type Message = Msg;
//!     type Properties = ();
//!
//!     fn create(ctx: &Context<Self>) -> Self {
//!         let service = ws::connect(ws::Connect::location_with_path(String::from("/ws")))
//!             .on_error(ctx.link().callback(Msg::Error))
//!             .build();
//!
//!         let input = NodeRef::default();
//!
//!         service.connect();
//!
//!         let listen = service.handle().listen(ctx.link().callback(Msg::Tick));
//!
//!         Self {
//!             service,
//!             input,
//!             _listen: listen,
//!             request: ws::Request::empty(),
//!             response: String::new(),
//!             tick: 0,
//!         }
//!     }
//!
//!     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//!         match msg {
//!             Msg::Error(error) => {
//!                 tracing::error!("WebSocket error: {:?}", error);
//!                 false
//!             }
//!             Msg::Send => {
//!                 let Some(input) = self.input.cast::<HtmlInputElement>() else {
//!                     return false;
//!                 };
//!
//!                 let value = input.value();
//!                 input.set_value("");
//!
//!                 self.request = self
//!                     .service
//!                     .handle()
//!                     .request::<api::Hello>()
//!                     .body(api::HelloRequest {
//!                         message: value.as_str(),
//!                     })
//!                     .on_packet(ctx.link().callback(Msg::HelloResponse))
//!                     .send();
//!
//!                 true
//!             }
//!             Msg::HelloResponse(Err(error)) => {
//!                 tracing::error!("Request error: {error}");
//!                 false
//!             }
//!             Msg::HelloResponse(Ok(packet)) => {
//!                 tracing::warn!("Got response");
//!
//!                 if let Ok(response) = packet.decode() {
//!                     self.response = response.message.to_owned();
//!                 }
//!
//!                 true
//!             }
//!             Msg::Tick(packet) => {
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
//!         let onclick = ctx.link().callback(|_: MouseEvent| Msg::Send);
//!
//!         html! {
//!             <div class="container">
//!                 <input type="text" ref={self.input.clone()} />
//!                 <button {onclick}>{"Send Message"}</button>
//!                 <div>{format!("Response: {}", self.response)}</div>
//!                 <div>{format!("Global tick: {}", self.tick)}</div>
//!             </div>
//!         }
//!     }
//! }
//! ```

use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use alloc::rc::Weak;

use wasm_bindgen02::JsCast;
use wasm_bindgen02::closure::Closure;
use web_sys03::js_sys::{ArrayBuffer, Math, Uint8Array};
use web_sys03::{BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket, window};

use crate::web::Location;
use crate::web::{Connect, Error, ServiceBuilder, Shared, WebImplementation};

/// Handles for websocket implementation.
#[doc(hidden)]
pub struct Handles {
    open: Closure<dyn Fn()>,
    close: Closure<dyn Fn(CloseEvent)>,
    message: Closure<dyn Fn(MessageEvent)>,
    error: Closure<dyn Fn(ErrorEvent)>,
}

/// WebSocket implementation for web-sys `0.3.x`.
///
/// See [`connect()`].
#[derive(Clone, Copy)]
pub enum Implementation {}

impl crate::web::sealed::Sealed for Implementation {}

impl WebImplementation for Implementation {
    type Handles = Handles;
    type Socket = WebSocket;

    #[inline]
    fn location() -> Result<Location, Error> {
        let Some(window) = window() else {
            return Err(Error::msg("No window available"));
        };

        let protocol = window.location().protocol()?;

        let protocol = match protocol.as_str() {
            "https:" => "wss:",
            "http:" => "ws:",
            other => {
                return Err(Error::msg(format_args!(
                    "Same host connection is not supported for protocol `{other}`"
                )));
            }
        };

        let host = window.location().hostname()?;
        let port = window.location().port()?;

        Ok(Location {
            protocol: protocol.to_owned(),
            host,
            port,
        })
    }

    #[inline]
    fn random(range: u32) -> u32 {
        ((Math::random() * range as f64).round() as u32).min(range)
    }

    #[inline]
    fn now() -> Option<f64> {
        Some(window()?.performance()?.now())
    }

    #[inline]
    fn new(url: &str, handles: &Self::Handles) -> Result<Self::Socket, Error> {
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);
        ws.set_onopen(Some(handles.open.as_ref().unchecked_ref()));
        ws.set_onclose(Some(handles.close.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(handles.message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(handles.error.as_ref().unchecked_ref()));
        Ok(ws)
    }

    #[inline]
    fn send(socket: &Self::Socket, data: &[u8]) -> Result<(), Error> {
        socket.send_with_u8_array(data)?;
        Ok(())
    }

    #[inline]
    fn close(socket: Self::Socket) -> Result<(), Error> {
        socket.close()?;
        Ok(())
    }

    #[inline]
    fn handles(shared: &Weak<Shared<Self>>) -> Self::Handles {
        let open = {
            let shared = shared.clone();

            Closure::new(move || {
                if let Some(shared) = shared.upgrade() {
                    shared.do_open();
                }
            })
        };

        let close = {
            let shared = shared.clone();

            Closure::new(move |e: CloseEvent| {
                if let Some(shared) = shared.upgrade() {
                    shared.do_close(e);
                }
            })
        };

        let message = {
            let shared = shared.clone();

            Closure::new(move |e: MessageEvent| {
                if let Some(shared) = shared.upgrade() {
                    shared.do_message(e);
                }
            })
        };

        let error = {
            let shared = shared.clone();

            Closure::new(move |e: ErrorEvent| {
                if let Some(shared) = shared.upgrade() {
                    shared.do_error(e);
                }
            })
        };

        Self::Handles {
            open,
            close,
            message,
            error,
        }
    }
}

/// Construct a new [`ServiceBuilder`] associated with the given [`Connect`]
/// strategy.
pub fn connect(connect: Connect) -> ServiceBuilder<Implementation> {
    crate::web::connect::<Implementation>(connect)
}

impl Shared<Implementation> {
    fn do_open(&self) {
        tracing::debug!("Open event");
        self.set_open();
    }

    fn do_close(self: &Rc<Self>, e: CloseEvent) {
        tracing::debug!(code = e.code(), reason = e.reason(), "Close event");
        self.close();
    }

    fn do_message(self: &Rc<Shared<Implementation>>, e: MessageEvent) {
        tracing::debug!("Message event");

        if let Err(error) = self.web03_message(e) {
            self.handle_error(error);
        }
    }

    fn web03_message(self: &Rc<Shared<Implementation>>, e: MessageEvent) -> Result<(), Error> {
        let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() else {
            return Err(Error::msg("Expected message as ArrayBuffer"));
        };

        let array = Uint8Array::new(&array_buffer);
        let needed = array.length() as usize;

        let mut buf = self.next_buffer(needed);

        // SAFETY: We've sized the buffer appropriately above.
        unsafe {
            array.raw_copy_to_ptr(buf.data.as_mut_ptr());
            buf.data.set_len(needed);
        }

        self.message(buf)
    }

    fn do_error(self: &Rc<Self>, e: ErrorEvent) {
        tracing::debug!(message = e.message(), "Error event");
        self.close();
    }
}
