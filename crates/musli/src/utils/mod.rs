//! Utility functions for when interacting with [Müsli].
//!
//! [Müsli]: https://docs.rs/musli

mod bytes_visitor_fn;
mod string_visitor_fn;

pub use self::bytes_visitor_fn::bytes_visitor_fn;
pub use self::string_visitor_fn::string_visitor_fn;
