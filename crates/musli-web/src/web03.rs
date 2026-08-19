//! Client side implementation for [`web-sys`] `0.3.x`.
//!
//! [`web-sys`]: <https://docs.rs/web-sys/0.3>
//!
//! # Examples
//!
//! This example uses [`web03`]:
//!
//! [`web03`]: crate::web03
//!
//! ```no_run
//! use musli_web::web03::prelude::*;
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
//!     api::define! {
//!         pub type Hello;
//!
//!         impl Endpoint for Hello {
//!             impl<'de> Request for HelloRequest<'de>;
//!             type Response<'de> = HelloResponse<'de>;
//!         }
//!     }
//! }
//!
//! let service = ws::connect(ws::Connect::location("ws"))
//!     .on_error(|error| {
//!         tracing::error!("WebSocket error: {error}");
//!     })
//!     .build();
//!
//! service.connect();
//!
//! let request = service
//!     .handle()
//!     .request()
//!     .body(api::HelloRequest {
//!         message: "Hello!",
//!     })
//!     .on_raw_packet(move |packet: Result<ws::RawPacket, ws::Error>| {
//!         match packet {
//!             Ok(packet) => {
//!                 if let Ok(response) = packet.decode::<api::HelloResponse>() {
//!                     tracing::info!("Response: {}", response.message);
//!                 }
//!             }
//!             Err(error) => {
//!                 tracing::error!("Request error: {error}");
//!             }
//!         }
//!     })
//!     .send();
//! ```

use alloc::rc::Rc;
use alloc::rc::Weak;

use wasm_bindgen02::closure::Closure;
use wasm_bindgen02::{JsCast, JsValue};
use web_sys03::js_sys::{ArrayBuffer, Math, Uint8Array};
use web_sys03::{
    BinaryType, CloseEvent, ErrorEvent, MessageEvent, RequestCache, RequestCredentials, RequestInit,
    RequestRedirect, Response, ResponseType, WebSocket, Window, window,
};

use crate::web::{
    Connect, EmptyCallback, Error, Location, ProbeOutcome, ServiceBuilder, Shared, SocketImpl,
    WebImpl, WindowImpl,
};

pub mod prelude {
    //! The public facing API for use with yew `0.2.1` and web-sys `0.3.x`.

    pub mod ws {
        //! Organization module prefixing all exported items with `ws` for
        //! convenient namespacing.

        pub use crate::api::ChannelId;
        pub use crate::web::{
            Connect, EmptyCallback, Error, Listener, Packet, RawPacket, Request, State,
            StateListener,
        };

        use crate::web03::Web03Impl;

        /// Implementation alias for [`connect`].
        ///
        /// [`connect`]: crate::web03::connect
        pub fn connect(connect: Connect) -> ServiceBuilder<EmptyCallback> {
            crate::web03::connect(connect)
        }

        /// Implementation alias for [`Service`].
        ///
        /// [`Service`]: crate::web::Service
        pub type Service = crate::web::Service<Web03Impl>;

        /// Implementation alias for [`Handle`].
        ///
        /// [`Handle`]: crate::web::Handle
        pub type Handle = crate::web::Handle<Web03Impl>;

        /// Implementation alias for [`Channel`].
        ///
        /// [`Channel`]: crate::web::Channel
        pub type Channel = crate::web::Channel<Web03Impl>;

        /// Implementation alias for [`RequestBuilder`].
        ///
        /// [`RequestBuilder`]: crate::web::RequestBuilder
        pub type RequestBuilder<'a, B, C> = crate::web::RequestBuilder<'a, Web03Impl, B, C>;

        /// Implementation alias for [`ServiceBuilder`].
        ///
        /// [`ServiceBuilder`]: crate::web::ServiceBuilder
        pub type ServiceBuilder<C> = crate::web::ServiceBuilder<Web03Impl, C>;
    }
}

/// Handles for websocket implementation.
#[doc(hidden)]
pub struct Handles {
    close: Closure<dyn Fn(CloseEvent)>,
    message: Closure<dyn Fn(MessageEvent)>,
    error: Closure<dyn Fn(ErrorEvent)>,
}

/// WebSocket implementation for web-sys `0.3.x`.
///
/// See [`connect()`].
#[derive(Clone, Copy)]
pub enum Web03Impl {}

impl crate::web::sealed_socket::Sealed for WebSocket {}

impl SocketImpl for WebSocket {
    type Handles = Handles;

    #[inline]
    fn new(url: &str, handles: &Self::Handles) -> Result<Self, Error> {
        let this = WebSocket::new(url)?;
        this.set_binary_type(BinaryType::Arraybuffer);
        this.set_onclose(Some(handles.close.as_ref().unchecked_ref()));
        this.set_onmessage(Some(handles.message.as_ref().unchecked_ref()));
        this.set_onerror(Some(handles.error.as_ref().unchecked_ref()));
        Ok(this)
    }

    #[inline]
    fn send(&self, data: &[u8]) -> Result<(), Error> {
        self.send_with_u8_array(data)?;
        Ok(())
    }

    #[inline]
    fn close(self) -> Result<(), Error> {
        // Clear event listeners to ensure they don't fire when we dispose of
        // the socket.
        WebSocket::set_onclose(&self, None);
        WebSocket::set_onmessage(&self, None);
        WebSocket::set_onerror(&self, None);
        WebSocket::close(&self)?;
        Ok(())
    }
}

impl crate::web::sealed_window::Sealed for Window {}

impl WindowImpl for Window {
    type Timeout = Timeout;
    type OnBeforeUnload = Event;
    type Fetch = Fetch;

    #[inline]
    fn new() -> Result<Self, Error> {
        let Some(window) = window() else {
            return Err(Error::message("No window in web-sys 0.3.x context"));
        };

        Ok(window)
    }

    #[inline]
    fn location(&self) -> Result<Location, Error> {
        let location = Window::location(self);

        Ok(Location {
            protocol: location.protocol()?,
            host: location.hostname()?,
            port: location.port()?,
        })
    }

    #[inline]
    fn onbeforeunload(&self, callback: impl Fn() + 'static) -> Result<Self::OnBeforeUnload, Error> {
        let closure = Closure::new(callback);
        window()
            .unwrap()
            .set_onbeforeunload(Some(closure.as_ref().unchecked_ref()));
        Ok(Event { closure })
    }

    #[inline]
    fn set_timeout(
        &self,
        millis: u32,
        callback: impl Fn() + 'static,
    ) -> Result<Self::Timeout, Error> {
        let closure = Closure::new(callback);

        let id = self.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            millis as i32,
        )?;

        Ok(Timeout {
            window: self.clone(),
            id: Some(id),
            closure: Some(closure),
        })
    }

    fn fetch(
        &self,
        url: &str,
        callback: impl Fn(ProbeOutcome) + 'static,
    ) -> Result<Self::Fetch, Error> {
        let init = RequestInit::new();
        init.set_method("GET");
        init.set_credentials(RequestCredentials::Include);
        init.set_redirect(RequestRedirect::Manual);
        // NB: A redirect towards a login page might be cacheable, and replaying
        // it would report a stale authentication signal.
        init.set_cache(RequestCache::NoStore);

        let promise = self.fetch_with_str_and_init(url, &init);

        let callback = Rc::new(callback);

        let resolved = {
            let callback = callback.clone();

            Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
                let outcome = match value.dyn_into::<Response>() {
                    Ok(response) if response.type_() == ResponseType::Opaqueredirect => {
                        ProbeOutcome::Redirect
                    }
                    Ok(response) => ProbeOutcome::Status(response.status()),
                    Err(..) => ProbeOutcome::Failed,
                };

                callback(outcome);
            })
        };

        let rejected = Closure::<dyn FnMut(JsValue)>::new(move |_: JsValue| {
            callback(ProbeOutcome::Failed);
        });

        let _ = promise.then2(&resolved, &rejected);

        Ok(Fetch {
            _resolved: resolved,
            _rejected: rejected,
        })
    }
}

pub struct Timeout {
    window: Window,
    id: Option<i32>,
    #[allow(dead_code)]
    closure: Option<Closure<dyn FnMut()>>,
}

impl Drop for Timeout {
    /// Disposes of the timeout, dually cancelling this timeout by calling
    /// `clearTimeout` directly.
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            self.window.clear_timeout_with_handle(id);
        }
    }
}

pub struct Event {
    #[allow(dead_code)]
    closure: Closure<dyn FnMut()>,
}

/// Keeps the callbacks of an in-flight probe request alive.
pub struct Fetch {
    _resolved: Closure<dyn FnMut(JsValue)>,
    _rejected: Closure<dyn FnMut(JsValue)>,
}

impl crate::web::sealed_web::Sealed for Web03Impl {}

impl WebImpl for Web03Impl {
    type Window = Window;
    type Handles = Handles;
    type Socket = WebSocket;

    #[inline]
    fn random(range: u32) -> u32 {
        ((Math::random() * range as f64).round() as u32).min(range)
    }

    #[inline]
    #[allow(private_interfaces)]
    fn handles(shared: &Weak<Shared<Self>>) -> Self::Handles {
        let close = {
            let shared = shared.clone();

            Closure::new(move |e: CloseEvent| {
                if let Some(shared) = shared.upgrade() {
                    shared.web03_close(e);
                }
            })
        };

        let message = {
            let shared = shared.clone();

            Closure::new(move |e: MessageEvent| {
                if let Some(shared) = shared.upgrade() {
                    shared.web03_message(e);
                }
            })
        };

        let error = {
            let shared = shared.clone();

            Closure::new(move |e: ErrorEvent| {
                if let Some(shared) = shared.upgrade() {
                    shared.web03_error(e);
                }
            })
        };

        Self::Handles {
            close,
            message,
            error,
        }
    }
}

/// Construct a new [`ServiceBuilder`] associated with the given [`Connect`]
/// strategy.
#[inline]
pub fn connect(connect: Connect) -> ServiceBuilder<Web03Impl, EmptyCallback> {
    crate::web::connect(connect)
}

impl Shared<Web03Impl> {
    fn web03_close(self: &Rc<Self>, e: CloseEvent) {
        tracing::debug!(code = e.code(), reason = e.reason(), "Close event");

        if let Err(e) = self.close_and_reconnect() {
            self.on_error.call(e);
        }
    }

    fn web03_message(self: &Rc<Shared<Web03Impl>>, e: MessageEvent) {
        tracing::debug!("Message event");

        let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() else {
            self.on_error
                .call(Error::message("Expected message as ArrayBuffer"));
            return;
        };

        let array = Uint8Array::new(&array_buffer);
        let needed = array.length() as usize;

        let mut buf = self.next_buffer(needed);

        // SAFETY: We've sized the buffer appropriately above.
        unsafe {
            array.raw_copy_to_ptr(buf.data.as_mut_ptr());
            buf.data.set_len(needed);
        }

        if let Err(e) = self.message(buf) {
            self.on_error.call(e);
        }
    }

    fn web03_error(self: &Rc<Self>, e: ErrorEvent) {
        tracing::debug!(message = e.message(), "Error event");

        if let Err(e) = self.close_and_reconnect() {
            self.on_error.call(e);
        }
    }
}
