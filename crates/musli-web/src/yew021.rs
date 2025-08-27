//! Prelude to import when integrating with yew `0.21.x`.

use yew021::Callback;

pub use crate::web03 as ws;

use crate::api;
use crate::web03::{Error, RawPacket};
use crate::web03::{Handle, Listener, Packet, RequestBuilder};
use crate::web03::{State, StateListener};

/// Request builder extension for interacting with handles in yew `0.21.x`.
pub trait HandleExt {
    /// List for broadcasts of type `T`.
    ///
    /// Returns a handle for the broadcasts.
    ///
    /// If the handle is dropped, the listener is cancelled.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::yew021::*;
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
    fn listen<T>(&self, callback: Callback<Packet<T>>) -> Listener
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
    /// use musli_web::yew021::*;
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
    fn on_state_change(&self, callback: Callback<State>) -> (State, StateListener);
}

impl HandleExt for Handle {
    #[inline]
    fn listen<T>(&self, callback: Callback<Packet<T>>) -> Listener
    where
        T: api::Listener,
    {
        Handle::listen::<T>(self, move |packet| callback.emit(Packet::new(packet)))
    }

    #[inline]
    fn on_state_change(&self, callback: Callback<State>) -> (State, StateListener) {
        Handle::on_state_change(self, move |state| callback.emit(state))
    }
}

/// Request builder extension for interacting with request builders in yew `0.21.x`.
pub trait RequestBuilderExt<E>
where
    Self: Sized,
    E: api::Endpoint,
{
    /// Handle the response by converting a typed packet into a message.
    fn on_packet(self, f: Callback<Result<Packet<E>, Error>>) -> Self;

    /// Handle the response by converting a raw packet into a message.
    ///
    /// This can be useful if all of your responses are expected to return the
    /// same response type and you only want to have one message to handle all
    /// of them.
    fn on_raw_packet(self, f: Callback<Result<RawPacket, Error>>) -> Self;
}

impl<E> RequestBuilderExt<E> for RequestBuilder<'_, E>
where
    E: api::Endpoint,
{
    #[inline]
    fn on_packet(self, callback: Callback<Result<Packet<E>, Error>>) -> Self {
        RequestBuilder::on_packet(self, move |result| match result {
            Ok(raw) => {
                callback.emit(Ok(Packet::new(raw)));
            }
            Err(error) => {
                callback.emit(Err(error));
            }
        })
    }

    #[inline]
    fn on_raw_packet(self, callback: Callback<Result<RawPacket, Error>>) -> Self {
        RequestBuilder::on_packet(self, move |result| match result {
            Ok(raw) => {
                callback.emit(Ok(raw));
            }
            Err(error) => {
                callback.emit(Err(error));
            }
        })
    }
}
