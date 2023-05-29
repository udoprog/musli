use core::convert::Infallible;
use core::fmt;

use musli_common::fixed_bytes::FixedBytesOverflow;
use musli_common::reader::SliceUnderflow;
use musli_common::writer::SliceOverflow;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

/// Error raised during storage encoding.
#[derive(Debug)]
pub struct Error {
    err: ErrorImpl,
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)
    }
}

#[derive(Debug)]
enum ErrorImpl {
    SliceUnderflow(SliceUnderflow),
    SliceOverflow(SliceOverflow),
    FixedBytesOverflow(FixedBytesOverflow),
    #[cfg(feature = "std")]
    Io(std::io::Error),
    #[cfg(feature = "alloc")]
    Message(Box<str>),
    #[cfg(not(feature = "alloc"))]
    Message,
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorImpl::SliceUnderflow(error) => error.fmt(f),
            ErrorImpl::SliceOverflow(error) => error.fmt(f),
            ErrorImpl::FixedBytesOverflow(error) => error.fmt(f),
            #[cfg(feature = "std")]
            ErrorImpl::Io(error) => error.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorImpl::Message(message) => message.fmt(f),
            #[cfg(not(feature = "alloc"))]
            ErrorImpl::Message => write!(f, "message error (see diagnostics)"),
        }
    }
}

impl From<SliceUnderflow> for Error {
    #[inline(always)]
    fn from(error: SliceUnderflow) -> Self {
        Self {
            err: ErrorImpl::SliceUnderflow(error),
        }
    }
}

impl From<SliceOverflow> for Error {
    #[inline(always)]
    fn from(error: SliceOverflow) -> Self {
        Self {
            err: ErrorImpl::SliceOverflow(error),
        }
    }
}

impl From<FixedBytesOverflow> for Error {
    #[inline(always)]
    fn from(error: FixedBytesOverflow) -> Self {
        Self {
            err: ErrorImpl::FixedBytesOverflow(error),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    #[inline(always)]
    fn from(error: std::io::Error) -> Self {
        Self {
            err: ErrorImpl::Io(error),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

impl musli::error::Error for Error {
    #[inline]
    fn custom<T>(error: T) -> Self
    where
        T: fmt::Display,
    {
        Self::message(error)
    }

    #[inline]
    #[allow(unused_variables)]
    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            #[cfg(feature = "alloc")]
            err: ErrorImpl::Message(message.to_string().into()),
            #[cfg(not(feature = "alloc"))]
            err: ErrorImpl::Message,
        }
    }
}
