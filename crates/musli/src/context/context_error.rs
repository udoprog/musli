//! Trait governing what error types associated with the encoding framework must
//! support.
//!
//! The most important component in here is `Error::custom` which allows custom
//! encoding implementations to raise custom errors based on types that
//! implement [Display][core::fmt::Display].

use core::error::Error;
use core::fmt;

use crate::Allocator;

use super::Location;

#[cfg(feature = "alloc")]
use rust_alloc::string::{String, ToString};

/// Trait governing errors raised during encodeing or decoding.
#[diagnostic::on_unimplemented(
    message = "`ContextError` must be implemented for `{Self}`, or any error type captured by custom contexts",
    note = "use `musli::context::ErrorMarker` to ignore errors",
    note = "use `std::io::Error` and `std::string::String`, if the `std` or `alloc` features are enabled for `musli`"
)]
pub trait ContextError<A> {
    /// Construct a custom error.
    fn custom<T>(alloc: A, error: T) -> Self
    where
        T: 'static + Send + Sync + Error;

    /// Collect an error from something that can be displayed.
    ///
    /// This is made available to format custom error messages in `no_std`
    /// environments. The error message is to be collected by formatting `T`.
    fn message<T>(alloc: A, message: T) -> Self
    where
        T: fmt::Display;

    /// Collect an error from something that can be displayed, which was raised
    /// at the given [`Location`].
    ///
    /// A context which traces, such as one built with
    /// [`DefaultContext::with_trace`], calls this instead of
    /// [`ContextError::message`]. The default implementation passes the
    /// message decorated through [`Location::with_message`] to
    /// [`ContextError::message`], so that the [`Display`] of the error
    /// includes the path and position, such as
    /// `.field[2].name: Expected string (at byte 14)`.
    ///
    /// [`DefaultContext::with_trace`]: super::DefaultContext::with_trace
    /// [`Display`]: fmt::Display
    #[inline]
    fn traced_message<T>(alloc: A, message: T, location: &Location<'_, A>) -> Self
    where
        Self: Sized,
        A: Allocator,
        T: fmt::Display,
    {
        Self::message(alloc, location.with_message(message))
    }

    /// Construct a custom error, which was raised at the given [`Location`].
    ///
    /// A context which traces, such as one built with
    /// [`DefaultContext::with_trace`], calls this instead of
    /// [`ContextError::custom`]. The default implementation formats the error
    /// decorated through [`Location::with_message`] and passes it to
    /// [`ContextError::message`], which keeps the location but not the
    /// original error. Implementations which can keep the error as a
    /// [`source`] should override this.
    ///
    /// [`DefaultContext::with_trace`]: super::DefaultContext::with_trace
    /// [`source`]: Error::source
    #[inline]
    fn traced_custom<T>(alloc: A, error: T, location: &Location<'_, A>) -> Self
    where
        Self: Sized,
        A: Allocator,
        T: 'static + Send + Sync + Error,
    {
        Self::message(alloc, location.with_message(error))
    }
}

#[cfg(feature = "std")]
impl<A> ContextError<A> for std::io::Error {
    #[inline]
    fn custom<T>(_: A, message: T) -> Self
    where
        T: 'static + Send + Sync + Error,
    {
        std::io::Error::other(message)
    }

    #[inline]
    fn message<T>(_: A, message: T) -> Self
    where
        T: fmt::Display,
    {
        std::io::Error::other(std::format!("{message}"))
    }

    #[inline]
    fn traced_custom<T>(_: A, error: T, location: &Location<'_, A>) -> Self
    where
        A: Allocator,
        T: 'static + Send + Sync + Error,
    {
        std::io::Error::other(TracedIoError {
            message: std::format!("{}", location.with_message(&error)),
            source: error,
        })
    }
}

/// A custom error which has been decorated with the location it was raised at,
/// which keeps the original error as its source.
#[cfg(feature = "std")]
struct TracedIoError<T> {
    message: String,
    source: T,
}

#[cfg(feature = "std")]
impl<T> fmt::Display for TracedIoError<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

#[cfg(feature = "std")]
impl<T> fmt::Debug for TracedIoError<T>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TracedIoError")
            .field("message", &self.message)
            .field("source", &self.source)
            .finish()
    }
}

#[cfg(feature = "std")]
impl<T> Error for TracedIoError<T>
where
    T: 'static + Error,
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[cfg(feature = "alloc")]
impl<A> ContextError<A> for String {
    #[inline]
    fn custom<T>(_: A, message: T) -> Self
    where
        T: fmt::Display,
    {
        message.to_string()
    }

    #[inline]
    fn message<T>(_: A, message: T) -> Self
    where
        T: fmt::Display,
    {
        message.to_string()
    }
}
