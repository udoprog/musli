use core::fmt;

#[cfg(feature = "alloc")]
use rust_alloc::boxed::Box;
#[cfg(feature = "alloc")]
use rust_alloc::string::ToString;

use crate::context::ContextError;

/// Error raised during descriptive encoding.
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
    #[cfg(feature = "alloc")]
    Message(Box<str>),
    #[cfg(feature = "alloc")]
    Custom(Box<dyn 'static + Send + Sync + core::error::Error>),
    #[cfg(not(feature = "alloc"))]
    Empty,
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "alloc")]
            ErrorImpl::Message(message) => message.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorImpl::Custom(message) => message.fmt(f),
            #[cfg(not(feature = "alloc"))]
            ErrorImpl::Empty => write!(f, "Message error (see diagnostics)"),
        }
    }
}

impl core::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.err {
            #[cfg(feature = "alloc")]
            ErrorImpl::Custom(err) => Some(&**err),
            _ => None,
        }
    }
}

impl ContextError for Error {
    #[inline]
    #[allow(unused_variables)]
    fn custom<T>(error: T) -> Self
    where
        T: 'static + Send + Sync + core::error::Error,
    {
        Self {
            #[cfg(feature = "alloc")]
            err: ErrorImpl::Custom(Box::new(error)),
            #[cfg(not(feature = "alloc"))]
            err: ErrorImpl::Empty,
        }
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
            err: ErrorImpl::Empty,
        }
    }
}
