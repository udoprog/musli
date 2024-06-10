use core::fmt;

#[cfg(feature = "alloc")]
use rust_alloc::boxed::Box;
#[cfg(feature = "alloc")]
use rust_alloc::string::ToString;

use crate::context::ContextError;

/// Error raised during json encoding.
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
    #[cfg(not(feature = "alloc"))]
    Empty,
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "alloc")]
            ErrorImpl::Message(message) => message.fmt(f),
            #[cfg(not(feature = "alloc"))]
            ErrorImpl::Empty => write!(f, "Message error (see diagnostics)"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl ContextError for Error {
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
            err: ErrorImpl::Empty,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ErrorMessage {
    ParseFloat,
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorMessage::ParseFloat => {
                write!(f, "Illegal float encountered")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ErrorMessage {}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum IntegerError {
    IntegerOverflow,
    Decimal,
    InvalidNumeric,
}

impl fmt::Display for IntegerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegerError::IntegerOverflow => write!(f, "Arithmetic overflow"),
            IntegerError::Decimal => write!(f, "Decimal number"),
            IntegerError::InvalidNumeric => write!(f, "Invalid numeric"),
        }
    }
}
