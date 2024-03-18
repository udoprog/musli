use crate::Context;

use super::{Decoder, SizeHint};

/// Trait governing how to decode a sequence.
pub trait SequenceDecoder<'de, C: ?Sized + Context> {
    /// The decoder for individual items.
    type DecodeNext<'this>: Decoder<'de, C>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self, cx: &C) -> SizeHint;

    /// Decode the next element.
    #[must_use = "Decoders must be consumed"]
    fn decode_next(&mut self, cx: &C) -> Result<Option<Self::DecodeNext<'_>>, C::Error>;

    /// Stop decoding the current sequence.
    ///
    /// This is required to call after a sequence has finished decoding.
    fn end(self, cx: &C) -> Result<(), C::Error>;
}
