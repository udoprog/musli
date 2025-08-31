use crate::{Allocator, Context};

use super::{Decoder, SizeHint};

/// Trait governing how to decode a sequence of map pairs.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde deserialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
#[must_use = "Must call end_entries to complete decoding"]
pub trait EntriesDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context<Error = Self::Error, Allocator = Self::Allocator>;
    /// Error associated with decoding.
    type Error;
    /// The allocator associated with the decoder.
    type Allocator: Allocator;
    /// The mode of the decoder.
    type Mode: 'static;
    /// The decoder to use for a tuple field index.
    type DecodeEntryKey<'this>: Decoder<
            'de,
            Cx = Self::Cx,
            Error = Self::Error,
            Allocator = Self::Allocator,
            Mode = Self::Mode,
        >
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeEntryValue<'this>: Decoder<
            'de,
            Cx = Self::Cx,
            Error = Self::Error,
            Allocator = Self::Allocator,
            Mode = Self::Mode,
        >
    where
        Self: 'this;

    /// Access the context associated with the decoder.
    fn cx(&self) -> Self::Cx;

    /// Get a size hint for the size of the map being decoded.
    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::any()
    }

    /// Try to return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error>;

    /// Decode the value in the map.
    #[must_use = "Decoders must be consumed"]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error>;

    /// End entries decoding.
    fn end_entries(self) -> Result<(), Self::Error>;
}
