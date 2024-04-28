//! Client side implementation for Yew.

use core::cell::{Cell, RefCell};
use core::fmt;
use core::marker::PhantomData;
use core::mem::take;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use std::collections::{hash_map, HashMap};

use gloo::timers::callback::Timeout;
use slab::Slab;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{ArrayBuffer, Uint8Array};
use web_sys::{window, BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use yew::html::ImplicitClone;
use yew::{Component, Context};

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
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<musli::storage::Error> for Error {
    fn from(error: musli::storage::Error) -> Self {
        Self::new(ErrorKind::Storage(error))
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

/// The WebSocket service.
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
    C: Component,
    C::Message: From<Msg> + From<Error>,
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
                log::trace!(
                    "Got response: index={}, serial={}",
                    header.index,
                    header.serial
                );

                let requests = self.shared.requests.borrow();

                let Some(pending) = requests.get(header.index as usize) else {
                    return Err("Header index out of bound".into());
                };

                if pending.serial == header.serial {
                    if let Some(error) = header.error {
                        (pending.callback)(Err(Error::from(error)));
                    } else {
                        let at = body.len() - reader.remaining();
                        let raw = RawPacket { body, at };
                        (pending.callback)(Ok(raw));
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

    fn set_closed(&mut self, ctx: &Context<C>) {
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
                log::trace!("Reconnect");

                if let Err(error) = self.inner_connect() {
                    ctx.link().send_message(error);
                    self.inner_reconnect(ctx);
                }
            }
            MsgKind::Open => {
                log::trace!("Open");
                self.set_open();

                let buffer = take(&mut self.buffer);

                for request in buffer {
                    if let Err(error) = self.send_client_request(request) {
                        ctx.link().send_message(error);
                    }
                }
            }
            MsgKind::Close(e) => {
                log::trace!("Close: {} ({})", e.code(), e.reason());
                self.set_closed(ctx);
            }
            MsgKind::Message(e) => {
                if let Err(error) = self.message(e) {
                    ctx.link().send_message(error);
                }
            }
            MsgKind::Error(e) => {
                log::error!("{}", e.message());
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
pub struct Packet<T>
where
    T: api::Marker,
{
    raw: RawPacket,
    _marker: PhantomData<T>,
}

impl<T> Packet<T>
where
    T: api::Marker,
{
    /// Handle a broadcast packet.
    pub fn decode<C, F>(&self, ctx: &Context<C>, f: F)
    where
        F: FnOnce(T::Type<'_>),
        C: Component,
        C::Message: From<Error>,
    {
        let Some(bytes) = self.raw.body.get(self.raw.at..) else {
            ctx.link()
                .send_message(C::Message::from(Error::new(ErrorKind::Overflow(
                    self.raw.at,
                    self.raw.body.len(),
                ))));
            return;
        };

        match musli::storage::from_slice(bytes) {
            Ok(value) => {
                f(value);
            }
            Err(error) => {
                ctx.link()
                    .send_message(C::Message::from(Error::from(error)));
            }
        }
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
    pub fn request<C, T>(&self, ctx: &Context<C>, request: T) -> Request<T::Marker>
    where
        C: Component,
        C::Message: From<Packet<T::Marker>> + From<Error>,
        T: api::Request,
    {
        let body = match musli::storage::to_vec(&request) {
            Ok(body) => body,
            Err(error) => {
                ctx.link()
                    .send_message(C::Message::from(Error::from(error)));
                return Request::default();
            }
        };

        let mut requests = self.shared.requests.borrow_mut();
        let serial = self.shared.serial.get();
        self.shared.serial.set(serial.wrapping_add(1));

        let link = ctx.link().clone();

        let pending = Pending {
            serial,
            callback: Box::new(move |result| {
                let raw = match result {
                    Ok(raw) => raw,
                    Err(error) => {
                        link.send_message(C::Message::from(error));
                        return;
                    }
                };

                link.send_message(C::Message::from(Packet {
                    raw,
                    _marker: PhantomData,
                }));
            }),
        };

        let index = requests.insert(pending) as u32;

        (self.shared.onmessage)(ClientRequest {
            header: api::RequestHeader {
                index,
                serial,
                kind: T::KIND,
            },
            body,
        });

        Request {
            inner: Some((self.shared.clone(), index)),
            _marker: PhantomData,
        }
    }

    /// List for broadcasts of type `T`.
    ///
    /// Returns a handle for the broadcasts.
    ///
    /// If the handle is dropped, the listener is cancelled.
    pub fn listen<C, T>(&self, ctx: &Context<C>) -> Listener<T>
    where
        C: Component,
        C::Message: From<Packet<T>> + From<Error>,
        T: api::Broadcast,
    {
        let mut broadcasts = self.shared.broadcasts.borrow_mut();

        let slots = broadcasts.entry(T::KIND).or_default();
        let link = ctx.link().clone();

        let index = slots.insert(Box::new(move |raw| {
            link.send_message(C::Message::from(Packet {
                raw,
                _marker: PhantomData,
            }));
        }));

        Listener {
            kind: T::KIND,
            index,
            shared: self.shared.clone(),
            _marker: PhantomData,
        }
    }

    /// Listen for state changes to the underlying connection.
    pub fn state_changes<C>(&self, ctx: &Context<C>) -> StateListener
    where
        C: Component,
        C::Message: From<State>,
    {
        let link = ctx.link().clone();
        let mut state = self.shared.state_changes.borrow_mut();

        let index = state.insert(Box::new(move |state| {
            link.send_message(C::Message::from(state))
        }));

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
    callback: Box<dyn Fn(Result<RawPacket>)>,
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
