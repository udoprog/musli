use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

use musli::de::{NumberHint, TypeHint};
use musli_common::reader::SliceUnderflow;

/// An error raised when encoding or decoding [`Value`][crate::Value].
#[derive(Debug)]
pub struct Error {
    err: ErrorImpl,
}

impl Error {
    #[inline(always)]
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            err: ErrorImpl::ValueError(kind),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)
    }
}

/// Errors specifically produced by value decoding.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ErrorKind {
    #[cfg(feature = "alloc")]
    ArrayOutOfBounds,
    ExpectedPackValue,
    ExpectedUnit(TypeHint),
    ExpectedBool(TypeHint),
    ExpectedChar(TypeHint),
    ExpectedNumber(NumberHint, TypeHint),
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

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "alloc")]
            ErrorKind::ArrayOutOfBounds => {
                write!(f, "tried to decode array that is out-of-bounds")
            }
            ErrorKind::ExpectedPackValue => write!(f, "Expected pack value"),
            ErrorKind::ExpectedUnit(hint) => write!(f, "Expected unit, but found {hint}"),
            ErrorKind::ExpectedBool(hint) => write!(f, "Expected boolean, but found {hint}"),
            ErrorKind::ExpectedChar(hint) => write!(f, "Expected character, but found {hint}"),
            ErrorKind::ExpectedNumber(number, hint) => {
                write!(f, "Expected {number}, but found {hint}")
            }
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedBytes(hint) => write!(f, "Expected bytes, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedString(hint) => write!(f, "Expected string, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedOption(hint) => write!(f, "Expected option, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedSequence(hint) => write!(f, "Expected sequence, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedPack(hint) => write!(f, "Expected pack, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedMap(hint) => write!(f, "Expected map, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedVariant(hint) => write!(f, "Expected struct, but found {hint}"),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ErrorImpl {
    ValueError(ErrorKind),
    SliceUnderflow(SliceUnderflow),
    #[cfg(feature = "alloc")]
    Message(Box<str>),
    #[cfg(not(feature = "alloc"))]
    Message,
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorImpl::ValueError(error) => error.fmt(f),
            ErrorImpl::SliceUnderflow(error) => error.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorImpl::Message(message) => message.fmt(f),
            #[cfg(not(feature = "alloc"))]
            ErrorImpl::Message => write!(f, "message error (see diagnostics)"),
        }
    }
}

impl From<SliceUnderflow> for Error {
    #[inline]
    fn from(error: SliceUnderflow) -> Self {
        Self {
            err: ErrorImpl::SliceUnderflow(error),
        }
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(error: ErrorKind) -> Self {
        Self::new(error)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

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
