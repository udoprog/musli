use crate::Context;

use super::Decoder;

/// Trait that allows a type to be repeatedly coerced into a decoder.
pub trait AsDecoder {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder we reborrow as.
    type Decoder<'this>: Decoder<
        'this,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Borrow self as a new decoder.
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, <Self::Cx as Context>::Error>;
}
