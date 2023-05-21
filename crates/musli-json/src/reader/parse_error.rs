use core::fmt;

use musli::error::Error;

use crate::reader::integer;
use crate::reader::Token;

/// An input error recorded at the given location.
#[derive(Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum ParseError {
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
    InvalidNumeric,
    CharEmptyString,
    ExpectedNull,
    ExpectedTrue,
    ExpectedFalse,
    ExpectedBool(Token),
    ExpectedString(Token),
    ExpectedValue(Token),
    ParseFloat(lexical::Error),
    IntegerError(integer::Error),
    #[cfg(feature = "musli-value")]
    ValueError(musli_value::ValueError),
    Eof,
    Custom,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ControlCharacterInString => {
                write!(f, "control character while parsing string")
            }
            ParseError::LoneLeadingSurrogatePair => {
                write!(f, "lone leading surrogate in hex escape")
            }
            ParseError::ExpectedColon(actual) => {
                write!(f, "expected `:`, found {actual}")
            }
            ParseError::ExpectedOpenBrace(actual) => {
                write!(f, "expected opening brace, found {actual}")
            }
            ParseError::ExpectedCloseBrace(actual) => {
                write!(f, "expected closing brace, found {actual}")
            }
            ParseError::ExpectedOpenBracket(actual) => {
                write!(f, "expected opening bracket, found {actual}")
            }
            ParseError::ExpectedCloseBracket(actual) => {
                write!(f, "expected closing bracket, found {actual}")
            }
            ParseError::InvalidEscape => write!(f, "invalid string escape"),
            ParseError::BufferUnderflow => write!(f, "buffer underflow"),
            ParseError::BufferOverflow => write!(f, "buffer overflow"),
            ParseError::UnexpectedHexEscapeEnd => {
                write!(f, "unexpected end of hex escape")
            }
            ParseError::InvalidUnicode => write!(f, "invalid unicode"),
            ParseError::InvalidNumeric => write!(f, "not numeric"),
            ParseError::CharEmptyString => {
                write!(f, "expected string with a single character")
            }
            ParseError::ExpectedNull => write!(f, "expected `null`"),
            ParseError::ExpectedTrue => write!(f, "expected `true`"),
            ParseError::ExpectedFalse => write!(f, "expected `false`"),
            ParseError::ExpectedBool(actual) => {
                write!(f, "expected boolean, found {actual}")
            }
            ParseError::ExpectedString(actual) => {
                write!(f, "expected string, found {actual}")
            }
            ParseError::ExpectedValue(actual) => {
                write!(f, "expected value, found {actual}")
            }
            ParseError::ParseFloat(error) => {
                write!(f, "expected float, got {error}")
            }
            ParseError::IntegerError(error) => {
                write!(f, "expected integer, got {error}")
            }
            #[cfg(feature = "musli-value")]
            ParseError::ValueError(error) => {
                write!(f, "value error: {error}")
            }
            ParseError::Eof => write!(f, "eof while parsing"),
            ParseError::Custom => {
                write!(f, "custom error")
            }
        }
    }
}

impl Error for ParseError {
    fn custom<T>(#[allow(unused)] error: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self::Custom
    }

    fn message<T>(#[allow(unused)] message: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

#[cfg(feature = "musli-value")]
impl From<musli_value::ValueError> for ParseError {
    #[inline(always)]
    fn from(error: musli_value::ValueError) -> Self {
        ParseError::ValueError(error)
    }
}
