//! Example musli-web client based on [`yew`].
//!
//! Run with:
//!
//! ```sh
//! trunk serve
//! ```
//!
//! [`yew`]: https://yew.rs

use musli_web::yew021::prelude::*;
use tracing::Level;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_wasm::{WASMLayer, WASMLayerConfigBuilder};
use web_sys::HtmlInputElement;
use yew::prelude::*;

enum Msg {
    Error(ws::Error),
    Send,
    HelloResponse(Result<ws::Packet<api::Hello>, ws::Error>),
    Tick(ws::Packet<api::Tick>),
}

struct App {
    service: ws::Service,
    input: NodeRef,
    _listen: ws::Listener,
    request: ws::Request,
    responses: Vec<String>,
    tick: u32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let service = ws::connect(ws::Connect::location_with_path(String::from("/ws")))
            .on_error(ctx.link().callback(Msg::Error))
            .build();

        let input = NodeRef::default();

        service.connect();

        let listen = service.handle().listen(ctx.link().callback(Msg::Tick));

        Self {
            service,
            input,
            _listen: listen,
            request: ws::Request::empty(),
            responses: Vec::new(),
            tick: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Error(error) => {
                tracing::error!("WebSocket error: {:?}", error);
                false
            }
            Msg::Send => {
                let Some(input) = self.input.cast::<HtmlInputElement>() else {
                    return false;
                };

                let value = input.value();
                input.set_value("");

                self.request = self
                    .service
                    .handle()
                    .request::<api::Hello>()
                    .body(api::HelloRequest {
                        message: value.as_str(),
                    })
                    .on_packet(
                        ctx.link()
                            .callback(|result: Result<_, _>| Msg::HelloResponse(result)),
                    )
                    .send();

                true
            }
            Msg::HelloResponse(Err(error)) => {
                tracing::error!("Request error: {:?}", error);
                false
            }
            Msg::HelloResponse(Ok(packet)) => {
                tracing::debug!("Got response");

                while !packet.is_empty() {
                    let Ok(response) = packet.decode() else {
                        break;
                    };

                    self.responses.push(response.message.to_owned());
                }

                true
            }
            Msg::Tick(packet) => {
                tracing::debug!("Got tick");

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
                {for self.responses.iter().enumerate().map(|(index, response)| html!(<div>{format!("Response #{index}: {response}")}</div>))}
                <div>{format!("Global tick: {}", self.tick)}</div>
            </div>
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();

    let mut config = WASMLayerConfigBuilder::new();
    config.set_max_level(Level::INFO);

    if let Err(error) = tracing::subscriber::set_global_default(
        Registry::default().with(WASMLayer::new(config.build())),
    ) {
        panic!("Failed to set logger: {error:?}");
    }

    tracing::trace!("Started up");
    yew::Renderer::<App>::new().render();
}
