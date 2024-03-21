use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

use musli::de::{NumberHint, TypeHint};

/// An error raised when encoding or decoding [`Value`][crate::Value].
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

#[allow(missing_docs)]
#[derive(Debug)]
#[non_exhaustive]
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

impl crate::context::Error for Error {
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

/// Errors specifically produced by value decoding.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub(crate) enum ErrorMessage {
    #[cfg(feature = "alloc")]
    ArrayOutOfBounds,
    ExpectedPackValue,
    ExpectedUnit(TypeHint),
    ExpectedBool(TypeHint),
    ExpectedChar(TypeHint),
    ExpectedNumber(NumberHint, TypeHint),
    ExpectedFieldName,
    ExpectedFieldValue,
    ExpectedMapValue,
    #[cfg(feature = "alloc")]
    ExpectedBytes(TypeHint),
    #[cfg(feature = "alloc")]
    ExpectedString(TypeHint),
    #[cfg(feature = "alloc")]
    ExpectedOption(TypeHint),
    #[cfg(feature = "alloc")]
    ExpectedSequence(TypeHint),
    #[cfg(feature = "alloc")]
    ExpectedPack(TypeHint),
    #[cfg(feature = "alloc")]
    ExpectedMap(TypeHint),
    #[cfg(feature = "alloc")]
    ExpectedVariant(TypeHint),
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "alloc")]
            ErrorMessage::ArrayOutOfBounds => {
                write!(f, "tried to decode array that is out-of-bounds")
            }
            ErrorMessage::ExpectedPackValue => write!(f, "Expected pack value"),
            ErrorMessage::ExpectedUnit(hint) => write!(f, "Expected unit, but found {hint}"),
            ErrorMessage::ExpectedBool(hint) => write!(f, "Expected boolean, but found {hint}"),
            ErrorMessage::ExpectedChar(hint) => write!(f, "Expected character, but found {hint}"),
            ErrorMessage::ExpectedNumber(number, hint) => {
                write!(f, "Expected {number}, but found {hint}")
            }
            ErrorMessage::ExpectedFieldName => write!(f, "Expected field name"),
            ErrorMessage::ExpectedFieldValue => write!(f, "Expected field value"),
            ErrorMessage::ExpectedMapValue => write!(f, "Expected map value"),
            #[cfg(feature = "alloc")]
            ErrorMessage::ExpectedBytes(hint) => write!(f, "Expected bytes, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorMessage::ExpectedString(hint) => write!(f, "Expected string, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorMessage::ExpectedOption(hint) => write!(f, "Expected option, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorMessage::ExpectedSequence(hint) => {
                write!(f, "Expected sequence, but found {hint}")
            }
            #[cfg(feature = "alloc")]
            ErrorMessage::ExpectedPack(hint) => write!(f, "Expected pack, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorMessage::ExpectedMap(hint) => write!(f, "Expected map, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorMessage::ExpectedVariant(hint) => write!(f, "Expected struct, but found {hint}"),
        }
    }
}
