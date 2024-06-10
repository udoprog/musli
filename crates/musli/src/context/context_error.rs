//! Trait governing what error types associated with the encoding framework must
//! support.
//!
//! The most important component in here is `Error::custom` which allows custom
//! encoding implementations to raise custom errors based on types that
//! implement [Display][core::fmt::Display].

use core::fmt;

use crate::no_std;

#[cfg(feature = "alloc")]
use rust_alloc::string::{String, ToString};

/// Trait governing errors raised during encodeing or decoding.
#[diagnostic::on_unimplemented(
    message = "`ContextError` must be implemented for `{Self}`, or any error type captured by custom contexts",
    note = "use `musli::context::ErrorMarker` to ignore errors",
    note = "use `std::io::Error` and `std::string::String`, if the `std` or `alloc` features are enabled for `musli`"
)]
pub trait ContextError: Sized + 'static + Send + Sync + fmt::Display + fmt::Debug {
    /// Construct a custom error.
    fn custom<T>(error: T) -> Self
    where
        T: 'static + Send + Sync + no_std::Error;

    /// Collect an error from something that can be displayed.
    ///
    /// This is made available to format custom error messages in `no_std`
    /// environments. The error message is to be collected by formatting `T`.
    fn message<T>(message: T) -> Self
    where
        T: fmt::Display;
}

#[cfg(all(feature = "std", feature = "alloc"))]
impl ContextError for std::io::Error {
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        std::io::Error::new(std::io::ErrorKind::Other, message.to_string())
    }

    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        std::io::Error::new(std::io::ErrorKind::Other, message.to_string())
    }
}

#[cfg(feature = "alloc")]
impl ContextError for String {
    #[inline]
    fn custom<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        message.to_string()
    }

    #[inline]
    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        message.to_string()
    }
}
