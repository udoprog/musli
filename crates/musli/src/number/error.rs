use core::fmt;

/// An error raised while parsing the ASCII representation of a number.
///
/// The error carries the offset into the number at which it was found, so that
/// a caller can point at the offending byte rather than at the number as a
/// whole. The offset is not part of the [`Display`] output, since a caller
/// which tracks a position of its own renders it better than this can.
///
/// [`Display`]: fmt::Display
#[derive(Debug, Clone, Copy)]
pub(crate) struct Error {
    /// Offset into the number being parsed.
    at: usize,
    /// What went wrong.
    kind: ErrorKind,
}

impl Error {
    #[inline]
    pub(crate) const fn new(at: usize, kind: ErrorKind) -> Self {
        Self { at, kind }
    }

    /// The offset into the number at which the error was found.
    #[inline]
    pub(crate) const fn at(&self) -> usize {
        self.at
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// What is being read when a number is rejected, so that the diagnostic can say
/// what was expected instead of just what was found.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Expected {
    /// The first digit of the number itself.
    Number,
    /// A digit of the fraction following a point.
    Fraction,
    /// A digit of the exponent following `e` or `E`.
    Exponent,
    /// A digit of a hexadecimal number following `0x`.
    Hex,
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expected::Number => write!(f, "a digit"),
            Expected::Fraction => write!(f, "a digit in the fraction"),
            Expected::Exponent => write!(f, "a digit in the exponent"),
            Expected::Hex => write!(f, "a hexadecimal digit"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub(crate) enum ErrorKind {
    /// The number ended where more of it was expected.
    Eof(Expected),
    /// A byte was found which cannot appear where it did.
    Unexpected(Expected, u8),
    /// A redundant leading zero, which canonical JSON does not permit.
    LeadingZero,
    /// The value does not fit in the type it is being decoded into.
    Overflow,
    /// The exponent is too large to be worked with.
    ExponentOverflow,
    /// The number has a fractional part, but a whole number was expected.
    Fraction,
    /// The number could not be converted into a float.
    ///
    /// The explicit parser accepts everything the conversion does, so this is
    /// only reached if the two ever disagree.
    Float,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Eof(expected) => {
                write!(f, "Expected {expected}, but the number ended")
            }
            ErrorKind::Unexpected(expected, b) => {
                write!(f, "Expected {expected}, but found ")?;
                Byte(*b).fmt(f)
            }
            ErrorKind::LeadingZero => {
                write!(f, "A number must not have a redundant leading zero")
            }
            ErrorKind::Overflow => write!(f, "Arithmetic overflow"),
            ErrorKind::ExponentOverflow => write!(f, "Exponent is out of range"),
            ErrorKind::Fraction => write!(f, "Expected a whole number, but found a fraction"),
            ErrorKind::Float => write!(f, "Illegal float encountered"),
        }
    }
}

/// A byte rendered the way it appears in the input, falling back to its numeric
/// value when it is not printable ASCII.
struct Byte(u8);

impl fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            b'\x20'..=b'\x7e' => write!(f, "`{}`", self.0 as char),
            b => write!(f, "byte {b:#04x}"),
        }
    }
}
