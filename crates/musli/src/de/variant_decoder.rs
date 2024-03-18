use crate::Context;

use super::Decoder;

/// Trait governing how to decode a variant.
pub trait VariantDecoder<'de, C: ?Sized + Context> {
    /// The decoder to use for the variant tag.
    type DecodeTag<'this>: Decoder<'de, C>
    where
        Self: 'this;
    /// The decoder to use for the variant value.
    type DecodeVariant<'this>: Decoder<'de, C>
    where
        Self: 'this;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_tag(&mut self, cx: &C) -> Result<Self::DecodeTag<'_>, C::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_value(&mut self, cx: &C) -> Result<Self::DecodeVariant<'_>, C::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_value(&mut self, cx: &C) -> Result<bool, C::Error>;

    /// End the pair decoder.
    fn end(self, cx: &C) -> Result<(), C::Error>;
}
