use crate::Context;

use super::Decoder;

/// Trait that allows a type to be repeatedly coerced into a decoder.
pub trait AsDecoder<C: ?Sized + Context> {
    /// The decoder we reborrow as.
    type Decoder<'this>: Decoder<'this, C>
    where
        Self: 'this;

    /// Borrow self as a new decoder.
    fn as_decoder(&self, cx: &C) -> Result<Self::Decoder<'_>, C::Error>;
}
