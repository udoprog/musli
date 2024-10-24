use core::fmt;
use std::cell::{Cell, RefCell};
use std::collections::{hash_map, HashMap};
use std::marker::PhantomData;
use std::mem::take;
use std::rc::Rc;

use gloo::timers::callback::Timeout;
use musli::mode::Binary;
use musli::{api, Decode, Encode};
use slab::Slab;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{ArrayBuffer, Uint8Array};
use web_sys::{window, BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use yew::html::{ImplicitClone, Scope};
use yew::{Component, Context};

#[cfg(feature = "log")]
use log::{error, trace};

#[cfg(not(feature = "log"))]
macro_rules! dummy {
    ($msg:literal $(, $expr:expr)* $(,)?) => { $(_ = $expr;)* };
}

#[cfg(not(feature = "log"))]
use {dummy as trace, dummy as error};

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
    Api(musli::api::Error),
    Overflow(usize, usize),
}

impl Error {
    #[inline]
    fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Message(message) => write!(f, "{message}"),
            ErrorKind::Api(error) => write!(f, "Encoding error: {error}"),
            ErrorKind::Overflow(at, len) => {
                write!(f, "Internal packet overflow, {at} not in range 0-{len}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Api(error) => Some(error),
            _ => None,
        }
    }
}

impl From<musli::api::Error> for Error {
    fn from(error: musli::api::Error) -> Self {
        Self::new(ErrorKind::Api(error))
    }
}

impl From<JsValue> for Error {
    fn from(error: JsValue) -> Self {
        Self::new(ErrorKind::Message(format!("{error:?}")))
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Self::new(ErrorKind::Message(error.to_string()))
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

/// The websocket service.
///
/// This needs to be wired up with messages inside of a component in yew, see
/// the example for how this can be done.
///
/// Interaction with the service is done through the [`Handle`] type.
///
/// # Examples
///
/// ```no_run
/// use yew::prelude::*;
/// use musli_yew::ws;
///
/// struct App {
///     ws: ws::Service<Self>,
///     handle: ws::Handle,
/// }
///
/// pub(crate) enum Msg {
///     WebSocket(ws::Msg),
///     Error(ws::Error),
/// }
///
/// impl From<ws::Msg> for Msg {
///     #[inline]
///     fn from(error: ws::Msg) -> Self {
///         Self::WebSocket(error)
///     }
/// }
///
/// impl From<ws::Error> for Msg {
///     #[inline]
///     fn from(error: ws::Error) -> Self {
///         Self::Error(error)
///     }
/// }
///
/// impl Component for App {
///     type Message = Msg;
///     type Properties = ();
///
///     fn create(ctx: &Context<Self>) -> Self {
///         let (ws, handle) = ws::Service::new(ctx);
///         let mut this = Self { ws, handle };
///         this.ws.connect(ctx);
///         this
///     }
///
///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
///         match msg {
///             Msg::WebSocket(msg) => {
///                 self.ws.update(ctx, msg);
///                 false
///             }
///             Msg::Error(error) => {
///                 log::error!("Websocket Error: {error}");
///                 false
///             }
///         }
///     }
///
///     fn view(&self, ctx: &Context<Self>) -> Html {
///         html! {
///             "Hello World"
///         }
///     }
/// }
/// ```
pub struct Service<C> {
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
    /// Construct a new websocket service, and return it and an associated
    /// handle to it.
    pub fn new(ctx: &Context<C>) -> (Self, Handle) {
        let link = ctx.link().clone();

        let shared = Rc::new(Shared {
            serial: Cell::new(0),
            onmessage: Box::new(move |request| {
                link.send_message(Msg::new(MsgKind::ClientRequest(request)))
            }),
            requests: RefCell::new(Slab::new()),
            broadcasts: RefCell::new(HashMap::new()),
            state_changes: RefCell::new(Slab::new()),
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
        musli::api::encode(musli::wrap::wrap(&mut self.output), &request.header)?;
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

        let header: api::ResponseHeader<'_> = musli::api::decode(&mut reader)?;

        match header.broadcast {
            Some(kind) => {
                let broadcasts = self.shared.broadcasts.borrow();
                let at = body.len() - reader.remaining();

                if let Some(broadcasts) = broadcasts.get(kind) {
                    let mut it = broadcasts.iter();

                    let last = it.next_back();

                    let raw = RawPacket {
                        body: body.clone(),
                        at,
                    };

                    for (_, callback) in it {
                        (callback)(raw.clone());
                    }

                    if let Some((_, callback)) = last {
                        (callback)(raw);
                    }
                }
            }
            None => {
                let (index, serial) = unpack(header.serial);
                trace!("Got response index={index}, serial={serial}");

                let requests = self.shared.requests.borrow();

                let Some(pending) = requests.get(index) else {
                    return Ok(());
                };

                if pending.serial == serial {
                    if let Some(error) = header.error {
                        pending.callback.error(error);
                    } else {
                        let at = body.len() - reader.remaining();
                        pending.callback.packet(RawPacket { body, at });
                    }
                }
            }
        }

        Ok(())
    }

    fn set_open(&mut self) {
        trace!("Set open");
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

    fn set_closed(&mut self, ctx: &Context<C>) {
        trace!(
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
        self.reconnect(ctx);
        self.emit_state_change(State::Closed);
    }

    fn emit_state_change(&mut self, state: State) {
        if self.state != state {
            let callbacks = self.shared.state_changes.borrow();

            for (_, callback) in callbacks.iter() {
                callback(state);
            }

            self.state = state;
        }
    }

    /// Handle an update message.
    pub fn update(&mut self, ctx: &Context<C>, message: Msg) {
        match message.kind {
            MsgKind::Reconnect => {
                trace!("Reconnect");

                if let Err(error) = self.inner_connect() {
                    ctx.link().send_message(error);
                    self.inner_reconnect(ctx);
                }
            }
            MsgKind::Open => {
                trace!("Open");
                self.set_open();

                let buffer = take(&mut self.buffer);

                for request in buffer {
                    if let Err(error) = self.send_client_request(request) {
                        ctx.link().send_message(error);
                    }
                }
            }
            MsgKind::Close(e) => {
                trace!("Close: {} ({})", e.code(), e.reason());
                self.set_closed(ctx);
            }
            MsgKind::Message(e) => {
                if let Err(error) = self.message(e) {
                    ctx.link().send_message(error);
                }
            }
            MsgKind::Error(e) => {
                error!("{}", e.message());
                self.set_closed(ctx);
            }
            MsgKind::ClientRequest(request) => {
                if self.opened.is_none() {
                    self.buffer.push(request);
                    return;
                }

                if let Err(error) = self.send_client_request(request) {
                    ctx.link().send_message(error);
                }
            }
        }
    }

    pub(crate) fn reconnect(&mut self, ctx: &Context<C>) {
        if let Some(old) = self.socket.take() {
            if let Err(error) = old.close() {
                ctx.link().send_message(Error::from(error));
            }
        }

        let link = ctx.link().clone();

        self._timeout = Some(Timeout::new(self.timeout, move || {
            link.send_message(Msg::new(MsgKind::Reconnect));
        }));
    }

    /// Attempt to establish a websocket connection.
    pub fn connect(&mut self, ctx: &Context<C>) {
        if let Err(error) = self.inner_connect() {
            ctx.link().send_message(error);
            self.inner_reconnect(ctx);
        }
    }

    fn inner_connect(&mut self) -> Result<()> {
        let window = window().ok_or("No window")?;
        let port = window.location().port()?;
        let url = format!("ws://127.0.0.1:{port}/ws");
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

    fn inner_reconnect(&mut self, ctx: &Context<C>) {
        let link = ctx.link().clone();

        self._timeout = Some(Timeout::new(1000, move || {
            link.send_message(Msg::new(MsgKind::Reconnect));
        }));
    }
}

/// The handle for a pending request. Dropping this handle cancels the request.
pub struct Request<T> {
    inner: Option<(Rc<Shared>, u32)>,
    _marker: PhantomData<T>,
}

impl<T> Request<T> {
    /// An empty request handler.
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }
}

impl<T> Default for Request<T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: None,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for Request<T> {
    #[inline]
    fn drop(&mut self) {
        if let Some((shared, index)) = self.inner.take() {
            shared.requests.borrow_mut().try_remove(index as usize);
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
        let mut broadcast = self.shared.broadcasts.borrow_mut();

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
            .state_changes
            .borrow_mut()
            .try_remove(self.index);
    }
}

#[derive(Clone)]
struct RawPacket {
    body: Rc<[u8]>,
    at: usize,
}

/// A packet of data.
pub struct Packet<T> {
    raw: RawPacket,
    _marker: PhantomData<T>,
}

impl<T> Packet<T> {
    /// Handle a broadcast packet.
    pub fn decode(
        &self,
        ctx: &Context<impl Component<Message: From<Error>>>,
        f: impl FnOnce(T::Response<'_>),
    ) where
        for<'de> T: api::Endpoint<Response<'de>: Decode<'de, Binary>>,
    {
        let Some(bytes) = self.raw.body.get(self.raw.at..) else {
            ctx.link().send_message(Error::new(ErrorKind::Overflow(
                self.raw.at,
                self.raw.body.len(),
            )));
            return;
        };

        match musli::api::from_slice(bytes) {
            Ok(value) => {
                f(value);
            }
            Err(error) => {
                ctx.link().send_message(Error::from(error));
            }
        }
    }
}

/// A handle to the WebSocket [`Service`].
///
/// Through a handle you can initialize a [`Request`] with [`Handle::request`].
/// This can conveniently be constructed through a [`Default`] implementation of
/// no request is pending. Dropping this handle will cancel the request.
///
/// You can also listen to broadcast events through [`Handle::listen`].
/// Similarly here, dropping the handle will cancel the listener.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::api::{Endpoint, Request};
/// use musli_yew::ws;
/// use yew::prelude::*;
///
/// #[derive(Encode, Decode)]
/// pub struct MessageOfTheDayResponse<'a> {
///     pub message_of_the_day: &'a str,
/// }
///
/// #[derive(Request, Encode, Decode)]
/// #[request(endpoint = MessageOfTheDay)]
/// pub struct MessageOfTheDayRequest;
///
/// #[derive(Endpoint)]
/// #[endpoint(response<'de> = MessageOfTheDayResponse<'de>)]
/// pub enum MessageOfTheDay {}
///
/// pub(crate) struct Dashboard {
///     message_of_the_day: String,
///     _initialize: ws::Request<MessageOfTheDay>,
/// }
///
/// pub(crate) enum Msg {
///     MessageOfTheDay(ws::Packet<MessageOfTheDay>),
///     Error(ws::Error),
/// }
///
/// impl From<ws::Packet<MessageOfTheDay>> for Msg {
///     #[inline]
///     fn from(packet: ws::Packet<MessageOfTheDay>) -> Self {
///         Self::MessageOfTheDay(packet)
///     }
/// }
///
/// impl From<ws::Error> for Msg {
///     #[inline]
///     fn from(error: ws::Error) -> Self {
///         Self::Error(error)
///     }
/// }
///
/// #[derive(Properties, PartialEq)]
/// pub(crate) struct Props {
///     pub(crate) ws: ws::Handle,
///     pub(crate) onerror: Callback<ws::Error>,
/// }
///
/// impl Component for Dashboard {
///     type Message = Msg;
///     type Properties = Props;
///
///     fn create(ctx: &Context<Self>) -> Self {
///         Self {
///             message_of_the_day: String::new(),
///             _initialize: ctx.props().ws.request(ctx, MessageOfTheDayRequest),
///         }
///     }
///
///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
///         match msg {
///             Msg::MessageOfTheDay(packet) => {
///                 packet.decode(ctx, |update| {
///                     self.message_of_the_day = update.message_of_the_day.to_owned();
///                 });
///
///                 true
///             }
///             Msg::Error(error) => {
///                 ctx.props().onerror.emit(error);
///                 false
///             }
///         }
///     }
///
///     fn view(&self, _: &Context<Self>) -> Html {
///         html! {
///             {self.message_of_the_day.clone()}
///         }
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Handle {
    shared: Rc<Shared>,
}

impl Handle {
    /// Send a request of type `T` and returns a handle for the request.
    ///
    /// If the handle is dropped, the request is cancelled.
    ///
    /// See [`Handle`] for an example.
    pub fn request<T>(
        &self,
        ctx: &Context<impl Component<Message: From<Packet<T::Endpoint>> + From<Error>>>,
        request: T,
    ) -> Request<T::Endpoint>
    where
        T: api::Request + Encode<Binary>,
    {
        struct CallbackImpl<C, T>
        where
            C: Component,
        {
            link: Scope<C>,
            _marker: PhantomData<T>,
        }

        impl<C, T> Callback for CallbackImpl<C, T>
        where
            C: Component<Message: From<Packet<T>> + From<Error>>,
        {
            fn packet(&self, raw: RawPacket) {
                self.link.send_message(Packet {
                    raw,
                    _marker: PhantomData,
                })
            }

            fn error(&self, error: &str) {
                self.link.send_message(Error::from(error));
            }
        }

        let body = match musli::api::to_vec(&request) {
            Ok(body) => body,
            Err(error) => {
                ctx.link().send_message(Error::from(error));
                return Request::default();
            }
        };

        let serial = self.shared.serial.get();
        self.shared.serial.set(serial.wrapping_add(1));

        let pending = Pending {
            serial,
            callback: Box::new(CallbackImpl {
                link: ctx.link().clone(),
                _marker: PhantomData,
            }),
        };

        let index = self.shared.requests.borrow_mut().insert(pending) as u32;

        (self.shared.onmessage)(ClientRequest {
            header: api::RequestHeader {
                serial: pack(index, serial),
                kind: <T::Endpoint as api::Endpoint>::KIND,
            },
            body,
        });

        Request {
            inner: Some((self.shared.clone(), index)),
            _marker: PhantomData,
        }
    }

    /// List for broadcasts of type `T` and returns a handle for the listener.
    ///
    /// If the handle is dropped, the listener is cancelled.
    ///
    /// See [`Handle`] for an example.
    pub fn listen<T>(
        &self,
        ctx: &Context<impl Component<Message: From<Packet<T>> + From<Error>>>,
    ) -> Listener<T>
    where
        T: api::Endpoint,
    {
        let mut broadcasts = self.shared.broadcasts.borrow_mut();

        let slots = broadcasts.entry(T::KIND).or_default();
        let link = ctx.link().clone();

        let index = slots.insert(Box::new(move |raw| {
            link.send_message(Packet {
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
    pub fn state_changes(
        &self,
        ctx: &Context<impl Component<Message: From<State>>>,
    ) -> StateListener {
        let link = ctx.link().clone();
        let mut state = self.shared.state_changes.borrow_mut();

        let index = state.insert(Box::new(move |state| link.send_message(state)));

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

trait Callback {
    /// Handle a packet.
    fn packet(&self, packet: RawPacket);

    /// Handle an error.
    fn error(&self, error: &str);
}

struct Pending {
    serial: u32,
    callback: Box<dyn Callback>,
}

type Broadcasts = HashMap<&'static str, Slab<Box<dyn Fn(RawPacket)>>>;
type OnMessageCallback = dyn Fn(ClientRequest<'static>);
type StateCallback = dyn Fn(State);

struct Shared {
    serial: Cell<u32>,
    onmessage: Box<OnMessageCallback>,
    requests: RefCell<Slab<Pending>>,
    broadcasts: RefCell<Broadcasts>,
    state_changes: RefCell<Slab<Box<StateCallback>>>,
}

#[derive(Debug, Clone, Copy)]
struct Opened {
    at: Option<f64>,
}

#[inline]
fn pack(a: u32, b: u32) -> u64 {
    ((a as u64) << 32) | (b as u64)
}

#[inline]
fn unpack(serial: u64) -> (usize, u32) {
    ((serial >> 32) as usize, serial as u32)
}

#[test]
fn test_pack() {
    assert_eq!(unpack(pack(0, 0)), (0, 0));
    assert_eq!(unpack(pack(1, 0)), (1, 0));
    assert_eq!(unpack(pack(u32::MAX, 0)), (u32::MAX as usize, 0));
}
