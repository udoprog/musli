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

use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::storage;
use musli::{Decode, Encode};

use gloo_timers03::callback::Timeout;
use slab::Slab;

use crate::api;

const MAX_CAPACITY: usize = 1048576;

type StateSlab = Slab<Rc<dyn Fn(State)>>;
type Broadcasts<H> = HashMap<&'static str, Slab<Rc<dyn Fn(RawPacket<H>)>>>;

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

pub(crate) mod sealed_web {
    pub trait Sealed {}
}

pub trait SocketImplementation
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
    type Window;

    #[doc(hidden)]
    type Performance;

    #[doc(hidden)]
    type Handles;

    #[doc(hidden)]
    type Socket: SocketImplementation<Handles = Self::Handles>;

    #[doc(hidden)]
    fn window() -> Result<Self::Window, Error>;

    #[doc(hidden)]
    fn performance(window: &Self::Window) -> Result<Self::Performance, Error>;

    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn handles(shared: &Weak<Shared<Self>>) -> Self::Handles;

    #[doc(hidden)]
    fn location(window: &Self::Window) -> Result<Location, Error>;

    #[doc(hidden)]
    fn random(range: u32) -> u32;

    #[doc(hidden)]
    fn now(window: &Self::Performance) -> f64;
}

/// Construct a new [`ServiceBuilder`] associated with the given [`Connect`]
/// strategy.
pub fn connect<H>(connect: Connect) -> ServiceBuilder<H>
where
    H: WebImpl,
{
    ServiceBuilder {
        connect,
        on_error: None,
        _marker: PhantomData,
    }
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

struct Requests<H>
where
    H: WebImpl,
{
    serials: HashMap<u32, usize>,
    pending: Slab<Pending<H>>,
}

impl<H> Requests<H>
where
    H: WebImpl,
{
    fn new() -> Self {
        Self {
            serials: HashMap::new(),
            pending: Slab::new(),
        }
    }

    #[inline]
    fn remove(&mut self, serial: u32) -> Option<Pending<H>> {
        let index = self.serials.remove(&serial)?;
        self.pending.try_remove(index)
    }

    #[inline]
    fn insert(&mut self, serial: u32, pending: Pending<H>) -> usize {
        let index = self.pending.insert(pending);

        if let Some(existing) = self.serials.insert(serial, index) {
            _ = self.pending.try_remove(existing);
        }

        index
    }
}

/// Queue of deferred items.
type Deferred<H> = VecDeque<Rc<dyn Fn(RawPacket<H>)>>;

/// Shared implementation details for websocket implementations.
pub(crate) struct Shared<H>
where
    H: WebImpl,
{
    connect: Connect,
    on_error: Option<Box<dyn Fn(Error)>>,
    window: H::Window,
    performance: H::Performance,
    handles: H::Handles,
    state: Cell<State>,
    opened: Cell<Option<f64>>,
    serial: Cell<u32>,
    state_listeners: RefCell<StateSlab>,
    requests: RefCell<Requests<H>>,
    broadcasts: RefCell<Broadcasts<H>>,
    deferred: RefCell<Deferred<H>>,
    socket: RefCell<Option<H::Socket>>,
    output: RefCell<Vec<u8>>,
    current_timeout: Cell<u32>,
    reconnect_timeout: RefCell<Option<Timeout>>,
    buffers: RefCell<VecDeque<Box<BufData<H>>>>,
}

impl<H> Drop for Shared<H>
where
    H: WebImpl,
{
    fn drop(&mut self) {
        if let Some(old) = self.socket.take()
            && let Err(error) = H::Socket::close(old)
            && let Some(handle) = &self.on_error
        {
            handle(error);
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
pub struct ServiceBuilder<H>
where
    H: WebImpl,
{
    connect: Connect,
    on_error: Option<Box<dyn Fn(Error)>>,
    _marker: PhantomData<H>,
}

impl<H> ServiceBuilder<H>
where
    H: WebImpl,
{
    /// Set the error handler to use for the service.
    pub fn on_error_cb(mut self, on_error: impl Fn(Error) + 'static) -> Self {
        self.on_error = Some(Box::new(on_error));
        self
    }

    /// Build a new service and handle.
    pub fn build(self) -> Service<H> {
        let window = match H::window() {
            Ok(window) => window,
            Err(error) => {
                panic!("{error}")
            }
        };

        let performance = match H::performance(&window) {
            Ok(performance) => performance,
            Err(error) => {
                panic!("{error}")
            }
        };

        let shared = Rc::<Shared<H>>::new_cyclic(|shared| Shared {
            connect: self.connect,
            on_error: self.on_error,
            window,
            performance,
            handles: H::handles(shared),
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
            buffers: RefCell::new(VecDeque::new()),
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
    /// Defer an error.
    #[inline]
    pub(crate) fn handle_error(&self, error: Error) {
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

        H::Socket::send(socket, out.as_slice())?;

        out.clear();
        out.shrink_to(MAX_CAPACITY);
        Ok(())
    }

    pub(crate) fn next_buffer(self: &Rc<Self>, needed: usize) -> Box<BufData<H>> {
        match self.buffers.borrow_mut().pop_front() {
            Some(mut buf) => {
                if buf.data.capacity() < needed {
                    buf.data.reserve(needed - buf.data.len());
                }

                buf
            }
            None => Box::new(BufData::with_capacity(Rc::downgrade(self), needed)),
        }
    }

    pub(crate) fn message(self: &Rc<Self>, buf: Box<BufData<H>>) -> Result<()> {
        // Wrap the buffer in a simple shared reference-counted container.
        let buf = BufRc::new(buf);
        let mut reader = SliceReader::new(&buf);

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

                let at = buf.len().saturating_sub(reader.remaining());

                let packet = RawPacket {
                    buf: buf.clone(),
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
                    let at = buf.len().saturating_sub(reader.remaining());

                    let packet = RawPacket {
                        buf,
                        at: Cell::new(at),
                        kind,
                    };

                    callback(Ok(packet));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn set_open(&self) {
        tracing::debug!("Set open");
        self.opened.set(Some(H::now(&self.performance)));
        self.emit_state_change(State::Open);
    }

    fn is_open_for_a_while(&self) -> bool {
        let Some(at) = self.opened.get() else {
            return false;
        };

        let now = H::now(&self.performance);

        (now - at) >= 250.0
    }

    pub(crate) fn close(self: &Rc<Self>) {
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

        if let Some(old) = self.socket.take()
            && let Err(error) = H::Socket::close(old)
        {
            self.handle_error(error);
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
                let Location {
                    protocol,
                    host,
                    port,
                } = H::location(&self.window)?;

                let protocol = match protocol.as_str() {
                    "https:" => "wss:",
                    "http:" => "ws:",
                    other => {
                        return Err(Error::msg(format_args!(
                            "Same host connection is not supported for protocol `{other}`"
                        )));
                    }
                };

                let path = match path {
                    Some(path) => ForcePrefix(path),
                    None => ForcePrefix(""),
                };

                format!("{protocol}//{host}:{port}{path}")
            }
            ConnectKind::Url { url } => url.clone(),
        };

        let ws = H::Socket::new(&url, &self.handles)?;

        let old = self.socket.borrow_mut().replace(ws);

        if let Some(old) = old {
            H::Socket::close(old)?;
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
pub struct RequestBuilder<'a, E, T, H>
where
    H: WebImpl,
{
    shared: &'a Weak<Shared<H>>,
    callback: Option<Rc<dyn Fn(Result<RawPacket<H>>)>>,
    body: T,
    _marker: PhantomData<E>,
}

impl<'a, E, H> RequestBuilder<'a, E, (), H>
where
    E: api::Endpoint,
    H: WebImpl,
{
    /// Set the body of the request.
    #[inline]
    pub fn body<T>(self, body: T) -> RequestBuilder<'a, E, T, H>
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

impl<'a, E, T, H> RequestBuilder<'a, E, T, H>
where
    E: api::Endpoint,
    T: Encode<Binary>,
    H: WebImpl,
{
    /// Build and return the request.
    pub fn send(self) -> Request<H> {
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

impl<'a, E, T, H> RequestBuilder<'a, E, T, H>
where
    H: WebImpl,
{
    /// Handle the response using the specified callback.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate yew021 as yew;
    /// use yew::prelude::*;
    /// use musli_web::yew021::prelude::*;
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
    ///             .on_packet_cb(move |packet| link.send_message(Msg::OnHello(packet)))
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
    pub fn on_packet_cb(mut self, f: impl Fn(Result<RawPacket<H>, Error>) + 'static) -> Self {
        self.callback = Some(Rc::new(f));
        self
    }
}

/// The handle for a pending request. Dropping this handle cancels the request.
pub struct Request<H>
where
    H: WebImpl,
{
    inner: Option<(Weak<Shared<H>>, u32)>,
}

impl<H> Request<H>
where
    H: WebImpl,
{
    /// An empty request handler.
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }
}

impl<H> Default for Request<H>
where
    H: WebImpl,
{
    #[inline]
    fn default() -> Self {
        Self { inner: None }
    }
}

impl<H> Drop for Request<H>
where
    H: WebImpl,
{
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
pub struct Listener<H>
where
    H: WebImpl,
{
    kind: &'static str,
    index: usize,
    shared: Weak<Shared<H>>,
}

impl<H> Listener<H>
where
    H: WebImpl,
{
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

impl<H> Drop for Listener<H>
where
    H: WebImpl,
{
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
pub struct StateListener<H>
where
    H: WebImpl,
{
    index: usize,
    shared: Weak<Shared<H>>,
}

impl<H> Drop for StateListener<H>
where
    H: WebImpl,
{
    #[inline]
    fn drop(&mut self) {
        let mut removed = None;

        if let Some(shared) = self.shared.upgrade() {
            removed = shared.state_listeners.borrow_mut().try_remove(self.index);
        }

        drop(removed);
    }
}

pub(crate) struct BufData<H>
where
    H: WebImpl,
{
    /// Reference to shared state where the buffer will be recycled to.
    shared: Weak<Shared<H>>,
    /// Buffer being used.
    pub(crate) data: Vec<u8>,
    /// Number of strong references to this buffer.
    strong: Cell<usize>,
}

impl<H> BufData<H>
where
    H: WebImpl,
{
    fn with_capacity(shared: Weak<Shared<H>>, capacity: usize) -> Self {
        Self {
            shared,
            data: Vec::with_capacity(capacity),
            strong: Cell::new(0),
        }
    }

    unsafe fn dec(ptr: NonNull<BufData<H>>) {
        unsafe {
            let count = ptr.as_ref().strong.get().wrapping_sub(1);
            ptr.as_ref().strong.set(count);

            if count > 0 {
                return;
            }

            let mut buf = Box::from_raw(ptr.as_ptr());

            // Try to recycle the buffer if shared is available, else let it be
            // dropped and free here.
            let Some(shared) = buf.as_ref().shared.upgrade() else {
                return;
            };

            let mut buffers = shared.buffers.borrow_mut();

            // Set the length of the recycled buffer.
            buf.data.set_len(buf.data.len().min(MAX_CAPACITY));

            // We size our buffers to some max capacity to avod overuse in case
            // we infrequently need to handle some massive message. If we don't
            // shrink the allocation, then memory use can run away over time.
            buf.data.shrink_to(MAX_CAPACITY);

            buffers.push_back(buf);
        }
    }

    unsafe fn inc(ptr: NonNull<BufData<H>>) {
        unsafe {
            let count = ptr.as_ref().strong.get().wrapping_add(1);

            if count == 0 {
                std::process::abort();
            }

            ptr.as_ref().strong.set(count);
        }
    }
}

struct BufRc<H>
where
    H: WebImpl,
{
    data: NonNull<BufData<H>>,
}

impl<H> BufRc<H>
where
    H: WebImpl,
{
    fn new(data: Box<BufData<H>>) -> Self {
        let data = NonNull::from(Box::leak(data));

        unsafe {
            BufData::inc(data);
        }

        Self { data }
    }
}

impl<H> Deref for BufRc<H>
where
    H: WebImpl,
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.data.as_ptr()).data }
    }
}

impl<H> Clone for BufRc<H>
where
    H: WebImpl,
{
    fn clone(&self) -> Self {
        unsafe {
            BufData::inc(self.data);
        }

        Self { data: self.data }
    }
}

impl<H> Drop for BufRc<H>
where
    H: WebImpl,
{
    fn drop(&mut self) {
        unsafe {
            BufData::dec(self.data);
        }
    }
}

/// A raw packet of data.
#[derive(Clone)]
pub struct RawPacket<H>
where
    H: WebImpl,
{
    buf: BufRc<H>,
    at: Cell<usize>,
    kind: &'static str,
}

impl<H> RawPacket<H>
where
    H: WebImpl,
{
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
pub struct Packet<T, H>
where
    H: WebImpl,
{
    raw: RawPacket<H>,
    _marker: PhantomData<T>,
}

impl<T, H> Packet<T, H>
where
    H: WebImpl,
{
    /// Construct a new typed package from a raw one.
    ///
    /// Note that this does not guarantee that the typed package is correct, but
    /// the `T` parameter becomes associated with it allowing it to be used
    /// automatically with methods such as [`Packet::decode`].
    #[inline]
    pub fn new(raw: RawPacket<H>) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    /// Convert a packet into a raw packet.
    ///
    /// To determine which endpoint or broadcast it belongs to the
    /// [`RawPacket::kind`] method can be used.
    pub fn into_raw(self) -> RawPacket<H> {
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

impl<T, H> Packet<T, H>
where
    T: api::Endpoint,
    H: WebImpl,
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

impl<T, H> Packet<T, H>
where
    T: api::Listener,
    H: WebImpl,
{
    /// Decode a typed broadcast.
    pub fn decode_broadcast(&self) -> Result<T::Broadcast<'_>> {
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
    /// use musli_web::yew021::prelude::*;
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
    pub fn request<E>(&self) -> RequestBuilder<'_, E, (), H>
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
    /// use musli_web::yew021::prelude::*;
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
    pub fn listen_cb<T>(&self, f: impl Fn(RawPacket<H>) + 'static) -> Listener<H>
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
    /// use musli_web::yew021::prelude::*;
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
    pub fn on_state_change_cb(&self, f: impl Fn(State) + 'static) -> (State, StateListener<H>) {
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

impl<H> PartialEq for Handle<H>
where
    H: WebImpl,
{
    #[inline]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

struct Pending<H>
where
    H: WebImpl,
{
    serial: u32,
    callback: Option<Rc<dyn Fn(Result<RawPacket<H>>)>>,
    kind: &'static str,
}

impl<H> fmt::Debug for Pending<H>
where
    H: WebImpl,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pending")
            .field("serial", &self.serial)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}
