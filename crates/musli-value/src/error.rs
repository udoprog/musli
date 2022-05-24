use core::fmt;

use musli::de::{NumberHint, TypeHint};
use musli::error::Error;

/// An error raised when encoding or decoding [Value][crate::Value].
#[allow(missing_docs)]
#[derive(Debug)]
#[non_exhaustive]
pub enum ValueError {
    ArrayOutOfBounds,
    ExpectedPackValue,
    ExpectedUnit(TypeHint),
    ExpectedBool(TypeHint),
    ExpectedChar(TypeHint),
    ExpectedNumber(NumberHint, TypeHint),
    ExpectedBytes(TypeHint),
    ExpectedString(TypeHint),
    ExpectedOption(TypeHint),
    ExpectedSequence(TypeHint),
    ExpectedPack(TypeHint),
    ExpectedMap(TypeHint),
    ExpectedVariant(TypeHint),
    Custom,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueError::ArrayOutOfBounds => {
                write!(f, "tried to decode array that is out-of-bounds")
            }
            ValueError::ExpectedPackValue => write!(f, "expected pack value"),
            ValueError::ExpectedUnit(hint) => write!(f, "expected unit, but found {hint}"),
            ValueError::ExpectedBool(hint) => write!(f, "expected boolean, but found {hint}"),
            ValueError::ExpectedChar(hint) => write!(f, "expected character, but found {hint}"),
            ValueError::ExpectedNumber(number, hint) => {
                write!(f, "expected {number}, but found {hint}")
            }
            ValueError::ExpectedBytes(hint) => write!(f, "expected bytes, but found {hint}"),
            ValueError::ExpectedString(hint) => write!(f, "expected string, but found {hint}"),
            ValueError::ExpectedOption(hint) => write!(f, "expected option, but found {hint}"),
            ValueError::ExpectedSequence(hint) => write!(f, "expected sequence, but found {hint}"),
            ValueError::ExpectedPack(hint) => write!(f, "expected pack, but found {hint}"),
            ValueError::ExpectedMap(hint) => write!(f, "expected map, but found {hint}"),
            ValueError::ExpectedVariant(hint) => write!(f, "expected struct, but found {hint}"),
            ValueError::Custom => write!(f, "custom error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ValueError {}

impl Error for ValueError {
    fn custom<T>(_: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self::Custom
    }

    fn message<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom
    }
}
