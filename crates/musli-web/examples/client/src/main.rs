//! Example musli-web client based on [`yew`].
//!
//! Run with:
//!
//! ```sh
//! trunk serve
//! ```
//!
//! [`yew`]: https://yew.rs

use musli_web::web03::prelude::*;
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
    State(ws::State),
}

struct App {
    state: ws::State,
    service: ws::Service,
    _listen: ws::Listener,
    _state_listener: ws::StateListener,
    request: ws::Request,
    text: String,
    tick: u32,
    responses: Vec<String>,
    messages: Vec<String>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let service = ws::connect(ws::Connect::location("ws"))
            .on_error(ctx.link().callback(Msg::Error))
            .build();

        service.connect();

        let listen = service
            .handle()
            .on_broadcast(ctx.link().callback(Msg::Tick));

        let (state, state_listener) = service
            .handle()
            .on_state_change(ctx.link().callback(Msg::State));

        Self {
            state,
            service,
            _listen: listen,
            _state_listener: state_listener,
            request: ws::Request::new(),
            text: String::new(),
            tick: 0,
            responses: Vec::new(),
            messages: Vec::new(),
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
                    .request()
                    .body(api::HelloRequest {
                        message: self.text.as_str(),
                    })
                    .on_packet(ctx.link().callback(Msg::HelloResponse))
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

                if let Ok(tick) = packet.decode_event() {
                    self.tick = tick.tick;
                    self.messages.push(tick.message.to_owned());
                }

                true
            }
            Msg::State(state) => {
                tracing::debug!(?state);

                self.state = state;
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
                <div key="input">
                    <input key="input" type="text" {oninput} {onkeydown} value={self.text.clone()} />
                    <button {onclick}>{"Send Message"}</button>
                </div>

                <div key="state">{format!("State: {:?}", self.state)}</div>
                {for self.responses.iter().enumerate().map(|(index, response)| html!(<div key={format!("response-{index}")}>{format!("Response #{index}: {response}")}</div>))}
                <div key="tick">{format!("Global tick: {}", self.tick)}</div>
                {for self.messages.iter().enumerate().map(|(index, message)| html!(<div key={format!("message-{index}")}>{format!("Message #{index}: {message}")}</div>))}
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
