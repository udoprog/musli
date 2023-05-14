//! Utility functions for when interacting with [Müsli].
//!
//! [Müsli]: https://docs.rs/musli

mod visit_bytes_fn;
mod visit_string_fn;

pub use self::visit_bytes_fn::visit_bytes_fn;
pub use self::visit_string_fn::visit_string_fn;
