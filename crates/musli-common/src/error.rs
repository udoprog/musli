//! Generic error types that can be used for most [Reader] / [Writer]
//! implementations.
//!
//! [Reader]: crate::reader::Reader
//! [Writer]: crate::reader::Writer

use core::fmt;

use musli::error::Error;

struct Repr(#[cfg(feature = "std")] Box<str>);

impl Repr {
    #[cfg(feature = "std")]
    fn collect<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(message.to_string().into())
    }

    #[cfg(not(feature = "std"))]
    fn collect<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        Self
    }
}

#[cfg(feature = "std")]
impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "buffer overflow / underflow".fmt(f)
    }
}

#[cfg(feature = "std")]
impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "buffer overflow / underflow".fmt(f)
    }
}

/// An error raised while decoding a slice.
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
