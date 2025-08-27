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
//! use musli_web::yew021::*;
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

use core::cell::{Cell, RefCell};
use core::fmt;
use core::marker::PhantomData;
use core::mem;
use std::collections::VecDeque;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use std::collections::hash_map::{Entry, HashMap};

use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::storage;
use musli::{Decode, Encode};

use gloo_timers03::callback::Timeout;
use slab::Slab;
use wasm_bindgen02::closure::Closure;
use wasm_bindgen02::{JsCast, JsValue};
use web_sys03::js_sys::{ArrayBuffer, Math, Uint8Array};
use web_sys03::{BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket, window};

use crate::api;

const MAX_CAPACITY: usize = 1048576;

type StateSlab = Slab<Rc<dyn Fn(State)>>;
type Broadcasts = HashMap<&'static str, Slab<Rc<dyn Fn(RawPacket)>>>;

/// Construct a new [`ServiceBuilder`] associated with the given [`Connect`]
/// strategy.
pub fn connect(connect: Connect) -> ServiceBuilder {
    let shared = Rc::<Shared>::new_cyclic(|shared| {
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

        Shared {
            connect,
            state: Cell::new(State::Closed),
            opened: Cell::new(None),
            serial: Cell::new(0),
            state_listeners: RefCell::new(Slab::new()),
            requests: RefCell::new(Requests::new()),
            broadcasts: RefCell::new(HashMap::new()),
            deferred: RefCell::new(VecDeque::new()),
            socket: RefCell::new(None),
            output: RefCell::new(Vec::new()),
            current_timeout: Cell::new(INITIAL_TIMEOUT),
            reconnect_timeout: RefCell::new(None),
            handles: Handles {
                open,
                close,
                message,
                error,
            },
            on_error: None,
        }
    });

    ServiceBuilder { shared }
}

/// The state of the connection.
///
/// A listener for state changes can be set up through for example
/// [`yew021::HandleExt::on_state_change`].
///
/// [`yew021::HandleExt::on_state_change`]: crate::yew021::HandleExt::on_state_change
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
    Storage(storage::Error),
    Overflow(usize, usize),
}

impl Error {
    #[inline]
    fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    #[inline]
    fn msg(message: impl fmt::Display) -> Self {
        Self::new(ErrorKind::Message(message.to_string()))
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

impl From<storage::Error> for Error {
    #[inline]
    fn from(error: storage::Error) -> Self {
        Self::new(ErrorKind::Storage(error))
    }
}

impl From<JsValue> for Error {
    #[inline]
    fn from(error: JsValue) -> Self {
        Self::new(ErrorKind::Message(format!("{error:?}")))
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

const INITIAL_TIMEOUT: u32 = 250;
const MAX_TIMEOUT: u32 = 4000;

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

struct Handles {
    open: Closure<dyn Fn()>,
    close: Closure<dyn Fn(CloseEvent)>,
    message: Closure<dyn Fn(MessageEvent)>,
    error: Closure<dyn Fn(ErrorEvent)>,
}

struct Requests {
    serials: HashMap<u32, usize>,
    pending: Slab<Pending>,
}

impl Requests {
    fn new() -> Self {
        Self {
            serials: HashMap::new(),
            pending: Slab::new(),
        }
    }

    #[inline]
    fn remove(&mut self, serial: u32) -> Option<Pending> {
        let index = self.serials.remove(&serial)?;
        self.pending.try_remove(index)
    }

    #[inline]
    fn insert(&mut self, serial: u32, pending: Pending) -> usize {
        let index = self.pending.insert(pending);

        if let Some(existing) = self.serials.insert(serial, index) {
            _ = self.pending.try_remove(existing);
        }

        index
    }
}

/// Queue of deferred items.
type Deferred = VecDeque<Rc<dyn Fn(RawPacket)>>;

struct Shared {
    connect: Connect,
    state: Cell<State>,
    opened: Cell<Option<Opened>>,
    serial: Cell<u32>,
    state_listeners: RefCell<StateSlab>,
    requests: RefCell<Requests>,
    broadcasts: RefCell<Broadcasts>,
    deferred: RefCell<Deferred>,
    socket: RefCell<Option<WebSocket>>,
    output: RefCell<Vec<u8>>,
    current_timeout: Cell<u32>,
    reconnect_timeout: RefCell<Option<Timeout>>,
    on_error: Option<Box<dyn Fn(Error)>>,
    handles: Handles,
}

impl Drop for Shared {
    fn drop(&mut self) {
        if let Some(old) = self.socket.take()
            && let Err(error) = old.close()
            && let Some(handle) = &self.on_error
        {
            handle(Error::from(error));
        }

        // We don't need to worry about mutable borrows here, since we only have
        // weak references to Shared and by virtue of this being dropped they
        // are all invalid.
        let state_listeners = mem::take(&mut *self.state_listeners.borrow_mut());
        let mut requests = self.requests.borrow_mut();

        for (_, listener) in state_listeners {
            listener(State::Closed);
        }

        requests.serials.clear();

        for pending in requests.pending.drain() {
            if let Some(callback) = pending.callback {
                callback(Err(Error::msg("Websocket service closed")));
            }
        }
    }
}

/// Builder of a service.
pub struct ServiceBuilder {
    shared: Rc<Shared>,
}

impl ServiceBuilder {
    /// Set the error handler to use for the service.
    pub fn on_error_cb(mut self, on_error: impl Fn(Error) + 'static) -> Self {
        use core::ptr;

        let old;

        // SAFETY: This builder ensures that the service cannot be safely used
        // in a way which causes the error handler to be exposed.
        unsafe {
            let ptr = Rc::into_raw(self.shared).cast_mut();
            old = ptr::addr_of_mut!((*ptr).on_error).replace(Some(Box::new(on_error)));
            self.shared = Rc::from_raw(ptr.cast_const());
        }

        drop(old);
        self
    }

    /// Build a new service and handle.
    pub fn build(self) -> Service {
        let handle = Handle {
            shared: Rc::downgrade(&self.shared),
        };

        Service {
            shared: self.shared,
            handle,
        }
    }
}

/// The WebSocket service.
pub struct Service {
    shared: Rc<Shared>,
    handle: Handle,
}

impl Service {
    /// Attempt to establish a websocket connection.
    pub fn connect(&self) {
        Shared::connect(&self.shared)
    }

    /// Build a handle to the service.
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl Shared {
    fn do_open(&self) {
        tracing::debug!("Open event");
        self.set_open();
    }

    fn do_close(self: &Rc<Self>, e: CloseEvent) {
        tracing::debug!(code = e.code(), reason = e.reason(), "Close event");
        self.close();
    }

    fn do_message(self: &Rc<Shared>, e: MessageEvent) {
        tracing::debug!("Message event");

        if let Err(error) = self.message(e) {
            self.handle_error(error);
        }
    }

    fn do_error(self: &Rc<Self>, e: ErrorEvent) {
        tracing::debug!(message = e.message(), "Error event");

        self.close();
    }

    /// Defer an error.
    #[inline]
    fn handle_error(&self, error: Error) {
        if let Some(handle) = &self.on_error {
            handle(error);
        }
    }

    /// Send a client message.
    fn send_client_request<E, T>(&self, serial: u32, body: &T) -> Result<()>
    where
        E: api::Endpoint,
        T: Encode<Binary>,
    {
        let Some(ref socket) = *self.socket.borrow() else {
            return Err(Error::msg("Socket is not connected"));
        };

        let header = api::RequestHeader {
            serial,
            kind: E::KIND,
        };

        let out = &mut *self.output.borrow_mut();

        storage::to_writer(&mut *out, &header)?;
        storage::to_writer(&mut *out, &body)?;

        tracing::debug!(
            header.serial,
            header.kind,
            len = out.len(),
            "Sending request"
        );

        socket.send_with_u8_array(out.as_slice())?;

        out.clear();
        out.shrink_to(MAX_CAPACITY);
        Ok(())
    }

    fn message(&self, e: MessageEvent) -> Result<()> {
        let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() else {
            return Err(Error::msg("Expected message as ArrayBuffer"));
        };

        let body = Rc::from(Uint8Array::new(&array_buffer).to_vec());
        let mut reader = SliceReader::new(&body);

        let header: api::ResponseHeader<'_> = storage::decode(&mut reader)?;

        match header.broadcast {
            Some(kind) => {
                tracing::debug!(kind, "Got broadcast",);

                // Note: We need to defer this, since the outcome of calling the
                // broadcast callback might be that the broadcast listener is
                // modified, which could require mutable access to broadcasts.
                let mut deferred = self.deferred.borrow_mut();

                let kind = {
                    let broadcasts = self.broadcasts.borrow();

                    let Some((&kind, broadcasts)) = broadcasts.get_key_value(kind) else {
                        return Ok(());
                    };

                    for (_, callback) in broadcasts.iter() {
                        deferred.push_back(callback.clone());
                    }

                    kind
                };

                if deferred.is_empty() {
                    return Ok(());
                }

                let at = body.len().saturating_sub(reader.remaining());

                let packet = RawPacket {
                    body: body.clone(),
                    at: Cell::new(at),
                    kind,
                };

                let last = deferred.pop_back();

                while let Some(callback) = deferred.pop_front() {
                    callback(packet.clone());
                }

                if let Some(callback) = last {
                    callback(packet);
                }
            }
            None => {
                tracing::debug!(header.serial, "Got response");

                let (kind, callback) = {
                    let mut requests = self.requests.borrow_mut();

                    let Some(p) = requests.remove(header.serial) else {
                        tracing::warn!(header.serial, "Missing request");
                        return Ok(());
                    };

                    (p.kind, p.callback)
                };

                if let Some(error) = header.error {
                    if let Some(callback) = callback {
                        callback(Err(Error::msg(error)));
                    }
                } else if let Some(callback) = callback {
                    let at = body.len().saturating_sub(reader.remaining());

                    let packet = RawPacket {
                        body,
                        at: Cell::new(at),
                        kind,
                    };

                    callback(Ok(packet));
                }
            }
        }

        Ok(())
    }

    fn set_open(&self) {
        tracing::debug!("Set open");
        self.opened.set(Some(Opened { at: now() }));
        self.emit_state_change(State::Open);
    }

    fn is_open_for_a_while(&self) -> bool {
        let Some(opened) = self.opened.get() else {
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

    fn close(self: &Rc<Self>) {
        tracing::debug!("Close connection");

        // We need a weak reference back to shared state to handle the timeout.
        let shared = Rc::downgrade(self);

        tracing::debug!(
            "Set closed timeout={}, opened={:?}",
            self.current_timeout.get(),
            self.opened.get(),
        );

        if !self.is_open_for_a_while() {
            let current_timeout = self.current_timeout.get();

            if current_timeout < MAX_TIMEOUT {
                let fuzz = random(50);

                self.current_timeout.set(
                    current_timeout
                        .saturating_mul(2)
                        .saturating_add(fuzz)
                        .min(MAX_TIMEOUT),
                );
            }
        } else {
            self.current_timeout.set(INITIAL_TIMEOUT);
        }

        self.opened.set(None);
        self.emit_state_change(State::Closed);

        if let Some(old) = self.socket.take()
            && let Err(error) = old.close()
        {
            self.handle_error(Error::from(error));
        }

        let timeout = Timeout::new(self.current_timeout.get(), move || {
            if let Some(shared) = shared.upgrade() {
                Self::connect(&shared);
            }
        });

        drop(self.reconnect_timeout.borrow_mut().replace(timeout));
    }

    fn emit_state_change(&self, state: State) {
        if self.state.get() != state {
            self.state.set(state);

            for (_, callback) in self.state_listeners.borrow().iter() {
                callback(state);
            }
        }
    }

    fn is_closed(&self) -> bool {
        self.opened.get().is_none()
    }

    fn connect(self: &Rc<Self>) {
        tracing::debug!("Connect");

        let failed = {
            if let Err(error) = self.build() {
                self.handle_error(error);
                true
            } else {
                false
            }
        };

        if failed {
            self.close();
        }
    }

    /// Build a websocket connection.
    fn build(self: &Rc<Self>) -> Result<()> {
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

        ws.set_onopen(Some(self.handles.open.as_ref().unchecked_ref()));
        ws.set_onclose(Some(self.handles.close.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(self.handles.message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(self.handles.error.as_ref().unchecked_ref()));

        let old = self.socket.borrow_mut().replace(ws);

        if let Some(old) = old {
            old.close()?;
        }

        Ok(())
    }
}

/// A request builder .
///
/// Associate the callback to be used by using either
/// [`RequestBuilderExt::on_packet`] or [`RequestBuilderExt::on_raw_packet`]
/// depending on your needs.
///
/// Send the request with [`RequestBuilder::send`].
///
/// [`RequestBuilderExt::on_packet`]: crate::yew021::RequestBuilderExt::on_packet
/// [`RequestBuilderExt::on_raw_packet`]: crate::yew021::RequestBuilderExt::on_raw_packet
pub struct RequestBuilder<'a, E, T> {
    shared: &'a Weak<Shared>,
    callback: Option<Rc<dyn Fn(Result<RawPacket>)>>,
    body: T,
    _marker: PhantomData<E>,
}

impl<'a, E> RequestBuilder<'a, E, ()>
where
    E: api::Endpoint,
{
    /// Set the body of the request.
    #[inline]
    pub fn body<T>(self, body: T) -> RequestBuilder<'a, E, T>
    where
        T: api::Request<Endpoint = E>,
    {
        RequestBuilder {
            shared: self.shared,
            callback: self.callback,
            body,
            _marker: self._marker,
        }
    }
}

impl<'a, E, T> RequestBuilder<'a, E, T>
where
    E: api::Endpoint,
    T: Encode<Binary>,
{
    /// Build and return the request.
    pub fn send(self) -> Request {
        let Some(shared) = self.shared.upgrade() else {
            return Request::empty();
        };

        if shared.is_closed() {
            if let Some(callback) = self.callback {
                callback(Err(Error::msg("WebSocket is not connected")));
            }

            return Request::empty();
        }

        let serial = shared.serial.get();

        if let Err(error) = shared.send_client_request::<E, T>(serial, &self.body) {
            shared.handle_error(error);
            return Request::empty();
        }

        shared.serial.set(serial.wrapping_add(1));

        let pending = Pending {
            serial,
            callback: self.callback,
            kind: E::KIND,
        };

        shared.requests.borrow_mut().insert(serial, pending);

        Request {
            inner: Some((self.shared.clone(), serial)),
        }
    }
}

impl<'a, E, T> RequestBuilder<'a, E, T> {
    /// Handle the response using the specified callback.
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
    ///     OnHello(Result<ws::RawPacket, ws::Error>),
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
    ///         let link = ctx.link().clone();
    ///
    ///         let hello = ctx.props().ws
    ///             .request::<api::Hello>()
    ///             .body(api::HelloRequest { message: "Hello!"})
    ///             .on_raw_packet_cb(move |packet| link.send_message(Msg::OnHello(packet)))
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
    ///                 tracing::error!("Request error: {:?}", error);
    ///                 false
    ///             }
    ///             Msg::OnHello(Ok(packet)) => {
    ///                 if let Ok(response) = packet.decode::<api::HelloResponse>() {
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
    /// ```
    pub fn on_raw_packet_cb(mut self, f: impl Fn(Result<RawPacket, Error>) + 'static) -> Self {
        self.callback = Some(Rc::new(f));
        self
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
        let removed = {
            let Some((s, serial)) = self.inner.take() else {
                return;
            };

            let Some(s) = s.upgrade() else {
                return;
            };

            let mut requests = s.requests.borrow_mut();

            let Some(p) = requests.remove(serial) else {
                return;
            };

            p
        };

        drop(removed);
    }
}

/// The handle for a pending request. Dropping this handle cancels the request.
pub struct Listener {
    kind: &'static str,
    index: usize,
    shared: Weak<Shared>,
}

impl Listener {
    /// Set up an empty listener.
    pub fn empty_with_kind(kind: &'static str) -> Self {
        Self {
            kind,
            index: 0,
            shared: Weak::new(),
        }
    }

    /// Set up an empty listener.
    pub fn empty() -> Self {
        Self {
            kind: "",
            index: 0,
            shared: Weak::new(),
        }
    }
}

impl Drop for Listener {
    #[inline]
    fn drop(&mut self) {
        let mut removed = None;
        let mut removed_value = None;

        {
            if let Some(s) = self.shared.upgrade()
                && let Entry::Occupied(mut e) = s.broadcasts.borrow_mut().entry(self.kind)
            {
                removed = e.get_mut().try_remove(self.index);

                if e.get().is_empty() {
                    removed_value = Some(e.remove());
                }
            }
        }

        // Drop here, to avoid invoking any destructors which might borrow
        // shared mutably earlier.
        drop(removed);
        drop(removed_value);
    }
}

/// The handle for state change listening. Dropping this handle cancels the request.
pub struct StateListener {
    index: usize,
    shared: Weak<Shared>,
}

impl Drop for StateListener {
    #[inline]
    fn drop(&mut self) {
        let mut removed = None;

        if let Some(shared) = self.shared.upgrade() {
            removed = shared.state_listeners.borrow_mut().try_remove(self.index);
        }

        drop(removed);
    }
}

/// A raw packet of data.
#[derive(Clone)]
pub struct RawPacket {
    body: Rc<[u8]>,
    at: Cell<usize>,
    kind: &'static str,
}

impl RawPacket {
    /// Decode the contents of a raw packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`RawPacket::is_empty`].
    pub fn decode<'this, T>(&'this self) -> Result<T>
    where
        T: Decode<'this, Binary, Global>,
    {
        let at = self.at.get();

        let Some(bytes) = self.body.get(at..) else {
            return Err(Error::new(ErrorKind::Overflow(at, self.body.len())));
        };

        let mut reader = SliceReader::new(bytes);

        match storage::decode(&mut reader) {
            Ok(value) => {
                self.at.set(at + bytes.len() - reader.remaining());
                Ok(value)
            }
            Err(error) => {
                self.at.set(self.body.len());
                Err(Error::from(error))
            }
        }
    }

    /// Check if the packet is empty.
    pub fn is_empty(&self) -> bool {
        self.at.get() >= self.body.len()
    }

    /// The kind of the packet this is a response to as specified by
    /// [`Endpoint::KIND`].
    ///
    /// [`Endpoint::KIND`]: crate::api::Endpoint::KIND
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
    /// Construct a new typed package from a raw one.
    ///
    /// Note that this does not guarantee that the typed package is correct, but
    /// the `T` parameter becomes associated with it allowing it to be used
    /// automatically with methods such as [`Packet::decode`].
    #[inline]
    pub fn new(raw: RawPacket) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    /// Convert a packet into a raw packet.
    ///
    /// To determine which endpoint or broadcast it belongs to the
    /// [`RawPacket::kind`] method can be used.
    pub fn into_raw(self) -> RawPacket {
        self.raw
    }

    /// Check if the packet is empty.
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// The kind of the packet this is a response to as specified by
    /// [`Endpoint::KIND`].
    ///
    /// [`Endpoint::KIND`]: crate::api::Endpoint::KIND
    pub fn kind(&self) -> &str {
        self.raw.kind()
    }
}

impl<T> Packet<T>
where
    T: api::Endpoint,
{
    /// Decode the contents of a packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`Packet::is_empty`].
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
#[repr(transparent)]
pub struct Handle {
    shared: Weak<Shared>,
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
    /// use musli_web::yew021::*;
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
    ///                 tracing::error!("Request error: {:?}", error);
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
    /// ```
    pub fn request<E>(&self) -> RequestBuilder<'_, E, ()>
    where
        E: api::Endpoint,
    {
        RequestBuilder {
            shared: &self.shared,
            callback: None,
            body: (),
            _marker: PhantomData,
        }
    }

    /// Listen for broadcasts of type `T`.
    ///
    /// Returns a handle for the listener that will cancel the listener if
    /// dropped.
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
    ///     Tick(ws::RawPacket),
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
    ///         let link = ctx.link().clone();
    ///         let listen = ctx.props().ws.listen_cb::<api::Tick>(move |packet| link.send_message(Msg::Tick(packet)));
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
    ///                 if let Ok(tick) = packet.decode::<api::TickEvent>() {
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
    /// ```
    pub fn listen_cb<T>(&self, f: impl Fn(RawPacket) + 'static) -> Listener
    where
        T: api::Listener,
    {
        let Some(shared) = self.shared.upgrade() else {
            return Listener::empty_with_kind(T::KIND);
        };

        let index = {
            let mut broadcasts = shared.broadcasts.borrow_mut();
            let slots = broadcasts.entry(T::KIND).or_default();
            slots.insert(Rc::new(f))
        };

        Listener {
            kind: T::KIND,
            index,
            shared: self.shared.clone(),
        }
    }

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
    ///         let link = ctx.link().clone();
    ///
    ///         let (state, listen) = ctx.props().ws.on_state_change_cb(move |state| {
    ///             link.send_message(Msg::StateChange(state));
    ///         });
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
    pub fn on_state_change_cb(&self, f: impl Fn(State) + 'static) -> (State, StateListener) {
        let Some(shared) = self.shared.upgrade() else {
            return (
                State::Closed,
                StateListener {
                    index: 0,
                    shared: Weak::new(),
                },
            );
        };

        let (state, index) = {
            let index = shared.state_listeners.borrow_mut().insert(Rc::new(f));
            (shared.state.get(), index)
        };

        let listener = StateListener {
            index,
            shared: self.shared.clone(),
        };

        (state, listener)
    }
}

impl PartialEq for Handle {
    #[inline]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

fn random(range: u32) -> u32 {
    ((Math::random() * range as f64).round() as u32).min(range)
}

fn now() -> Option<f64> {
    Some(window()?.performance()?.now())
}

struct Pending {
    serial: u32,
    callback: Option<Rc<dyn Fn(Result<RawPacket>)>>,
    kind: &'static str,
}

impl fmt::Debug for Pending {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pending")
            .field("serial", &self.serial)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
struct Opened {
    at: Option<f64>,
}
