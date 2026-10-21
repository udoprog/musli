//! Example musli-web client based on [`yew`].
//!
//! Run with:
//!
//! ```sh
//! trunk serve
//! ```
//!
//! [`yew`]: https://yew.rs

use std::collections::VecDeque;

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
    Channel1(ws::ChannelEvent),
    Channel2(ws::ChannelEvent),
    CloseChannel1,
    CloseChannel2,
    Reconnect,
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
    messages: VecDeque<(usize, String)>,
    channel1: ws::Channel,
    channel2: ws::Channel,
    count: usize,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let service = ws::connect(ws::Connect::location("ws"))
            .close_before_unload()
            .on_error(ctx.link().callback(Msg::Error))
            .build();

        let listen = service
            .handle()
            .on_broadcast(ctx.link().callback(Msg::Tick));

        let (state, _state_listener) = service
            .handle()
            .on_state_change(ctx.link().callback(Msg::State));

        // NB: Channels open once the connection does, and open again by
        // themselves every time it reconnects.
        let channel1 = service
            .handle()
            .channel()
            .on_change(ctx.link().callback(Msg::Channel1))
            .open();

        let channel2 = service
            .handle()
            .channel()
            .on_change(ctx.link().callback(Msg::Channel2))
            .open();

        Self {
            state,
            service,
            _listen: listen,
            _state_listener,
            request: ws::Request::new(),
            text: String::new(),
            tick: 0,
            responses: Vec::new(),
            messages: VecDeque::new(),
            channel1,
            channel2,
            count: 0,
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
                tracing::trace!("Got tick");

                if let Ok(tick) = packet.decode() {
                    self.tick = tick.tick;
                    self.messages
                        .push_back((self.count, tick.message.to_owned()));
                    self.count += 1;

                    if self.messages.len() > 10 {
                        self.messages.pop_front();
                    }
                }

                true
            }
            Msg::State(state) => {
                tracing::debug!(?state);

                self.state = state;

                if self.state.is_open() {
                    self.count = 0;
                    self.responses.clear();
                    self.messages.clear();
                }

                true
            }
            Msg::Reconnect => {
                self.channel1 = self
                    .service
                    .handle()
                    .channel()
                    .on_change(ctx.link().callback(Msg::Channel1))
                    .open();

                self.channel2 = self
                    .service
                    .handle()
                    .channel()
                    .on_change(ctx.link().callback(Msg::Channel2))
                    .open();

                true
            }
            Msg::Channel1(event) => {
                tracing::info!(channel1 = ?event);
                true
            }
            Msg::Channel2(event) => {
                tracing::info!(channel2 = ?event);
                true
            }
            Msg::CloseChannel1 => {
                // NB: Dropping the channel closes it for good.
                self.channel1 = ws::Channel::default();
                true
            }
            Msg::CloseChannel2 => {
                self.channel2 = ws::Channel::default();
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

        let close_channel1 = ctx.link().callback(|_: MouseEvent| Msg::CloseChannel1);
        let close_channel2 = ctx.link().callback(|_: MouseEvent| Msg::CloseChannel2);

        let reconnect_channels = ctx.link().callback(|_: MouseEvent| Msg::Reconnect);

        html! {
            <div class="container">
                <div key="state">{format!("State: {:?}", self.state)}</div>

                <div key="input">
                    <input key="input" type="text" {oninput} {onkeydown} value={self.text.clone()} />
                    <button {onclick}>{"Send Message"}</button>
                </div>

                <div>
                    <button onclick={reconnect_channels}>{"Reconnect channels"}</button>
                </div>

                <div key="channel1">
                    {format!("Channel #1: {:?}", self.channel1.id())}
                </div>

                <div>
                    <button onclick={close_channel1}>{"Close channel 1"}</button>
                </div>

                <div key="channel2">
                    {format!("Channel #2: {:?}", self.channel2.id())}
                </div>

                <div>
                    <button onclick={close_channel2}>{"Close channel 2"}</button>
                </div>

                <div>
                    {for self.responses.iter().enumerate().map(|(index, response)| html!(<div key={format!("response-{index}")}>{format!("Response #{index}: {response}")}</div>))}
                    <div key="tick">{format!("Global tick: {}", self.tick)}</div>
                    {for self.messages.iter().map(|(count, message)| html!(<div key={format!("message-{count}")}>{format!("Message #{count}: {message}")}</div>))}
                </div>
            </div>
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();

    let mut config = WASMLayerConfigBuilder::new();
    config.set_max_level(Level::DEBUG);

    if let Err(error) = tracing::subscriber::set_global_default(
        Registry::default().with(WASMLayer::new(config.build())),
    ) {
        panic!("Failed to set logger: {error:?}");
    }

    tracing::trace!("Started up");
    yew::Renderer::<App>::new().render();
}
