use core::fmt;

use super::type_hint::{NumberHint, TypeHint};

crate::macros::implement_error! {
    /// Error raised during wire encoding.
    pub struct Error;
}

/// Errors specifically produced by value decoding.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub(crate) enum ErrorMessage {
    ArrayOutOfBounds,
    ExpectedPackValue,
    ExpectedEmpty(TypeHint),
    ExpectedBool(TypeHint),
    ExpectedChar(TypeHint),
    ExpectedNumber(NumberHint, TypeHint),
    ExpectedMapValue,
    ExpectedBytes(TypeHint),
    ExpectedString(TypeHint),
    ExpectedStringAsNumber,
    ExpectedSequence(TypeHint),
    ExpectedPack(TypeHint),
    ExpectedMap(TypeHint),
    ExpectedVariant(TypeHint),
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorMessage::ArrayOutOfBounds => {
                write!(
                    f,
                    "Value buffer tried to decode array that is out-of-bounds"
                )
            }
            ErrorMessage::ExpectedPackValue => write!(f, "Value buffer expected pack value"),
            ErrorMessage::ExpectedEmpty(hint) => {
                write!(f, "Value buffer expected empty, but found {hint}")
            }
            ErrorMessage::ExpectedBool(hint) => {
                write!(f, "Value buffer expected boolean, but found {hint}")
            }
            ErrorMessage::ExpectedChar(hint) => {
                write!(f, "Value buffer expected character, but found {hint}")
            }
            ErrorMessage::ExpectedNumber(number, hint) => {
                write!(f, "Value buffer expected {number}, but found {hint}")
            }
            ErrorMessage::ExpectedMapValue => write!(f, "Value buffer expected map value"),
            ErrorMessage::ExpectedBytes(hint) => {
                write!(f, "Value buffer expected bytes, but found {hint}")
            }
            ErrorMessage::ExpectedString(hint) => {
                write!(f, "Value buffer expected string, but found {hint}")
            }
            ErrorMessage::ExpectedStringAsNumber => {
                write!(f, "Value buffer expected string containing number")
            }
            ErrorMessage::ExpectedSequence(hint) => {
                write!(f, "Value buffer expected sequence, but found {hint}")
            }
            ErrorMessage::ExpectedPack(hint) => {
                write!(f, "Value buffer expected pack of bytes, but found {hint}")
            }
            ErrorMessage::ExpectedMap(hint) => {
                write!(f, "Value buffer expected map, but found {hint}")
            }
            ErrorMessage::ExpectedVariant(hint) => {
                write!(f, "Value buffer expected variant, but found {hint}")
            }
        }
    }
}
