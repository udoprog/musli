use crate::Context;

use super::Decoder;

/// Trait that allows a type to be repeatedly coerced into a decoder.
pub trait AsDecoder {
    /// Context associated with the decoder.
    type Cx: Context;
    /// The mode of the decoder.
    type Mode: 'static;
    /// The decoder we reborrow as.
    type Decoder<'this>: Decoder<
        'this,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = Self::Mode,
        Allocator = <Self::Cx as Context>::Allocator,
    >
    where
        Self: 'this;

    /// Borrow self as a new decoder.
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, <Self::Cx as Context>::Error>;
}
