use crate::{Allocator, Context};

use super::{Decoder, SizeHint};

/// Trait governing how to decode a map entry.
pub trait EntryDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context<Error = Self::Error, Allocator = Self::Allocator>;
    /// Error associated with decoding.
    type Error;
    /// The allocator associated with the decoder.
    type Allocator: Allocator;
    /// The mode of the decoder.
    type Mode: 'static;
    /// The decoder to use for a tuple field index.
    type DecodeKey<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeValue: Decoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
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
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_value(self) -> Result<Self::DecodeValue, Self::Error>;
}
