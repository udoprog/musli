use core::fmt;

crate::macros::implement_error! {
    /// Error raised during JSON encoding.
    pub struct Error;
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
