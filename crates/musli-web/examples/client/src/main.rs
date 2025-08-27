//! Example musli-web client based on [`yew`].
//!
//! Run with:
//!
//! ```sh
//! trunk serve
//! ```
//!
//! [`yew`]: https://yew.rs

use musli_web::yew021 as ws;
use web_sys::HtmlInputElement;
use yew::prelude::*;

enum Msg {
    Error(ws::Error),
    WebSocket(ws::Msg),
    Send,
    HelloResponse(Result<ws::Packet<api::Hello>, ws::Error>),
    Tick(ws::Packet<api::Tick>),
}

impl From<ws::Error> for Msg {
    #[inline]
    fn from(error: ws::Error) -> Self {
        Msg::Error(error)
    }
}

impl From<ws::Msg> for Msg {
    #[inline]
    fn from(error: ws::Msg) -> Self {
        Msg::WebSocket(error)
    }
}

struct App {
    service: ws::Service<Self>,
    handle: ws::Handle,
    input: NodeRef,
    _listen: ws::Listener<api::Tick>,
    request: ws::Request,
    response: String,
    tick: u32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (mut service, handle) =
            ws::Service::new(ctx, ws::Connect::location_with_path(String::from("/ws")));
        let input = NodeRef::default();

        service.connect();

        let listen = handle.listen(ctx.link().callback(Msg::Tick));

        Self {
            service,
            handle,
            input,
            _listen: listen,
            request: ws::Request::empty(),
            response: String::new(),
            tick: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Error(error) => {
                log::error!("WebSocket error: {:?}", error);
                false
            }
            Msg::WebSocket(msg) => {
                self.service.update(msg);
                false
            }
            Msg::Send => {
                let Some(input) = self.input.cast::<HtmlInputElement>() else {
                    return false;
                };

                let value = input.value();
                input.set_value("");

                self.request = self
                    .handle
                    .request::<api::Hello>()
                    .body(api::HelloRequest {
                        message: value.as_str(),
                    })
                    .on_packet(ctx.link().callback(Msg::HelloResponse))
                    .send();

                true
            }
            Msg::HelloResponse(Err(error)) => {
                log::error!("Request error: {:?}", error);
                false
            }
            Msg::HelloResponse(Ok(packet)) => {
                log::warn!("Got response");

                if let Ok(response) = packet.decode() {
                    self.response = response.message.to_owned();
                }

                true
            }
            Msg::Tick(packet) => {
                if let Ok(tick) = packet.decode_broadcast() {
                    self.tick = tick.tick;
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_: MouseEvent| Msg::Send);

        html! {
            <div class="container">
                <input type="text" ref={self.input.clone()} />
                <button {onclick}>{"Send Message"}</button>
                <div>{format!("Response: {}", self.response)}</div>
                <div>{format!("Global tick: {}", self.tick)}</div>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Started up");
    yew::Renderer::<App>::new().render();
}
