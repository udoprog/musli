use crate::Context;

use super::Decoder;

/// Trait governing how to decode a map entry.
pub trait MapEntryDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for a tuple field index.
    type DecodeMapKey<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeMapValue: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_map_key(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeMapKey<'_>, <Self::Cx as Context>::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_map_value(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeMapValue, <Self::Cx as Context>::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_map_value(self, cx: &Self::Cx) -> Result<bool, <Self::Cx as Context>::Error>;
}
