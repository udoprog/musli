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
    Change(String),
    Send,
    HelloResponse(Result<ws::Packet<api::Hello>, ws::Error>),
    Tick(Result<ws::Packet<api::Tick>, ws::Error>),
}

struct App {
    service: ws::Service,
    _listen: ws::Listener,
    request: ws::Request,
    text: String,
    tick: u32,
    responses: Vec<String>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let service = ws::connect(ws::Connect::location("ws"))
            .on_error(ctx.link().callback(Msg::Error))
            .build();

        service.connect();

        let listen = service.handle().listen(ctx.link().callback(Msg::Tick));

        Self {
            service,
            _listen: listen,
            request: ws::Request::new(),
            text: String::new(),
            tick: 0,
            responses: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Error(error) => {
                tracing::error!("WebSocket error: {:?}", error);
                false
            }
            Msg::Change(text) => {
                self.text = text;
                true
            }
            Msg::Send => {
                self.request = self
                    .service
                    .handle()
                    .request::<api::Hello>()
                    .body(api::HelloRequest {
                        message: self.text.as_str(),
                    })
                    .on_packet(
                        ctx.link()
                            .callback(|result: Result<_, _>| Msg::HelloResponse(result)),
                    )
                    .send();

                self.text.clear();
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
            Msg::Tick(Err(error)) => {
                tracing::error!("Tick error: {error}");
                false
            }
            Msg::Tick(Ok(packet)) => {
                tracing::debug!("Got tick");

                if let Ok(tick) = packet.decode_broadcast() {
                    self.tick = tick.tick;
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            Msg::Change(input.value())
        });

        let onkeydown = ctx
            .link()
            .batch_callback(|e: KeyboardEvent| (e.key() == "Enter").then_some(Msg::Send));

        let onclick = ctx.link().callback(|_: MouseEvent| Msg::Send);

        html! {
            <div class="container">
                <input key="input" type="text" {oninput} {onkeydown} value={self.text.clone()} />
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
