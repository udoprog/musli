use crate::Context;

use super::Decoder;

/// Trait governing how to decode a variant.
pub trait VariantDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with decoding.
    type Error;
    /// The mode of the decoder.
    type Mode: 'static;
    /// The decoder to use for the variant tag.
    type DecodeTag<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Mode = Self::Mode,
        Allocator = <Self::Cx as Context>::Allocator,
    >
    where
        Self: 'this;
    /// The decoder to use for the variant value.
    type DecodeValue<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Mode = Self::Mode,
        Allocator = <Self::Cx as Context>::Allocator,
    >
    where
        Self: 'this;

    /// Access the context associated with the decoder.
    fn cx(&self) -> Self::Cx;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error>;
}
