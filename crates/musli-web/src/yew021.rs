//! Client side implementation for [`yew`].
//!
//! [`yew`]: https://yew.rs
//!
//! # Examples
//!
//! ```
//! # extern crate yew021 as yew;
//! # extern crate web_sys03 as web_sys;
//! use web_sys::HtmlInputElement;
//! use yew::prelude::*;
//!
//! use musli_web::yew021 as ws;
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
//!     WebSocket(ws::Msg),
//!     Send,
//!     HelloResponse(Result<ws::Packet<api::Hello>, ws::Error>),
//!     Tick(ws::Packet<api::Tick>),
//! }
//!
//! impl From<ws::Error> for Msg {
//!     #[inline]
//!     fn from(error: ws::Error) -> Self {
//!         Msg::Error(error)
//!     }
//! }
//!
//! impl From<ws::Msg> for Msg {
//!     #[inline]
//!     fn from(error: ws::Msg) -> Self {
//!         Msg::WebSocket(error)
//!     }
//! }
//!
//! struct App {
//!     service: ws::Service<Self>,
//!     handle: ws::Handle,
//!     input: NodeRef,
//!     _listen: ws::Listener<api::Tick>,
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
//!         let (mut service, handle) =
//!             ws::Service::new(ctx, ws::Connect::location_with_path(String::from("/ws")));
//!         let input = NodeRef::default();
//!
//!         service.connect();
//!
//!         let listen = handle.listen(ctx.link().callback(Msg::Tick));
//!
//!         Self {
//!             service,
//!             handle,
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
//!                 log::error!("WebSocket error: {:?}", error);
//!                 false
//!             }
//!             Msg::WebSocket(msg) => {
//!                 self.service.update(msg);
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
//!                     .handle
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
//!                 log::error!("Request error: {error}");
//!                 false
//!             }
//!             Msg::HelloResponse(Ok(packet)) => {
//!                 log::warn!("Got response");
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

use core::cell::{Cell, Ref, RefCell, RefMut};
use core::fmt;
use core::marker::PhantomData;
use core::mem::take;
use std::rc::Weak;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use musli::Decode;
use musli::alloc::Global;
use musli::mode::Binary;

use std::collections::{HashMap, hash_map};

use gloo_timers03::callback::Timeout;
use slab::Slab;
use wasm_bindgen02::closure::Closure;
use wasm_bindgen02::{JsCast, JsValue};
use web_sys03::js_sys::{ArrayBuffer, Uint8Array};
use web_sys03::{BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket, window};
use yew021::html::{ImplicitClone, Scope};
use yew021::{Callback, Component, Context};

use crate::api;

const MAX_CAPACITY: usize = 1048576;

/// The state of the connection.
///
/// A listener for state changes can be set up through
/// [`Handle::state_changes`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum State {
    /// The connection is open.
    Open,
    /// The connection is closed.
    Closed,
}

/// Error type for the WebSocket service.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Message(String),
    Storage(musli::storage::Error),
    Overflow(usize, usize),
}

impl Error {
    #[inline]
    fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Message(message) => write!(f, "{message}"),
            ErrorKind::Storage(error) => write!(f, "Encoding error: {error}"),
            ErrorKind::Overflow(at, len) => {
                write!(f, "Internal packet overflow, {at} not in range 0-{len}")
            }
        }
    }
}

impl core::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<musli::storage::Error> for Error {
    #[inline]
    fn from(error: musli::storage::Error) -> Self {
        Self::new(ErrorKind::Storage(error))
    }
}

impl From<JsValue> for Error {
    #[inline]
    fn from(error: JsValue) -> Self {
        Self::new(ErrorKind::Message(format!("{error:?}")))
    }
}

impl From<String> for Error {
    #[inline]
    fn from(message: String) -> Self {
        Self::new(ErrorKind::Message(message))
    }
}

impl From<&str> for Error {
    #[inline]
    fn from(error: &str) -> Self {
        Self::from(error.to_string())
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

const INITIAL_TIMEOUT: u32 = 250;
const MAX_TIMEOUT: u32 = 16000;

struct ClientRequest<'a> {
    header: api::RequestHeader<'a>,
    body: Vec<u8>,
}

enum MsgKind {
    Reconnect,
    Open,
    Close(CloseEvent),
    Message(MessageEvent),
    Error(ErrorEvent),
    ClientRequest(ClientRequest<'static>),
}

/// A message passed into the WebSocket service.
pub struct Msg {
    kind: MsgKind,
}

impl Msg {
    #[inline]
    const fn new(kind: MsgKind) -> Self {
        Self { kind }
    }
}

/// How to connect to the websocket.
enum ConnectKind {
    Location { path: Option<String> },
    Url { url: String },
}

/// A specification for how to connect a websocket.
pub struct Connect {
    kind: ConnectKind,
}

impl Connect {
    /// Connect to the same location.
    #[inline]
    pub fn location() -> Self {
        Self {
            kind: ConnectKind::Location { path: None },
        }
    }

    /// Connect to the same location with a custom path.
    #[inline]
    pub fn location_with_path(path: impl AsRef<str>) -> Self {
        Self {
            kind: ConnectKind::Location {
                path: Some(String::from(path.as_ref())),
            },
        }
    }

    /// Connect to the specified URL.
    #[inline]
    pub fn url(url: String) -> Self {
        Self {
            kind: ConnectKind::Url { url },
        }
    }
}

/// The WebSocket service.
pub struct Service<C>
where
    C: Component,
{
    link: Scope<C>,
    connect: Connect,
    shared: Rc<Shared>,
    socket: Option<WebSocket>,
    opened: Option<Opened>,
    state: State,
    buffer: Vec<ClientRequest<'static>>,
    output: Vec<u8>,
    timeout: u32,
    on_open: Closure<dyn Fn()>,
    on_close: Closure<dyn Fn(CloseEvent)>,
    on_message: Closure<dyn Fn(MessageEvent)>,
    on_error: Closure<dyn Fn(ErrorEvent)>,
    _timeout: Option<Timeout>,
    _ping_timeout: Option<Timeout>,
    _marker: PhantomData<C>,
}

impl<C> Service<C>
where
    C: Component<Message: From<Msg> + From<Error>>,
{
    /// Construct a new websocket service, and return it and return the service
    /// instance and handle associated with it.
    pub fn new(ctx: &Context<C>, connect: Connect) -> (Self, Handle) {
        let link = ctx.link().clone();

        let shared = Rc::new(Shared {
            serial: Cell::new(0),
            onmessage: Box::new(move |request| {
                link.send_message(Msg::new(MsgKind::ClientRequest(request)))
            }),
            mutable: RefCell::new(Mutable {
                requests: Slab::new(),
                broadcasts: HashMap::new(),
                state_changes: Slab::new(),
            }),
        });

        let on_open = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn()> = Box::new(move || {
                link.send_message(Msg::new(MsgKind::Open));
            });

            Closure::wrap(cb)
        };

        let on_close = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn(CloseEvent)> = Box::new(move |e: CloseEvent| {
                link.send_message(Msg::new(MsgKind::Close(e)));
            });

            Closure::wrap(cb)
        };

        let on_message = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn(MessageEvent)> = Box::new(move |e: MessageEvent| {
                link.send_message(Msg::new(MsgKind::Message(e)));
            });

            Closure::wrap(cb)
        };

        let on_error = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn(ErrorEvent)> = Box::new(move |e: ErrorEvent| {
                link.send_message(Msg::new(MsgKind::Error(e)));
            });

            Closure::wrap(cb)
        };

        let this = Self {
            link: ctx.link().clone(),
            connect,
            shared: shared.clone(),
            socket: None,
            opened: None,
            state: State::Closed,
            buffer: Vec::new(),
            output: Vec::new(),
            timeout: INITIAL_TIMEOUT,
            on_open,
            on_close,
            on_message,
            on_error,
            _timeout: None,
            _ping_timeout: None,
            _marker: PhantomData,
        };

        let handle = Handle { shared };

        (this, handle)
    }

    /// Send a client message.
    fn send_client_request(&mut self, request: ClientRequest<'_>) -> Result<()> {
        let Some(socket) = &self.socket else {
            return Err("Socket is not connected".into());
        };

        self.output.clear();
        musli::storage::to_writer(&mut self.output, &request.header)?;
        self.output.extend_from_slice(request.body.as_slice());
        socket.send_with_u8_array(&self.output)?;
        self.output.shrink_to(MAX_CAPACITY);
        Ok(())
    }

    fn message(&mut self, e: MessageEvent) -> Result<()> {
        let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() else {
            return Err("Expected message as ArrayBuffer".into());
        };

        let body = Rc::from(Uint8Array::new(&array_buffer).to_vec());
        let mut reader = musli::reader::SliceReader::new(&body);

        let header: api::ResponseHeader<'_> = musli::storage::decode(&mut reader)?;

        match header.broadcast {
            Some(kind) => {
                let broadcasts = Ref::map(self.shared.mutable.borrow(), |m| &m.broadcasts);
                let at = body.len() - reader.remaining();

                if let Some((&kind, broadcasts)) = broadcasts.get_key_value(kind) {
                    for (_, callback) in broadcasts.iter() {
                        (callback)(RawPacket {
                            body: body.clone(),
                            at,
                            kind,
                        });
                    }
                }
            }
            None => {
                log::trace!(
                    "Got response: index={}, serial={}",
                    header.index,
                    header.serial
                );

                let requests = Ref::map(self.shared.mutable.borrow(), |m| &m.requests);

                if let Some(pending) = requests.get(header.index as usize)
                    && pending.serial == header.serial
                {
                    if let Some(error) = header.error {
                        if let Some(callback) = &pending.callback {
                            callback(Err(Error::from(error)));
                        }
                    } else if let Some(callback) = &pending.callback {
                        let at = body.len() - reader.remaining();
                        let raw = RawPacket {
                            body,
                            at,
                            kind: pending.kind,
                        };
                        callback(Ok(raw));
                    }
                }
            }
        }

        Ok(())
    }

    fn set_open(&mut self) {
        log::trace!("Set open");
        self.opened = Some(Opened { at: now() });
        self.emit_state_change(State::Open);
    }

    fn is_open_for_a_while(&self) -> bool {
        let Some(opened) = self.opened else {
            return false;
        };

        let Some(at) = opened.at else {
            return false;
        };

        let Some(now) = now() else {
            return false;
        };

        (now - at) >= 250.0
    }

    fn set_closed(&mut self) {
        log::trace!(
            "Set closed timeout={}, opened={:?}",
            self.timeout,
            self.opened
        );

        if !self.is_open_for_a_while() {
            if self.timeout < MAX_TIMEOUT {
                self.timeout *= 2;
            }
        } else {
            self.timeout = INITIAL_TIMEOUT;
        }

        self.opened = None;
        self.reconnect();
        self.emit_state_change(State::Closed);
    }

    fn emit_state_change(&mut self, state: State) {
        if self.state != state {
            let callbacks = Ref::map(self.shared.mutable.borrow(), |m| &m.state_changes);

            for (_, callback) in callbacks.iter() {
                callback(state);
            }

            self.state = state;
        }
    }

    /// Handle an update message.
    pub fn update(&mut self, message: Msg) {
        match message.kind {
            MsgKind::Reconnect => {
                log::trace!("Reconnect");

                if let Err(error) = self.inner_connect() {
                    self.link.send_message(error);
                    self.inner_reconnect();
                }
            }
            MsgKind::Open => {
                log::trace!("Open");
                self.set_open();

                let buffer = take(&mut self.buffer);

                for request in buffer {
                    if let Err(error) = self.send_client_request(request) {
                        self.link.send_message(error);
                    }
                }
            }
            MsgKind::Close(e) => {
                log::trace!("Close: {} ({})", e.code(), e.reason());
                self.set_closed();
            }
            MsgKind::Message(e) => {
                if let Err(error) = self.message(e) {
                    self.link.send_message(error);
                }
            }
            MsgKind::Error(e) => {
                log::error!("{}", e.message());
                self.set_closed();
            }
            MsgKind::ClientRequest(request) => {
                if self.opened.is_none() {
                    self.buffer.push(request);
                    return;
                }

                if let Err(error) = self.send_client_request(request) {
                    self.link.send_message(error);
                }
            }
        }
    }

    pub(crate) fn reconnect(&mut self) {
        if let Some(old) = self.socket.take()
            && let Err(error) = old.close()
        {
            self.link.send_message(Error::from(error));
        }

        let link = self.link.clone();

        self._timeout = Some(Timeout::new(self.timeout, move || {
            link.send_message(Msg::new(MsgKind::Reconnect));
        }));
    }

    /// Attempt to establish a websocket connection.
    pub fn connect(&mut self) {
        if let Err(error) = self.inner_connect() {
            self.link.send_message(error);
            self.inner_reconnect();
        }
    }

    fn inner_connect(&mut self) -> Result<()> {
        struct ForcePrefix<'a>(&'a str);

        impl fmt::Display for ForcePrefix<'_> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                '/'.fmt(f)?;
                self.0.trim_start_matches('/').fmt(f)?;
                Ok(())
            }
        }

        let url = match &self.connect.kind {
            ConnectKind::Location { path } => {
                let window = window().ok_or("No window")?;
                let protocol = window.location().protocol()?;

                let protocol = match protocol.as_str() {
                    "https:" => "wss:",
                    "http:" => "ws:",
                    other => {
                        return Err(Error::from(format!(
                            "Same host connection is not supported for protocol `{other}`"
                        )));
                    }
                };

                let host = window.location().hostname()?;
                let port = window.location().port()?;

                let path = match path {
                    Some(path) => ForcePrefix(path),
                    None => ForcePrefix(""),
                };

                format!("{protocol}//{host}:{port}{path}")
            }
            ConnectKind::Url { url } => url.clone(),
        };

        let ws = WebSocket::new(&url)?;

        ws.set_binary_type(BinaryType::Arraybuffer);
        ws.set_onopen(Some(self.on_open.as_ref().unchecked_ref()));
        ws.set_onclose(Some(self.on_close.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(self.on_message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(self.on_error.as_ref().unchecked_ref()));

        if let Some(old) = self.socket.replace(ws) {
            old.close()?;
        }

        Ok(())
    }

    fn inner_reconnect(&mut self) {
        let link = self.link.clone();

        self._timeout = Some(Timeout::new(1000, move || {
            link.send_message(Msg::new(MsgKind::Reconnect));
        }));
    }
}

/// A request builder .
///
/// Associate the callback to be used by using either
/// [`RequestBuilder::on_packet`] or [`RequestBuilder::on_raw_packet`] depending
/// on your needs.
///
/// Send the request with [`RequestBuilder::send`].
pub struct RequestBuilder<'a, E> {
    shared: &'a Rc<Shared>,
    body: Option<Vec<u8>>,
    callback: Option<Box<dyn Fn(Result<RawPacket>)>>,
    error: Option<Error>,
    _marker: PhantomData<E>,
}

impl<'a, E> RequestBuilder<'a, E>
where
    E: api::Endpoint,
{
    /// Set the body of the request.
    pub fn body<T>(mut self, body: T) -> RequestBuilder<'a, E>
    where
        T: api::Request<Endpoint = E>,
    {
        match musli::storage::to_vec(&body) {
            Ok(vec) => self.body = Some(vec),
            Err(err) => self.error = Some(Error::from(err)),
        }

        self
    }

    /// Handle the response by converting a typed packet into a message.
    pub fn on_packet(mut self, f: Callback<Result<Packet<E>>>) -> Self {
        self.callback = Some(Box::new(move |result| match result {
            Ok(raw) => {
                f.emit(Ok(Packet {
                    raw,
                    _marker: PhantomData,
                }));
            }
            Err(error) => {
                f.emit(Err(error));
            }
        }));

        self
    }

    /// Handle the response by converting a raw packet into a message.
    ///
    /// This can be useful if all of your responses are expected to return the
    /// same response type and you only want to have one message to handle all
    /// of them.
    pub fn on_raw_packet(mut self, f: Callback<Result<RawPacket>>) -> Self {
        self.callback = Some(Box::new(move |result| match result {
            Ok(raw) => {
                f.emit(Ok(raw));
            }
            Err(error) => {
                f.emit(Err(error));
            }
        }));

        self
    }

    /// Build and return the request.
    pub fn send(self) -> Request {
        if let Some(error) = self.error {
            if let Some(callback) = self.callback {
                callback(Err(error));
            }

            return Request::empty();
        }

        let body = self.body.unwrap_or_default();

        let serial = self.shared.serial.get();
        self.shared.serial.set(serial.wrapping_add(1));

        let pending = Pending {
            serial,
            callback: self.callback,
            kind: E::KIND,
        };

        let index = self.shared.mutable.borrow_mut().requests.insert(pending) as u32;

        (self.shared.onmessage)(ClientRequest {
            header: api::RequestHeader {
                index,
                serial,
                kind: E::KIND,
            },
            body,
        });

        Request {
            inner: Some((Rc::downgrade(self.shared), index)),
        }
    }
}

/// The handle for a pending request. Dropping this handle cancels the request.
pub struct Request {
    inner: Option<(Weak<Shared>, u32)>,
}

impl Request {
    /// An empty request handler.
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Default for Request {
    #[inline]
    fn default() -> Self {
        Self { inner: None }
    }
}

impl Drop for Request {
    #[inline]
    fn drop(&mut self) {
        if let Some((s, index)) = self.inner.take()
            && let Some(s) = s.upgrade()
        {
            s.mutable.borrow_mut().requests.try_remove(index as usize);
        }
    }
}

/// The handle for a pending request. Dropping this handle cancels the request.
pub struct Listener<T> {
    kind: &'static str,
    index: usize,
    shared: Rc<Shared>,
    _marker: PhantomData<T>,
}

impl<T> Drop for Listener<T> {
    #[inline]
    fn drop(&mut self) {
        let mut broadcast = RefMut::map(self.shared.mutable.borrow_mut(), |m| &mut m.broadcasts);

        if let hash_map::Entry::Occupied(mut e) = broadcast.entry(self.kind) {
            e.get_mut().try_remove(self.index);

            if e.get().is_empty() {
                e.remove();
            }
        }
    }
}

/// The handle for state change listening. Dropping this handle cancels the request.
pub struct StateListener {
    index: usize,
    shared: Rc<Shared>,
}

impl Drop for StateListener {
    #[inline]
    fn drop(&mut self) {
        self.shared
            .mutable
            .borrow_mut()
            .state_changes
            .try_remove(self.index);
    }
}

/// A raw packet of data.
#[derive(Clone)]
pub struct RawPacket {
    body: Rc<[u8]>,
    at: usize,
    kind: &'static str,
}

impl RawPacket {
    /// Decode the contents of a raw packet.
    pub fn decode<'this, T>(&'this self) -> Result<T>
    where
        T: Decode<'this, Binary, Global>,
    {
        let Some(bytes) = self.body.get(self.at..) else {
            return Err(Error::new(ErrorKind::Overflow(self.at, self.body.len())));
        };

        match musli::storage::from_slice(bytes) {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::from(error)),
        }
    }

    /// The kind of the packet.
    pub fn kind(&self) -> &str {
        self.kind
    }
}

/// A typed packet of data.
#[derive(Clone)]
pub struct Packet<T> {
    raw: RawPacket,
    _marker: PhantomData<T>,
}

impl<T> Packet<T> {
    /// Convert a packet into a raw packet.
    ///
    /// To determine which endpoint or broadcast it belongs to the
    /// [`RawPacket::kind`] method can be used.
    pub fn into_raw(self) -> RawPacket {
        self.raw
    }
}

impl<T> Packet<T>
where
    T: api::Endpoint,
{
    /// Decode a typed response.
    pub fn decode(&self) -> Result<T::Response<'_>> {
        self.raw.decode()
    }
}

impl<T> Packet<T>
where
    T: api::Listener,
{
    /// Decode a typed broadcast.
    pub fn decode_broadcast(&self) -> Result<T::Broadcast<'_>> {
        self.raw.decode()
    }
}

/// A handle to the WebSocket service.
#[derive(Clone)]
pub struct Handle {
    shared: Rc<Shared>,
}

impl Handle {
    /// Send a request of type `T`.
    ///
    /// Returns a handle for the request.
    ///
    /// If the handle is dropped, the request is cancelled.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    ///
    /// use musli_web::yew021 as ws;
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
    ///             .request::<api::Hello>()
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
    ///                 log::error!("Request error: {:?}", error);
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
    pub fn request<E>(&self) -> RequestBuilder<'_, E>
    where
        E: api::Endpoint,
    {
        RequestBuilder {
            shared: &self.shared,
            body: None,
            callback: None,
            error: None,
            _marker: PhantomData,
        }
    }

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
    ///
    /// use musli_web::yew021 as ws;
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
    ///     Error(ws::Error),
    ///     Tick(ws::Packet<api::Tick>),
    /// }
    ///
    /// impl From<ws::Error> for Msg {
    ///     #[inline]
    ///     fn from(error: ws::Error) -> Self {
    ///         Msg::Error(error)
    ///     }
    /// }
    ///
    /// #[derive(Properties, PartialEq)]
    /// struct Props {
    ///     ws: ws::Handle,
    /// }
    ///
    /// struct App {
    ///     tick: u32,
    ///     _listen: ws::Listener<api::Tick>,
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
    ///             Msg::Error(error) => {
    ///                 log::error!("Broadcast error: {:?}", error);
    ///                 false
    ///             }
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
    pub fn listen<T>(&self, f: Callback<Packet<T>>) -> Listener<T>
    where
        T: api::Listener,
    {
        let mut broadcasts = RefMut::map(self.shared.mutable.borrow_mut(), |m| &mut m.broadcasts);

        let slots = broadcasts.entry(T::KIND).or_default();

        let index = slots.insert(Box::new(move |raw| {
            f.emit(Packet {
                raw,
                _marker: PhantomData,
            });
        }));

        Listener {
            kind: T::KIND,
            index,
            shared: self.shared.clone(),
            _marker: PhantomData,
        }
    }

    /// Listen for state changes to the underlying connection.
    pub fn state_changes<C>(
        &self,
        ctx: &Context<C>,
        f: impl Fn(State) -> C::Message + 'static,
    ) -> StateListener
    where
        C: Component,
    {
        let link = ctx.link().clone();
        let mut state = RefMut::map(self.shared.mutable.borrow_mut(), |m| &mut m.state_changes);
        let index = state.insert(Box::new(move |state| link.send_message(f(state))));

        StateListener {
            index,
            shared: self.shared.clone(),
        }
    }
}

impl ImplicitClone for Handle {
    #[inline]
    fn implicit_clone(&self) -> Self {
        self.clone()
    }
}

impl PartialEq for Handle {
    #[inline]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

fn now() -> Option<f64> {
    Some(window()?.performance()?.now())
}

struct Pending {
    serial: u32,
    callback: Option<Box<dyn Fn(Result<RawPacket>)>>,
    kind: &'static str,
}

type Broadcasts = HashMap<&'static str, Slab<Box<dyn Fn(RawPacket)>>>;
type OnMessageCallback = dyn Fn(ClientRequest<'static>);
type StateCallback = dyn Fn(State);

struct Mutable {
    requests: Slab<Pending>,
    broadcasts: Broadcasts,
    state_changes: Slab<Box<StateCallback>>,
}

struct Shared {
    serial: Cell<u32>,
    onmessage: Box<OnMessageCallback>,
    mutable: RefCell<Mutable>,
}

#[derive(Debug, Clone, Copy)]
struct Opened {
    at: Option<f64>,
}
