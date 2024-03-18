use crate::Context;

use super::Decoder;

/// Trait governing how to decode a map entry.
pub trait MapEntryDecoder<'de, C: ?Sized + Context> {
    /// The decoder to use for a tuple field index.
    type DecodeMapKey<'this>: Decoder<'de, C>
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeMapValue: Decoder<'de, C>;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_map_key(&mut self, cx: &C) -> Result<Self::DecodeMapKey<'_>, C::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_map_value(self, cx: &C) -> Result<Self::DecodeMapValue, C::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_map_value(self, cx: &C) -> Result<bool, C::Error>;
}
