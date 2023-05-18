//! Generic error types that can be used for most [`Reader`] / [`Writer`]
//! implementations.
//!
//! [`Reader`]: crate::reader::Reader
//! [`Writer`]: crate::writer::Writer

use core::fmt;

use musli::error::Error;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

struct Repr(#[cfg(feature = "alloc")] Box<str>);

impl Repr {
    #[cfg(feature = "alloc")]
    fn collect<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(message.to_string().into())
    }

    #[cfg(not(feature = "alloc"))]
    fn collect<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        Self()
    }
}

#[cfg(feature = "alloc")]
impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(not(feature = "alloc"))]
impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "buffer overflow / underflow".fmt(f)
    }
}

#[cfg(feature = "alloc")]
impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "buffer overflow / underflow".fmt(f)
    }
}

/// An error raised while working with a buffer.
#[derive(Debug)]
pub struct BufferError(Repr);

impl fmt::Display for BufferError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BufferError {
    #[inline]
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self(Repr::collect(message))
    }

    #[inline]
    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(Repr::collect(message))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BufferError {}
