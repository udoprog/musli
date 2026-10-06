//! Client side implementation for [`tokio-tungstenite`] `0.30.x`.
//!
//! This allows non-browser clients to talk to the same websocket API which is
//! served by [`ws::Server`], such as through the [`axum08`] integration.
//!
//! [`axum08`]: <https://docs.rs/musli-web/latest/musli_web/axum08/>
//! [`tokio-tungstenite`]: <https://docs.rs/tokio-tungstenite/0.30>
//! [`ws::Server`]: <https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html>
//!
//! # Examples
//!
//! ```no_run
//! use musli_web::tungstenite030::prelude::*;
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
//! # async fn example() -> Result<(), Box<dyn core::error::Error>> {
//! let mut service = ws::connect("ws://localhost:3000/ws")
//!     .on_error(|error| {
//!         tracing::error!("WebSocket error: {error}");
//!     })
//!     .build();
//!
//! let handle = service.handle().clone();
//!
//! tokio::spawn(async move {
//!     if let Err(error) = service.run().await {
//!         tracing::error!("WebSocket service error: {error}");
//!     }
//! });
//!
//! handle.wait_until_open().await?;
//!
//! let packet = handle
//!     .request()
//!     .body(api::HelloRequest { message: "Hello!" })
//!     .send()
//!     .await?;
//!
//! let response = packet.decode()?;
//! println!("Response: {}", response.message);
//! # Ok(())
//! # }
//! ```
//!
//! # Connecting over any stream
//!
//! [`connect_with()`] establishes the websocket over a stream the caller
//! opens, such as a [`UnixStream`], instead of connecting to the url. Combined
//! with [`Service::try_connect`] a caller can fail fast if there is nothing to
//! connect to, rather than retrying in the background:
//!
//! ```no_run
//! use musli_web::tungstenite030::prelude::*;
//!
//! # async fn example() -> Result<(), Box<dyn core::error::Error>> {
//! let mut service = ws::connect_with("ws://localhost/ws", || {
//!     tokio::net::UnixStream::connect("/run/user/1000/app.sock")
//! })
//! .reconnect(false)
//! .build();
//!
//! service.try_connect().await?;
//!
//! tokio::spawn(async move {
//!     if let Err(error) = service.run().await {
//!         tracing::error!("WebSocket service error: {error}");
//!     }
//! });
//! # Ok(())
//! # }
//! ```
//!
//! [`UnixStream`]: <https://docs.rs/tokio/1/tokio/net/struct.UnixStream.html>
//! [`Service::try_connect`]: crate::client::Service::try_connect

tungstenite_impl!(tokio_tungstenite030, Tungstenite030Impl, "0.30.x");
