use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

use musli::de::{NumberHint, TypeHint};
use musli_common::reader::SliceUnderflow;

/// An error raised when encoding or decoding [Value][crate::Value].
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[inline(always)]
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ErrorKind {
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
    SliceUnderflow(SliceUnderflow),
    #[cfg(feature = "alloc")]
    Message(Box<str>),
    #[cfg(not(feature = "alloc"))]
    Message,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "alloc")]
            ErrorKind::ArrayOutOfBounds => {
                write!(f, "tried to decode array that is out-of-bounds")
            }
            ErrorKind::ExpectedPackValue => write!(f, "expected pack value"),
            ErrorKind::ExpectedUnit(hint) => write!(f, "expected unit, but found {hint}"),
            ErrorKind::ExpectedBool(hint) => write!(f, "expected boolean, but found {hint}"),
            ErrorKind::ExpectedChar(hint) => write!(f, "expected character, but found {hint}"),
            ErrorKind::ExpectedNumber(number, hint) => {
                write!(f, "expected {number}, but found {hint}")
            }
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedBytes(hint) => write!(f, "expected bytes, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedString(hint) => write!(f, "expected string, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedOption(hint) => write!(f, "expected option, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedSequence(hint) => write!(f, "expected sequence, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedPack(hint) => write!(f, "expected pack, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedMap(hint) => write!(f, "expected map, but found {hint}"),
            #[cfg(feature = "alloc")]
            ErrorKind::ExpectedVariant(hint) => write!(f, "expected struct, but found {hint}"),
            ErrorKind::SliceUnderflow(error) => error.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorKind::Message(message) => message.fmt(f),
            #[cfg(not(feature = "alloc"))]
            ErrorKind::Message => write!(f, "message error (see diagnostics)"),
        }
    }
}

impl From<SliceUnderflow> for Error {
    #[inline]
    fn from(error: SliceUnderflow) -> Self {
        Self::new(ErrorKind::SliceUnderflow(error))
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
            kind: ErrorKind::Message(message.to_string().into()),
            #[cfg(not(feature = "alloc"))]
            kind: ErrorKind::Message,
        }
    }
}
