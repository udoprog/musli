use crate::Context;

use super::{MapEntriesDecoder, MapEntryDecoder, SizeHint};

/// Trait governing how to decode a sequence of pairs.
pub trait MapDecoder<'de, C: ?Sized + Context> {
    /// The decoder to use for a key.
    type DecodeEntry<'this>: MapEntryDecoder<'de, C>
    where
        Self: 'this;

    /// Decoder for a sequence of map pairs.
    type IntoMapEntries: MapEntriesDecoder<'de, C>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::map_decoder]`][crate::map_decoder]
    /// attribute macro when implementing [`MapDecoder`].
    #[doc(hidden)]
    type __UseMusliMapDecoderAttributeMacro;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self, cx: &C) -> SizeHint;

    /// Decode the next key. This returns `Ok(None)` where there are no more
    /// elements to decode.
    #[must_use = "Decoders must be consumed"]
    fn decode_entry(&mut self, cx: &C) -> Result<Option<Self::DecodeEntry<'_>>, C::Error>;

    /// End the pair decoder.
    ///
    /// If there are any remaining elements in the sequence of pairs, this
    /// indicates that they should be flushed.
    fn end(self, cx: &C) -> Result<(), C::Error>;

    /// Simplified decoding a map of unknown length.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    #[inline]
    fn into_map_entries(self, cx: &C) -> Result<Self::IntoMapEntries, C::Error>
    where
        Self: Sized,
    {
        Err(cx.message("Decoder does not support MapPairs decoding"))
    }
}
