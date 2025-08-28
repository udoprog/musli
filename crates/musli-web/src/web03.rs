//! Client side implementation for [`web-sys`] `0.3.x`.
//!
//! [`web-sys`]: <https://docs.rs/web-sys/0.3>
//!
//! # Examples
//!
//! This example uses [`yew021`]:
//!
//! [`yew021`]: crate::yew021
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
//!         endpoint Hello {
//!             request<'de> = HelloRequest<'de>;
//!             response<'de> = HelloResponse<'de>;
//!         }
//!     }
//! }
//!
//! let service = ws::connect(ws::Connect::location_with_path(String::from("/ws")))
//!     .on_error_cb(|error| {
//!         tracing::error!("WebSocket error: {error}");
//!     })
//!     .build();
//!
//! service.connect();
//!
//! let request = service
//!     .handle()
//!     .request::<api::Hello>()
//!     .body(api::HelloRequest {
//!         message: "Hello!",
//!     })
//!     .on_packet_cb(move |packet| {
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

use wasm_bindgen02::JsCast;
use wasm_bindgen02::closure::Closure;
use web_sys03::Performance;
use web_sys03::Window;
use web_sys03::js_sys::{ArrayBuffer, Math, Uint8Array};
use web_sys03::{BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket, window};

use crate::web::Location;
use crate::web::SocketImplementation;
use crate::web::{Connect, Error, ServiceBuilder, Shared, WebImpl};

pub mod prelude {
    //! The public facing API for use with yew `0.2.1` and web-sys `0.3.x`.

    pub mod ws {
        //! Organization module prefixing all exported items with `ws` for
        //! convenient namespacing.

        pub use crate::web::{Connect, Error, State};
        use crate::web03::Web03Impl;

        /// Implementation alias for [`connect`].
        ///
        /// [`connect`]: crate::web03::connect
        pub fn connect(connect: Connect) -> ServiceBuilder {
            crate::web03::connect(connect)
        }

        /// Implementation alias for [`Service`].
        ///
        /// [`Service`]: crate::web::Service
        pub type Service = crate::web::Service<Web03Impl>;

        /// Implementation alias for [`Request`].
        ///
        /// [`Request`]: crate::web::Request
        pub type Request = crate::web::Request<Web03Impl>;

        /// Implementation alias for [`Handle`].
        ///
        /// [`Handle`]: crate::web::Handle
        pub type Handle = crate::web::Handle<Web03Impl>;

        /// Implementation alias for [`Listener`].
        ///
        /// [`Listener`]: crate::web::Listener
        pub type Listener = crate::web::Listener<Web03Impl>;

        /// Implementation alias for [`Packet`].
        ///
        /// [`Packet`]: crate::web::Packet
        pub type Packet<T> = crate::web::Packet<T, Web03Impl>;

        /// Implementation alias for [`RawPacket`].
        ///
        /// [`RawPacket`]: crate::web::RawPacket
        pub type RawPacket = crate::web::RawPacket<Web03Impl>;

        /// Implementation alias for [`RequestBuilder`].
        ///
        /// [`RequestBuilder`]: crate::web::RequestBuilder
        pub type RequestBuilder<'a, E, T> = crate::web::RequestBuilder<'a, E, T, Web03Impl>;

        /// Implementation alias for [`ServiceBuilder`].
        ///
        /// [`ServiceBuilder`]: crate::web::ServiceBuilder
        pub type ServiceBuilder = crate::web::ServiceBuilder<Web03Impl>;

        /// Implementation alias for [`StateListener`].
        ///
        /// [`StateListener`]: crate::web::StateListener
        pub type StateListener = crate::web::StateListener<Web03Impl>;
    }
}

/// Handles for websocket implementation.
#[doc(hidden)]
pub struct Handles {
    open: Closure<dyn Fn()>,
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

impl SocketImplementation for WebSocket {
    type Handles = Handles;

    #[inline]
    fn new(url: &str, handles: &Self::Handles) -> Result<Self, Error> {
        let this = WebSocket::new(url)?;
        this.set_binary_type(BinaryType::Arraybuffer);
        this.set_onopen(Some(handles.open.as_ref().unchecked_ref()));
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
        WebSocket::close(&self)?;
        Ok(())
    }
}

impl crate::web::sealed_web::Sealed for Web03Impl {}

impl WebImpl for Web03Impl {
    type Window = Window;
    type Performance = Performance;
    type Handles = Handles;
    type Socket = WebSocket;

    #[inline]
    fn window() -> Result<Self::Window, Error> {
        let Some(window) = window() else {
            return Err(Error::msg("No window in web-sys 0.3.x context"));
        };

        Ok(window)
    }

    #[inline]
    fn performance(window: &Self::Window) -> Result<Self::Performance, Error> {
        let Some(performance) = window.performance() else {
            return Err(Error::msg("No window.performance in web-sys 0.3.x context"));
        };

        Ok(performance)
    }

    #[inline]
    fn location(window: &Self::Window) -> Result<Location, Error> {
        let location = window.location();

        Ok(Location {
            protocol: location.protocol()?,
            host: location.hostname()?,
            port: location.port()?,
        })
    }

    #[inline]
    fn random(range: u32) -> u32 {
        ((Math::random() * range as f64).round() as u32).min(range)
    }

    #[inline]
    fn now(performance: &Self::Performance) -> f64 {
        performance.now()
    }

    #[inline]
    #[allow(private_interfaces)]
    fn handles(shared: &Weak<Shared<Self>>) -> Self::Handles {
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

        Self::Handles {
            open,
            close,
            message,
            error,
        }
    }
}

/// Construct a new [`ServiceBuilder`] associated with the given [`Connect`]
/// strategy.
pub fn connect(connect: Connect) -> ServiceBuilder<Web03Impl> {
    crate::web::connect::<Web03Impl>(connect)
}

impl Shared<Web03Impl> {
    fn do_open(&self) {
        tracing::debug!("Open event");
        self.set_open();
    }

    fn do_close(self: &Rc<Self>, e: CloseEvent) {
        tracing::debug!(code = e.code(), reason = e.reason(), "Close event");
        self.close();
    }

    fn do_message(self: &Rc<Shared<Web03Impl>>, e: MessageEvent) {
        tracing::debug!("Message event");

        if let Err(error) = self.web03_message(e) {
            self.handle_error(error);
        }
    }

    fn web03_message(self: &Rc<Shared<Web03Impl>>, e: MessageEvent) -> Result<(), Error> {
        let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() else {
            return Err(Error::msg("Expected message as ArrayBuffer"));
        };

        let array = Uint8Array::new(&array_buffer);
        let needed = array.length() as usize;

        let mut buf = self.next_buffer(needed);

        // SAFETY: We've sized the buffer appropriately above.
        unsafe {
            array.raw_copy_to_ptr(buf.data.as_mut_ptr());
            buf.data.set_len(needed);
        }

        self.message(buf)
    }

    fn do_error(self: &Rc<Self>, e: ErrorEvent) {
        tracing::debug!(message = e.message(), "Error event");
        self.close();
    }
}
