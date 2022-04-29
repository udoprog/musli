use core::fmt;

use musli::error::Error;

use crate::reader::Token;

/// The span of an error.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    #[inline]
    const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline]
    const fn point(at: u32) -> Self {
        Self {
            start: at,
            end: at.saturating_add(1),
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            self.start.fmt(f)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

/// The kind of the parse error.
#[derive(Debug)]
pub(crate) enum ParseErrorKind {
    ControlCharacterInString,
    LoneLeadingSurrogatePair,
    ExpectedOpenBrace(Token),
    ExpectedOpenBracket(Token),
    InvalidEscape,
    BufferUnderflow,
    BufferOverflow,
    UnexpectedHexEscapeEnd,
    InvalidUnicode,
    NumericalOverflow,
    ExpectedWholeNumber,
    InvalidNumeric,
    UnsupportedExponent,
    ExpectedNull,
    ExpectedTrue,
    ExpectedFalse,
    ExpectedBool(Token),
    ExpectedValue(Token),
    Eof,
    Custom,
}

/// An input error recorded at the given location.
#[derive(Debug)]
pub struct ParseError {
    // Position of the parse error.
    span: Span,
    kind: ParseErrorKind,
    #[cfg(feature = "std")]
    custom: Option<Box<str>>,
}

impl ParseError {
    #[inline]
    pub(crate) fn at(at: u32, kind: ParseErrorKind) -> Self {
        Self {
            span: Span::point(at),
            kind,
            #[cfg(feature = "std")]
            custom: None,
        }
    }

    #[inline]
    pub(crate) fn spanned(start: u32, end: u32, kind: ParseErrorKind) -> Self {
        Self {
            span: Span::new(start, end),
            kind,
            #[cfg(feature = "std")]
            custom: None,
        }
    }

    /// Get the span of the parse error.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let span = self.span;

        match &self.kind {
            ParseErrorKind::ControlCharacterInString => {
                write!(f, "control character while parsing string (at {span})")
            }
            ParseErrorKind::LoneLeadingSurrogatePair => {
                write!(f, "lone leading surrogate in hex escape (at {span})")
            }
            ParseErrorKind::ExpectedOpenBrace(actual) => {
                write!(f, "expected opening brace, found {actual} (at {span})")
            }
            ParseErrorKind::ExpectedOpenBracket(actual) => {
                write!(f, "expected opening bracket, found {actual} (at {span})")
            }
            ParseErrorKind::InvalidEscape => write!(f, "invalid string escape (at {span})"),
            ParseErrorKind::BufferUnderflow => write!(f, "buffer underflow (at {span})"),
            ParseErrorKind::BufferOverflow => write!(f, "buffer overflow (at {span})"),
            ParseErrorKind::UnexpectedHexEscapeEnd => {
                write!(f, "unexpected end of hex escape (at {span})")
            }
            ParseErrorKind::InvalidUnicode => write!(f, "invalid unicode (at {span})"),
            ParseErrorKind::NumericalOverflow => write!(f, "numerical overflow (at {span})"),
            ParseErrorKind::ExpectedWholeNumber => write!(f, "expected whole number (at {span})"),
            ParseErrorKind::InvalidNumeric => write!(f, "not numeric (at {span})"),
            ParseErrorKind::UnsupportedExponent => write!(f, "unsupported exponent (at {span})"),
            ParseErrorKind::ExpectedNull => write!(f, "expected `null` (at {span})"),
            ParseErrorKind::ExpectedTrue => write!(f, "expected `true` (at {span})"),
            ParseErrorKind::ExpectedFalse => write!(f, "expected `false` (at {span})"),
            ParseErrorKind::ExpectedBool(actual) => {
                write!(f, "expected boolean, found {actual} (at {span})")
            }
            ParseErrorKind::ExpectedValue(actual) => {
                write!(f, "expected value, found {actual} (at {span})")
            }
            ParseErrorKind::Eof => write!(f, "eof while parsing (at {span})"),
            ParseErrorKind::Custom => {
                #[cfg(feature = "std")]
                if let Some(custom) = &self.custom {
                    write!(f, "{}", custom)
                } else {
                    write!(f, "custom error")
                }

                #[cfg(not(feature = "std"))]
                {
                    write!(f, "custom error")
                }
            }
        }
    }
}

impl Error for ParseError {
    fn custom<T>(#[allow(unused)] error: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self {
            kind: ParseErrorKind::Custom,
            span: Span::empty(),
            #[cfg(feature = "std")]
            custom: Some(error.to_string().into()),
        }
    }

    fn message<T>(#[allow(unused)] message: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            kind: ParseErrorKind::Custom,
            span: Span::empty(),
            #[cfg(feature = "std")]
            custom: Some(message.to_string().into()),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
