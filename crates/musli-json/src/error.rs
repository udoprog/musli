use core::convert::Infallible;
use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

use musli_common::fixed_bytes::FixedBytesOverflow;
use musli_common::reader::SliceUnderflow;
use musli_common::writer::SliceOverflow;

use crate::reader::Token;

/// Error raised during json encoding.
#[derive(Debug)]
pub struct Error {
    err: ErrorImpl,
}

impl Error {
    #[inline(always)]
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            err: ErrorImpl::JsonError(kind),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    IntegerOverflow,
    Decimal,
    InvalidNumeric,
    ControlCharacterInString,
    LoneLeadingSurrogatePair,
    ExpectedColon(Token),
    ExpectedOpenBrace(Token),
    ExpectedCloseBrace(Token),
    ExpectedOpenBracket(Token),
    ExpectedCloseBracket(Token),
    InvalidEscape,
    BufferUnderflow,
    BufferOverflow,
    UnexpectedHexEscapeEnd,
    InvalidUnicode,
    CharEmptyString,
    ExpectedNull,
    ExpectedTrue,
    ExpectedFalse,
    ExpectedBool(Token),
    ExpectedString(Token),
    ExpectedValue(Token),
    ParseFloat(lexical::Error),
    Eof,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::IntegerOverflow => write!(f, "Arithmetic overflow"),
            ErrorKind::Decimal => write!(f, "Decimal number"),
            ErrorKind::InvalidNumeric => write!(f, "Invalid numeric"),
            ErrorKind::ControlCharacterInString => {
                write!(f, "Control character while parsing string")
            }
            ErrorKind::LoneLeadingSurrogatePair => {
                write!(f, "Lone leading surrogate in hex escape")
            }
            ErrorKind::ExpectedColon(actual) => {
                write!(f, "Expected `:`, found {actual}")
            }
            ErrorKind::ExpectedOpenBrace(actual) => {
                write!(f, "Expected opening brace, found {actual}")
            }
            ErrorKind::ExpectedCloseBrace(actual) => {
                write!(f, "Expected closing brace, found {actual}")
            }
            ErrorKind::ExpectedOpenBracket(actual) => {
                write!(f, "Expected opening bracket, found {actual}")
            }
            ErrorKind::ExpectedCloseBracket(actual) => {
                write!(f, "Expected closing bracket, found {actual}")
            }
            ErrorKind::InvalidEscape => write!(f, "Invalid string escape"),
            ErrorKind::BufferUnderflow => write!(f, "Buffer underflow"),
            ErrorKind::BufferOverflow => write!(f, "Buffer overflow"),
            ErrorKind::UnexpectedHexEscapeEnd => {
                write!(f, "Unexpected end of hex escape")
            }
            ErrorKind::InvalidUnicode => write!(f, "Invalid unicode"),
            ErrorKind::CharEmptyString => {
                write!(f, "Expected string with a single character")
            }
            ErrorKind::ExpectedNull => write!(f, "Expected `null`"),
            ErrorKind::ExpectedTrue => write!(f, "Expected `true`"),
            ErrorKind::ExpectedFalse => write!(f, "Expected `false`"),
            ErrorKind::ExpectedBool(actual) => {
                write!(f, "Expected boolean, found {actual}")
            }
            ErrorKind::ExpectedString(actual) => {
                write!(f, "Expected string, found {actual}")
            }
            ErrorKind::ExpectedValue(actual) => {
                write!(f, "Expected value, found {actual}")
            }
            ErrorKind::ParseFloat(error) => {
                write!(f, "Expected float, got {error}")
            }
            ErrorKind::Eof => write!(f, "Eof while parsing"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ErrorImpl {
    SliceUnderflow(SliceUnderflow),
    SliceOverflow(SliceOverflow),
    FixedBytesOverflow(FixedBytesOverflow),
    #[cfg(feature = "std")]
    Io(std::io::Error),
    #[cfg(feature = "musli-value")]
    ValueError(musli_value::ErrorKind),
    JsonError(ErrorKind),
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
            #[cfg(feature = "musli-value")]
            ErrorImpl::ValueError(error) => error.fmt(f),
            ErrorImpl::JsonError(error) => error.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorImpl::Message(message) => message.fmt(f),
            #[cfg(not(feature = "alloc"))]
            ErrorImpl::Message => write!(f, "Message error (see diagnostics)"),
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

impl From<Infallible> for Error {
    #[inline(always)]
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[cfg(feature = "musli-value")]
impl From<musli_value::ErrorKind> for Error {
    #[inline(always)]
    fn from(error: musli_value::ErrorKind) -> Self {
        Error {
            err: ErrorImpl::ValueError(error),
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

impl musli_common::context::Error for Error {
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
