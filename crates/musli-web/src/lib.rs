//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-web.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-web)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--web-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-web)
//!
//! This crate provides a set of utilities for working with various web-based
//! APIs and [Müsli].
//!
//! It includes support for:
//! - [`axum`] [`Json`] integration, allowing you to use Müsli for serialization
//!   and deserialization in your Axum applications.
//! - [`axum`] [`ws::Server`] integration, allowing you to build the server side
//!   of the websocket protocol this crate implements.
//! - [`yew`] integration, allowing you to use Müsli for communicating with
//!   websocket clients using a well-defined API.
//!
//! Note that the organization of the modules include the version of the corresponding
//! crate. Unstable versions are prefixed with `0`, such as [`yew021`].
//!
//! See the following modules for how to use:
//! * [`axum08`] for [`axum`] `0.8.x` integration.
//! * [`yew021`] for [`yew`] `0.21.x` integration.
//! * [`web03`] for [`web-sys`] `0.3.x` integration.
//!
//! <br>
//!
//! ## Examples
//!
//! * [`api`] is the example crate which defines API types shared between server
//!   and client.
//! * [`server`] is the axum-based server implementation.
//! * [`client`] is the yew client communicating with the server.
//!
//! You can run the client like this:
//!
//! ```sh
//! cd examples/client && trunk serve
//! ```
//!
//! You can run the server like this:
//!
//! ```sh
//! cd examples/server && cargo run
//! ```
//!
//! [`api`]: <https://github.com/udoprog/musli/tree/main/crates/musli-web/examples/api/>
//! [`axum`]: <https://docs.rs/axum>
//! [`client`]: <https://github.com/udoprog/musli/tree/main/crates/musli-web/examples/client/>
//! [`Json`]: <https://docs.rs/musli-web/latest/musli-web/Json.struct.html>
//! [`server`]: <https://github.com/udoprog/musli/tree/main/crates/musli-web/examples/server/>
//! [`web-sys`]: <https://docs.rs/web-sys>
//! [`ws::Server`]: <https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html>
//! [`yew`]: <https://yew.rs>
//! [Müsli]: <https://github.com/udoprog/musli>

#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![allow(clippy::type_complexity)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "axum08")]
mod buf;
#[cfg(feature = "axum08")]
use self::buf::Buf;

#[cfg(all(feature = "json", feature = "alloc"))]
mod json;
#[cfg(all(feature = "json", feature = "alloc"))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "json", feature = "alloc"))))]
pub use self::json::Json;

#[cfg(feature = "api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "api")))]
pub mod api;

#[cfg(feature = "axum08")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "axum08")))]
pub mod axum08;

#[cfg(feature = "web03")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "web03")))]
pub mod web;

#[cfg(feature = "web03")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "web03")))]
pub mod web03;

#[cfg(feature = "yew021")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "yew021")))]
pub mod yew021;

#[cfg(feature = "ws")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "ws")))]
pub mod ws;

#[doc(hidden)]
pub mod __macros {
    pub use core::fmt;
}
