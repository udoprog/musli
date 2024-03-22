use crate::Context;

use super::Decoder;

/// Trait governing how to decode a variant.
pub trait VariantDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for the variant tag.
    type DecodeTag<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The decoder to use for the variant value.
    type DecodeVariant<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_tag(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeTag<'_>, <Self::Cx as Context>::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_value(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeVariant<'_>, <Self::Cx as Context>::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_value(&mut self, cx: &Self::Cx) -> Result<bool, <Self::Cx as Context>::Error>;

    /// End the pair decoder.
    fn end(self, cx: &Self::Cx) -> Result<(), <Self::Cx as Context>::Error>;
}
