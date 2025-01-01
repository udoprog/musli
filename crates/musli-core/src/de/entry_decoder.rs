use crate::Context;

use super::{Decoder, SizeHint};

/// Trait governing how to decode a map entry.
pub trait EntryDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context;
    /// The decoder to use for a tuple field index.
    type DecodeKey<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeValue: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >;

    /// Access the context associated with the decoder.
    fn cx(&self) -> Self::Cx;

    /// Get a size hint for the size of the map being decoded.
    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::any()
    }

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, <Self::Cx as Context>::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_value(self) -> Result<Self::DecodeValue, <Self::Cx as Context>::Error>;
}
