//! The generic web implementation.
//!
//! This is specialized over the `H` parameter through modules such as:
//!
//! * [`web03`] for `web-sys` `0.3.x`.
//!
//! [`web03`]: crate::web03

use core::cell::{Cell, RefCell};
use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::ops::Deref;
use core::ptr::NonNull;
use std::collections::VecDeque;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use std::collections::hash_map::{Entry, HashMap};

use musli::Decode;
use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::storage;

use slab::Slab;

use crate::api::{self, Event, MessageId};

const MAX_CAPACITY: usize = 1048576;

/// An empty request body.
#[non_exhaustive]
pub struct EmptyBody;

/// An empty callback.
#[non_exhaustive]
pub struct EmptyCallback;

/// Slab of state listeners.
type StateListeners = Slab<Rc<dyn Callback<State>>>;
/// Slab of broadcast listeners.
type Broadcasts = HashMap<MessageId, Slab<Rc<dyn Callback<Result<RawPacket>>>>>;
/// Queue of recycled buffers.
type Buffers = VecDeque<Box<BufData>>;
/// Pending requests.
type Requests = HashMap<u32, Box<Pending<dyn Callback<Result<RawPacket>>>>>;

/// Location information for websocket implementation.
#[doc(hidden)]
pub struct Location {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: String,
}

pub(crate) mod sealed_socket {
    pub trait Sealed {}
}

pub(crate) trait SocketImpl
where
    Self: Sized + self::sealed_socket::Sealed,
{
    #[doc(hidden)]
    type Handles;

    #[doc(hidden)]
    fn new(url: &str, handles: &Self::Handles) -> Result<Self, Error>;

    #[doc(hidden)]
    fn send(&self, data: &[u8]) -> Result<(), Error>;

    #[doc(hidden)]
    fn close(self) -> Result<(), Error>;
}

pub(crate) mod sealed_performance {
    pub trait Sealed {}
}

pub trait PerformanceImpl
where
    Self: Sized + self::sealed_performance::Sealed,
{
    #[doc(hidden)]
    fn now(&self) -> f64;
}

pub(crate) mod sealed_window {
    pub trait Sealed {}
}

pub(crate) trait WindowImpl
where
    Self: Sized + self::sealed_window::Sealed,
{
    #[doc(hidden)]
    type Performance: PerformanceImpl;

    #[doc(hidden)]
    type Timeout;

    #[doc(hidden)]
    fn new() -> Result<Self, Error>;

    #[doc(hidden)]
    fn performance(&self) -> Result<Self::Performance, Error>;

    #[doc(hidden)]
    fn location(&self) -> Result<Location, Error>;

    #[doc(hidden)]
    fn set_timeout(
        &self,
        millis: u32,
        callback: impl FnOnce() + 'static,
    ) -> Result<Self::Timeout, Error>;
}

pub(crate) mod sealed_web {
    pub trait Sealed {}
}

/// Central trait for web integration.
///
/// Since web integration is currently unstable, this requires multiple
/// different implementations, each time an ecosystem breaking change is
/// released.
///
/// The crate in focus here is `web-sys`, and the corresponding modules provide
/// integrations:
///
/// * [web03] for `web-sys` `0.3.x`.
///
/// [web03]: crate::web03
pub trait WebImpl
where
    Self: 'static + Copy + Sized + self::sealed_web::Sealed,
{
    #[doc(hidden)]
    #[allow(private_bounds)]
    type Window: WindowImpl;

    #[doc(hidden)]
    type Handles;

    #[doc(hidden)]
    #[allow(private_bounds)]
    type Socket: SocketImpl<Handles = Self::Handles>;

    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn handles(shared: &Weak<Shared<Self>>) -> Self::Handles;

    #[doc(hidden)]
    fn random(range: u32) -> u32;
}

/// Construct a new [`ServiceBuilder`] associated with the given [`Connect`]
/// strategy.
pub fn connect<H>(connect: Connect) -> ServiceBuilder<H, EmptyCallback>
where
    H: WebImpl,
{
    ServiceBuilder {
        connect,
        on_error: EmptyCallback,
        _marker: PhantomData,
    }
}

/// The state of the connection.
///
/// A listener for state changes can be set up through for example
/// [`Handle::on_state_change`].
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
    pub(crate) fn msg(message: impl fmt::Display) -> Self {
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

#[cfg(feature = "wasm_bindgen02")]
impl From<wasm_bindgen02::JsValue> for Error {
    #[inline]
    fn from(error: wasm_bindgen02::JsValue) -> Self {
        Self::new(ErrorKind::Message(format!("{error:?}")))
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

const INITIAL_TIMEOUT: u32 = 250;
const MAX_TIMEOUT: u32 = 4000;

/// How to connect to the websocket.
enum ConnectKind {
    Location { path: String },
    Url { url: String },
}

/// A specification for how to connect a websocket.
pub struct Connect {
    kind: ConnectKind,
}

impl Connect {
    /// Connect to the same location with a custom path.
    ///
    /// Note that any number of `/` prefixes are ignored, the canonical
    /// representation always ignores them and the path is relative to the
    /// current location.
    #[inline]
    pub fn location(path: impl AsRef<str>) -> Self {
        Self {
            kind: ConnectKind::Location {
                path: String::from(path.as_ref()),
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

/// Generic but shared fields which do not depend on specialization over `H`.
struct Generic {
    state_listeners: RefCell<StateListeners>,
    requests: RefCell<Requests>,
    broadcasts: RefCell<Broadcasts>,
    buffers: RefCell<Buffers>,
}

/// Shared implementation details for websocket implementations.
pub(crate) struct Shared<H>
where
    H: WebImpl,
{
    connect: Connect,
    pub(crate) on_error: Box<dyn Callback<Error>>,
    window: H::Window,
    performance: <H::Window as WindowImpl>::Performance,
    handles: H::Handles,
    state: Cell<State>,
    opened: Cell<Option<f64>>,
    serial: Cell<u32>,
    defer_broadcasts: RefCell<VecDeque<Weak<dyn Callback<Result<RawPacket>>>>>,
    defer_state_listeners: RefCell<VecDeque<Weak<dyn Callback<State>>>>,
    pub(crate) socket: RefCell<Option<H::Socket>>,
    output: RefCell<Vec<u8>>,
    current_timeout: Cell<u32>,
    reconnect_timeout: RefCell<Option<<H::Window as WindowImpl>::Timeout>>,
    g: Rc<Generic>,
}

impl<H> Drop for Shared<H>
where
    H: WebImpl,
{
    fn drop(&mut self) {
        if let Some(s) = self.socket.take()
            && let Err(e) = s.close()
        {
            self.on_error.call(e);
        }

        // We don't need to worry about mutable borrows here, since we only have
        // weak references to Shared and by virtue of this being dropped they
        // are all invalid.
        let state_listeners = mem::take(&mut *self.g.state_listeners.borrow_mut());
        let mut requests = self.g.requests.borrow_mut();

        for (_, listener) in state_listeners {
            listener.call(State::Closed);
        }

        for (_, p) in requests.drain() {
            p.callback.call(Err(Error::msg("Websocket service closed")));
        }
    }
}

/// Builder of a service.
pub struct ServiceBuilder<H, E>
where
    H: WebImpl,
{
    connect: Connect,
    on_error: E,
    _marker: PhantomData<H>,
}

impl<H, E> ServiceBuilder<H, E>
where
    H: WebImpl,
    E: Callback<Error>,
{
    /// Set the error handler to use for the service.
    pub fn on_error<U>(self, on_error: U) -> ServiceBuilder<H, U>
    where
        U: Callback<Error>,
    {
        ServiceBuilder {
            connect: self.connect,
            on_error,
            _marker: self._marker,
        }
    }

    /// Build a new service and handle.
    pub fn build(self) -> Service<H> {
        let window = match H::Window::new() {
            Ok(window) => window,
            Err(error) => {
                panic!("{error}")
            }
        };

        let performance = match WindowImpl::performance(&window) {
            Ok(performance) => performance,
            Err(error) => {
                panic!("{error}")
            }
        };

        let shared = Rc::<Shared<H>>::new_cyclic(move |shared| Shared {
            connect: self.connect,
            on_error: Box::new(self.on_error),
            window,
            performance,
            handles: H::handles(shared),
            state: Cell::new(State::Closed),
            opened: Cell::new(None),
            serial: Cell::new(0),
            defer_broadcasts: RefCell::new(VecDeque::new()),
            defer_state_listeners: RefCell::new(VecDeque::new()),
            socket: RefCell::new(None),
            output: RefCell::new(Vec::new()),
            current_timeout: Cell::new(INITIAL_TIMEOUT),
            reconnect_timeout: RefCell::new(None),
            g: Rc::new(Generic {
                state_listeners: RefCell::new(Slab::new()),
                broadcasts: RefCell::new(HashMap::new()),
                requests: RefCell::new(Requests::new()),
                buffers: RefCell::new(VecDeque::new()),
            }),
        });

        let handle = Handle {
            shared: Rc::downgrade(&shared),
        };

        Service { shared, handle }
    }
}

/// The service handle.
///
/// Once dropped this will cause the service to be disconnected and all requests
/// to be cancelled.
pub struct Service<H>
where
    H: WebImpl,
{
    shared: Rc<Shared<H>>,
    handle: Handle<H>,
}

impl<H> Service<H>
where
    H: WebImpl,
{
    /// Attempt to establish a websocket connection.
    pub fn connect(&self) {
        self.shared.connect()
    }

    /// Build a handle to the service.
    pub fn handle(&self) -> &Handle<H> {
        &self.handle
    }
}

impl<H> Shared<H>
where
    H: WebImpl,
{
    /// Send a client message.
    fn send_client_request<T>(&self, serial: u32, body: &T) -> Result<()>
    where
        T: api::Request,
    {
        let Some(ref socket) = *self.socket.borrow() else {
            return Err(Error::msg("Socket is not connected"));
        };

        let header = api::RequestHeader {
            serial,
            id: <T::Endpoint as api::Endpoint>::ID.get(),
        };

        let out = &mut *self.output.borrow_mut();

        storage::to_writer(&mut *out, &header)?;
        storage::to_writer(&mut *out, &body)?;

        tracing::debug!(
            header.serial,
            ?header.id,
            len = out.len(),
            "Sending request"
        );

        socket.send(out.as_slice())?;

        out.clear();
        out.shrink_to(MAX_CAPACITY);
        Ok(())
    }

    pub(crate) fn next_buffer(self: &Rc<Self>, needed: usize) -> Box<BufData> {
        match self.g.buffers.borrow_mut().pop_front() {
            Some(mut buf) => {
                if buf.data.capacity() < needed {
                    buf.data.reserve(needed - buf.data.len());
                }

                buf
            }
            None => Box::new(BufData::with_capacity(Rc::downgrade(&self.g), needed)),
        }
    }

    pub(crate) fn message(self: &Rc<Self>, buf: Box<BufData>) -> Result<()> {
        // Wrap the buffer in a simple shared reference-counted container.
        let buf = BufRc::new(buf);
        let mut reader = SliceReader::new(&buf);

        let header: api::ResponseHeader = storage::decode(&mut reader)?;

        if let Some(broadcast) = MessageId::new(header.broadcast) {
            tracing::debug!(?header.broadcast, "Got broadcast");

            if !self.prepare_broadcast(broadcast) {
                return Ok(());
            };

            if let Some(id) = MessageId::new(header.error) {
                let error = if id == MessageId::ERROR_MESSAGE {
                    storage::decode(&mut reader)?
                } else {
                    api::ErrorMessage {
                        message: "Unknown error",
                    }
                };

                while let Some(callback) = self.defer_broadcasts.borrow_mut().pop_front() {
                    if let Some(callback) = callback.upgrade() {
                        callback.call(Err(Error::msg(error.message)));
                    }
                }

                return Ok(());
            }

            let at = buf.len().saturating_sub(reader.remaining());

            let packet = RawPacket {
                buf: buf.clone(),
                at: Cell::new(at),
                id: broadcast,
            };

            while let Some(callback) = self.defer_broadcasts.borrow_mut().pop_front() {
                if let Some(callback) = callback.upgrade() {
                    callback.call(Ok(packet.clone()));
                }
            }
        } else {
            tracing::debug!(header.serial, "Got response");

            let p = {
                let mut requests = self.g.requests.borrow_mut();

                let Some(p) = requests.remove(&header.serial) else {
                    tracing::debug!(header.serial, "Missing request");
                    return Ok(());
                };

                p
            };

            if let Some(id) = MessageId::new(header.error) {
                let error = if id == MessageId::ERROR_MESSAGE {
                    storage::decode(&mut reader)?
                } else {
                    api::ErrorMessage {
                        message: "Unknown error",
                    }
                };

                p.callback.call(Err(Error::msg(error.message)));
                return Ok(());
            }

            let at = buf.len().saturating_sub(reader.remaining());

            let packet = RawPacket {
                id: p.kind,
                buf,
                at: Cell::new(at),
            };

            p.callback.call(Ok(packet));
        }

        Ok(())
    }

    fn prepare_broadcast(self: &Rc<Self>, kind: MessageId) -> bool {
        // Note: We need to defer this, since the outcome of calling
        // the broadcast callback might be that the broadcast
        // listener is modified, which could require mutable access
        // to broadcasts.
        let mut defer = self.defer_broadcasts.borrow_mut();

        let broadcasts = self.g.broadcasts.borrow();

        let Some(broadcasts) = broadcasts.get(&kind) else {
            return false;
        };

        for (_, callback) in broadcasts.iter() {
            defer.push_back(Rc::downgrade(callback));
        }

        !defer.is_empty()
    }

    pub(crate) fn set_open(&self) {
        tracing::debug!("Set open");
        self.opened
            .set(Some(PerformanceImpl::now(&self.performance)));
        self.emit_state_change(State::Open);
    }

    fn is_open_for_a_while(&self) -> bool {
        let Some(at) = self.opened.get() else {
            return false;
        };

        let now = PerformanceImpl::now(&self.performance);
        (now - at) >= 250.0
    }

    pub(crate) fn close(self: &Rc<Self>) -> Result<(), Error> {
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
                let fuzz = H::random(50);

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

        if let Some(s) = self.socket.take() {
            s.close()?;
        }

        self.close_pending();

        let timeout = self
            .window
            .set_timeout(self.current_timeout.get(), move || {
                if let Some(shared) = shared.upgrade() {
                    Self::connect(&shared);
                }
            })?;

        drop(self.reconnect_timeout.borrow_mut().replace(timeout));
        Ok(())
    }

    /// Close an pending requests with an error, since there is no chance they
    /// will be responded to any more.
    fn close_pending(self: &Rc<Self>) {
        loop {
            let Some(serial) = self.g.requests.borrow().keys().next().copied() else {
                break;
            };

            let p = {
                let mut requests = self.g.requests.borrow_mut();

                let Some(p) = requests.remove(&serial) else {
                    break;
                };

                p
            };

            p.callback.call(Err(Error::msg("Connection closed")));
        }
    }

    fn emit_state_change(&self, state: State) {
        if self.state.get() == state {
            return;
        }

        self.state.set(state);

        {
            // We need to collect callbacks to avoid the callback recursively
            // borrowing state listeners, which it would if it modifies any
            // existing state listeners.
            let mut defer = self.defer_state_listeners.borrow_mut();

            for (_, callback) in self.g.state_listeners.borrow().iter() {
                defer.push_back(Rc::downgrade(callback));
            }

            if defer.is_empty() {
                return;
            }
        }

        while let Some(callback) = self.defer_state_listeners.borrow_mut().pop_front() {
            if let Some(callback) = callback.upgrade() {
                callback.call(state);
            }
        }
    }

    fn is_closed(&self) -> bool {
        self.opened.get().is_none()
    }

    fn connect(self: &Rc<Self>) {
        tracing::debug!("Connect");

        if let Err(e) = self.build() {
            self.on_error.call(e);
        } else {
            return;
        }

        if let Err(e) = self.close() {
            self.on_error.call(e);
        }
    }

    /// Build a websocket connection.
    fn build(self: &Rc<Self>) -> Result<()> {
        let url = match &self.connect.kind {
            ConnectKind::Location { path } => {
                let Location {
                    protocol,
                    host,
                    port,
                } = WindowImpl::location(&self.window)?;

                let protocol = match protocol.as_str() {
                    "https:" => "wss:",
                    "http:" => "ws:",
                    other => {
                        return Err(Error::msg(format_args!(
                            "Same host connection is not supported for protocol `{other}`"
                        )));
                    }
                };

                let path = ForcePrefix(path, '/');
                format!("{protocol}//{host}:{port}{path}")
            }
            ConnectKind::Url { url } => url.clone(),
        };

        let ws = SocketImpl::new(&url, &self.handles)?;

        let old = self.socket.borrow_mut().replace(ws);

        if let Some(old) = old {
            old.close()?;
        }

        Ok(())
    }
}

/// Trait governing how callbacks are called.
pub trait Callback<I>
where
    Self: 'static,
{
    /// Call the callback.
    fn call(&self, input: I);
}

impl<I> Callback<I> for EmptyCallback {
    #[inline]
    fn call(&self, _: I) {}
}

impl<F, I> Callback<I> for F
where
    F: 'static + Fn(I),
{
    #[inline]
    fn call(&self, input: I) {
        self(input)
    }
}

/// A request builder .
///
/// Associate the callback to be used by using either
/// [`RequestBuilder::on_packet`] or [`RequestBuilder::on_raw_packet`] depending
/// on your needs.
///
/// Send the request with [`RequestBuilder::send`].
pub struct RequestBuilder<'a, H, B, C>
where
    H: WebImpl,
{
    shared: &'a Weak<Shared<H>>,
    body: B,
    callback: C,
}

impl<'a, H, B, C> RequestBuilder<'a, H, B, C>
where
    H: WebImpl,
{
    /// Set the body of the request.
    #[inline]
    pub fn body<U>(self, body: U) -> RequestBuilder<'a, H, U, C>
    where
        U: api::Request,
    {
        RequestBuilder {
            shared: self.shared,
            body,
            callback: self.callback,
        }
    }

    /// Handle the response using the specified callback.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::web03::prelude::*;
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
    ///         pub type Hello;
    ///
    ///         impl Endpoint for Hello {
    ///             impl<'de> Request for HelloRequest<'de>;
    ///             type Response<'de> = HelloResponse<'de>;
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
    ///             .request()
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
    pub fn on_packet<E>(
        self,
        callback: impl Callback<Result<Packet<E>>>,
    ) -> RequestBuilder<'a, H, B, impl Callback<Result<RawPacket>>>
    where
        E: api::Endpoint,
    {
        self.on_raw_packet(move |result: Result<RawPacket>| match result {
            Ok(ok) => callback.call(Ok(Packet::new(ok))),
            Err(err) => callback.call(Err(err)),
        })
    }

    /// Handle the response using the specified callback.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::web03::prelude::*;
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
    ///         pub type Hello;
    ///
    ///         impl Endpoint for Hello {
    ///             impl<'de> Request for HelloRequest<'de>;
    ///             type Response<'de> = HelloResponse<'de>;
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
    ///             .request()
    ///             .body(api::HelloRequest { message: "Hello!"})
    ///             .on_raw_packet(move |packet| link.send_message(Msg::OnHello(packet)))
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
    pub fn on_raw_packet<U>(self, callback: U) -> RequestBuilder<'a, H, B, U>
    where
        U: Callback<Result<RawPacket, Error>>,
    {
        RequestBuilder {
            shared: self.shared,
            body: self.body,
            callback,
        }
    }
}

impl<'a, H, B, C> RequestBuilder<'a, H, B, C>
where
    B: api::Request,
    C: Callback<Result<RawPacket>>,
    H: WebImpl,
{
    /// Send the request.
    ///
    /// This requires that a body has been set using [`RequestBuilder::body`].
    pub fn send(self) -> Request {
        let Some(shared) = self.shared.upgrade() else {
            self.callback
                .call(Err(Error::msg("WebSocket service is down")));
            return Request::new();
        };

        if shared.is_closed() {
            self.callback
                .call(Err(Error::msg("WebSocket is not connected")));
            return Request::new();
        }

        let serial = shared.serial.get();

        if let Err(error) = shared.send_client_request(serial, &self.body) {
            shared.on_error.call(error);
            return Request::new();
        }

        shared.serial.set(serial.wrapping_add(1));

        let pending = Pending {
            kind: <B::Endpoint as api::Endpoint>::ID,
            serial,
            callback: self.callback,
        };

        let existing = shared
            .g
            .requests
            .borrow_mut()
            .insert(serial, Box::new(pending));

        if let Some(p) = existing {
            p.callback.call(Err(Error::msg("Request cancelled")));
        }

        Request {
            serial,
            g: Rc::downgrade(&shared.g),
        }
    }
}

/// The handle for a pending request.
///
/// Dropping or [`clear()`] this handle will cancel the request.
///
/// [`clear()`]: Self::clear
pub struct Request {
    serial: u32,
    g: Weak<Generic>,
}

impl Request {
    /// An empty request handler.
    #[inline]
    pub const fn new() -> Self {
        Self {
            serial: 0,
            g: Weak::new(),
        }
    }

    /// Clear the request handle without dropping it, cancelling any pending
    /// requests.
    pub fn clear(&mut self) {
        let removed = {
            let serial = mem::take(&mut self.serial);

            let Some(g) = self.g.upgrade() else {
                return;
            };

            self.g = Weak::new();

            let Some(p) = g.requests.borrow_mut().remove(&serial) else {
                return;
            };

            p
        };

        drop(removed);
    }
}

impl Default for Request {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Request {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

/// The handle for a pending request.
///
/// Dropping or calling [`clear()`] on this handle remove the listener.
///
/// [`clear()`]: Self::clear
pub struct Listener {
    kind: Option<MessageId>,
    index: usize,
    g: Weak<Generic>,
}

impl Listener {
    /// Construct an empty listener.
    #[inline]
    pub const fn new() -> Self {
        Self {
            kind: None,
            index: 0,
            g: Weak::new(),
        }
    }

    /// Build up an empty listener with the specified kind.
    #[inline]
    pub(crate) const fn empty_with_kind(kind: MessageId) -> Self {
        Self {
            kind: Some(kind),
            index: 0,
            g: Weak::new(),
        }
    }

    /// Clear the listener without dropping it.
    ///
    /// This will remove the associated broadcast listener from being notified.
    pub fn clear(&mut self) {
        // Gather values here to drop them outside of the upgrade block.
        let removed;
        let removed_value;

        {
            let Some(g) = self.g.upgrade() else {
                return;
            };

            self.g = Weak::new();
            let index = mem::take(&mut self.index);

            let Some(kind) = self.kind.take() else {
                return;
            };

            let mut broadcasts = g.broadcasts.borrow_mut();

            let Entry::Occupied(mut e) = broadcasts.entry(kind) else {
                return;
            };

            removed = e.get_mut().try_remove(index);

            if e.get().is_empty() {
                removed_value = Some(e.remove());
            } else {
                removed_value = None;
            }
        }

        // Drop here, to avoid invoking any destructors which might borrow
        // shared mutably earlier.
        drop(removed);
        drop(removed_value);
    }
}

impl Default for Listener {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Listener {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

/// The handle for state change listening.
///
/// Dropping or calling [`clear()`] on this handle will remove the associated
/// callback from being notified.
///
/// [`clear()`]: Self::clear
pub struct StateListener {
    index: usize,
    g: Weak<Generic>,
}

impl StateListener {
    /// Construct an empty state listener.
    #[inline]
    pub const fn new() -> Self {
        Self {
            index: 0,
            g: Weak::new(),
        }
    }

    /// Clear the state listener without dropping it.
    ///
    /// This will remove the associated callback from being notified.
    pub fn clear(&mut self) {
        let removed = {
            let Some(g) = self.g.upgrade() else {
                return;
            };

            self.g = Weak::new();

            g.state_listeners.borrow_mut().try_remove(self.index)
        };

        drop(removed);
    }
}

impl Default for StateListener {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StateListener {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

pub(crate) struct BufData {
    /// Buffer being used.
    pub(crate) data: Vec<u8>,
    /// Number of strong references to this buffer.
    strong: Cell<usize>,
    /// Reference to shared state where the buffer will be recycled to.
    g: Weak<Generic>,
}

impl BufData {
    fn with_capacity(g: Weak<Generic>, capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            strong: Cell::new(0),
            g,
        }
    }

    unsafe fn dec(ptr: NonNull<BufData>) {
        unsafe {
            let count = ptr.as_ref().strong.get().wrapping_sub(1);
            ptr.as_ref().strong.set(count);

            if count > 0 {
                return;
            }

            let mut buf = Box::from_raw(ptr.as_ptr());

            // Try to recycle the buffer if shared is available, else let it be
            // dropped and free here.
            let Some(g) = buf.as_ref().g.upgrade() else {
                return;
            };

            let mut buffers = g.buffers.borrow_mut();

            // Set the length of the recycled buffer.
            buf.data.set_len(buf.data.len().min(MAX_CAPACITY));

            // We size our buffers to some max capacity to avod overuse in case
            // we infrequently need to handle some massive message. If we don't
            // shrink the allocation, then memory use can run away over time.
            buf.data.shrink_to(MAX_CAPACITY);

            buffers.push_back(buf);
        }
    }

    unsafe fn inc(ptr: NonNull<BufData>) {
        unsafe {
            let count = ptr.as_ref().strong.get().wrapping_add(1);

            if count == 0 {
                std::process::abort();
            }

            ptr.as_ref().strong.set(count);
        }
    }
}

/// A shared buffer of data that is recycled when dropped.
struct BufRc {
    data: NonNull<BufData>,
}

impl BufRc {
    fn new(data: Box<BufData>) -> Self {
        let data = NonNull::from(Box::leak(data));

        unsafe {
            BufData::inc(data);
        }

        Self { data }
    }
}

impl Deref for BufRc {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.data.as_ptr()).data }
    }
}

impl Clone for BufRc {
    fn clone(&self) -> Self {
        unsafe {
            BufData::inc(self.data);
        }

        Self { data: self.data }
    }
}

impl Drop for BufRc {
    fn drop(&mut self) {
        unsafe {
            BufData::dec(self.data);
        }
    }
}

/// A raw packet of data.
#[derive(Clone)]
pub struct RawPacket {
    id: MessageId,
    buf: BufRc,
    at: Cell<usize>,
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

        let Some(bytes) = self.buf.get(at..) else {
            return Err(Error::new(ErrorKind::Overflow(at, self.buf.len())));
        };

        let mut reader = SliceReader::new(bytes);

        match storage::decode(&mut reader) {
            Ok(value) => {
                self.at.set(at + bytes.len() - reader.remaining());
                Ok(value)
            }
            Err(error) => {
                self.at.set(self.buf.len());
                Err(Error::from(error))
            }
        }
    }

    /// Check if the packet is empty.
    pub fn is_empty(&self) -> bool {
        self.at.get() >= self.buf.len()
    }

    /// The id of the packet this is a response to as specified by
    /// [`Endpoint::ID`] or [`Broadcast::ID`].
    ///
    /// [`Endpoint::ID`]: crate::api::Endpoint::ID
    /// [`Broadcast::ID`]: crate::api::Broadcast::ID
    pub fn id(&self) -> MessageId {
        self.id
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
    /// [`RawPacket::id`] method can be used.
    pub fn into_raw(self) -> RawPacket {
        self.raw
    }

    /// Check if the packet is empty.
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// The id of the packet this is a response to as specified by
    /// [`Endpoint::ID`] or [`Broadcast::ID`].
    ///
    /// [`Endpoint::ID`]: crate::api::Endpoint::ID
    /// [`Broadcast::ID`]: crate::api::Broadcast::ID
    pub fn id(&self) -> MessageId {
        self.raw.id()
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
        self.decode_any()
    }

    /// Decode any contents of a packet.
    ///
    /// This can be called multiple times if there are multiple payloads in
    /// sequence of the response.
    ///
    /// You can check if the packet is empty using [`Packet::is_empty`].
    pub fn decode_any<'de, R>(&'de self) -> Result<R>
    where
        R: Decode<'de, Binary, Global>,
    {
        self.raw.decode()
    }
}

impl<T> Packet<T>
where
    T: api::Broadcast,
{
    /// Decode the primary event related to a broadcast.
    pub fn decode_event<'de>(&'de self) -> Result<T::Event<'de>> {
        self.decode_event_any()
    }

    /// Decode any event related to a broadcast.
    pub fn decode_event_any<'de, E>(&'de self) -> Result<E>
    where
        E: Event<Broadcast = T> + Decode<'de, Binary, Global>,
    {
        self.raw.decode()
    }
}

/// A handle to the WebSocket service.
#[derive(Clone)]
#[repr(transparent)]
pub struct Handle<H>
where
    H: WebImpl,
{
    shared: Weak<Shared<H>>,
}

impl<H> Handle<H>
where
    H: WebImpl,
{
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
    /// use musli_web::web03::prelude::*;
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
    ///         pub type Hello;
    ///
    ///         impl Endpoint for Hello {
    ///             impl<'de> Request for HelloRequest<'de>;
    ///             type Response<'de> = HelloResponse<'de>;
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
    ///             .request()
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
    pub fn request(&self) -> RequestBuilder<'_, H, EmptyBody, EmptyCallback> {
        RequestBuilder {
            shared: &self.shared,
            body: EmptyBody,
            callback: EmptyCallback,
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
    /// use musli_web::web03::prelude::*;
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
    ///         pub type Tick;
    ///
    ///         impl Broadcast for Tick {
    ///             impl<'de> Event for TickEvent<'de>;
    ///         }
    ///     }
    /// }
    ///
    /// enum Msg {
    ///     Tick(Result<ws::Packet<api::Tick>, ws::Error>),
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
    ///         let listen = ctx.props().ws.on_broadcast(ctx.link().callback(Msg::Tick));
    ///
    ///         Self {
    ///             tick: 0,
    ///             _listen: listen,
    ///         }
    ///     }
    ///
    ///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    ///         match msg {
    ///             Msg::Tick(Err(error)) => {
    ///                 tracing::error!("Tick error: {error}");
    ///                 false
    ///             }
    ///             Msg::Tick(Ok(packet)) => {
    ///                 if let Ok(tick) = packet.decode_event() {
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
    pub fn on_broadcast<T>(&self, callback: impl Callback<Result<Packet<T>>>) -> Listener
    where
        T: api::Broadcast,
    {
        self.on_raw_broadcast::<T>(move |result| match result {
            Ok(packet) => callback.call(Ok(Packet::new(packet))),
            Err(error) => callback.call(Err(error)),
        })
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
    /// use musli_web::web03::prelude::*;
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
    ///         pub type Tick;
    ///
    ///         impl Broadcast for Tick {
    ///             impl<'de> Event for TickEvent<'de>;
    ///         }
    ///     }
    /// }
    ///
    /// enum Msg {
    ///     Tick(Result<ws::RawPacket, ws::Error>),
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
    ///         let listen = ctx.props().ws.on_raw_broadcast::<api::Tick>(ctx.link().callback(Msg::Tick));
    ///
    ///         Self {
    ///             tick: 0,
    ///             _listen: listen,
    ///         }
    ///     }
    ///
    ///     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    ///         match msg {
    ///             Msg::Tick(Err(error)) => {
    ///                 tracing::error!("Tick error: {error}");
    ///                 false
    ///             }
    ///             Msg::Tick(Ok(packet)) => {
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
    pub fn on_raw_broadcast<T>(&self, callback: impl Callback<Result<RawPacket>>) -> Listener
    where
        T: api::Broadcast,
    {
        let Some(shared) = self.shared.upgrade() else {
            return Listener::empty_with_kind(T::ID);
        };

        let index = {
            let mut broadcasts = shared.g.broadcasts.borrow_mut();
            let slots = broadcasts.entry(T::ID).or_default();
            slots.insert(Rc::new(callback))
        };

        Listener {
            kind: Some(T::ID),
            index,
            g: Rc::downgrade(&shared.g),
        }
    }

    /// Listen for state changes to the underlying connection.
    ///
    /// This indicates when the connection is open and ready to receive requests
    /// through [`State::Open`], or if it's closed and requests will be queued
    /// through [`State::Closed`].
    ///
    /// Note that if you are connecting through a proxy the reported updates
    /// might be volatile. It is always best to send a message over the
    /// connection on the server side that once received allows the client to
    /// know that it is connected.
    ///
    /// Dropping the returned handle will cancel the listener.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::web03::prelude::*;
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
    ///         let (state, listen) = ctx.props().ws.on_state_change(move |state| {
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
    pub fn on_state_change(&self, callback: impl Callback<State>) -> (State, StateListener) {
        let Some(shared) = self.shared.upgrade() else {
            return (
                State::Closed,
                StateListener {
                    index: 0,
                    g: Weak::new(),
                },
            );
        };

        let (state, index) = {
            let index = shared
                .g
                .state_listeners
                .borrow_mut()
                .insert(Rc::new(callback));
            (shared.state.get(), index)
        };

        let listener = StateListener {
            index,
            g: Rc::downgrade(&shared.g),
        };

        (state, listener)
    }
}

impl<H> PartialEq for Handle<H>
where
    H: WebImpl,
{
    #[inline]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

struct Pending<C>
where
    C: ?Sized,
{
    kind: MessageId,
    serial: u32,
    callback: C,
}

impl<C> fmt::Debug for Pending<C> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pending")
            .field("serial", &self.serial)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}
struct ForcePrefix<'a>(&'a str, char);

impl fmt::Display for ForcePrefix<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(string, prefix) = *self;
        prefix.fmt(f)?;
        string.trim_start_matches(prefix).fmt(f)?;
        Ok(())
    }
}
