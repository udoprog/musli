use crate::Context;

use super::Decoder;

/// A pack that can construct decoders.
pub trait PackDecoder<'de, C: ?Sized + Context> {
    /// The encoder to use for the pack.
    type DecodeNext<'this>: Decoder<'de, C>
    where
        Self: 'this;

    /// Return decoder to unpack the next element.
    #[must_use = "Decoders must be consumed"]
    fn decode_next(&mut self, cx: &C) -> Result<Self::DecodeNext<'_>, C::Error>;

    /// Stop decoding the current pack.
    ///
    /// This is required to call after a pack has finished decoding.
    fn end(self, cx: &C) -> Result<(), C::Error>;
}
