use crate::Context;

use super::{Decoder, SizeHint};

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

    /// Get a size hint for the size of the map being decoded.
    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Any
    }

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_map_key(&mut self) -> Result<Self::DecodeMapKey<'_>, <Self::Cx as Context>::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_map_value(self) -> Result<Self::DecodeMapValue, <Self::Cx as Context>::Error>;
}
