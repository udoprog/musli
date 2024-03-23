use crate::Context;

use super::{Decode, Decoder, MapEntriesDecoder, MapEntryDecoder, SizeHint};

/// Trait governing how to decode a sequence of pairs.
pub trait MapDecoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for a key.
    type DecodeEntry<'this>: MapEntryDecoder<'de, Cx = Self::Cx>
    where
        Self: 'this;
    /// Decoder for a sequence of map pairs.
    type IntoMapEntries: MapEntriesDecoder<'de, Cx = Self::Cx>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::map_decoder]`][crate::map_decoder]
    /// attribute macro when implementing [`MapDecoder`].
    #[doc(hidden)]
    type __UseMusliMapDecoderAttributeMacro;

    /// Return the context associated with the decoder.
    fn cx(&self) -> &Self::Cx;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> SizeHint;

    /// Decode the next key. This returns `Ok(None)` where there are no more
    /// elements to decode.
    #[must_use = "Decoders must be consumed"]
    fn decode_entry(
        &mut self,
    ) -> Result<Option<Self::DecodeEntry<'_>>, <Self::Cx as Context>::Error>;

    /// End the pair decoder.
    ///
    /// If there are any remaining elements in the sequence of pairs, this
    /// indicates that they should be flushed.
    #[inline]
    fn end(mut self) -> Result<(), <Self::Cx as Context>::Error> {
        // Skip remaining elements.
        while let Some(mut item) = self.decode_entry()? {
            item.decode_map_key()?.skip()?;
            item.skip_map_value()?;
        }

        Ok(())
    }

    /// Decode the next map entry as a tuple.
    fn entry<K, V>(&mut self) -> Result<Option<(K, V)>, <Self::Cx as Context>::Error>
    where
        K: Decode<'de, <Self::Cx as Context>::Mode>,
        V: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        match self.decode_entry()? {
            Some(mut entry) => {
                let key = entry.decode_map_key()?.decode()?;
                let value = entry.decode_map_value()?.decode()?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    /// Simplified decoding a map of unknown length.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    #[inline]
    fn into_map_entries(self) -> Result<Self::IntoMapEntries, <Self::Cx as Context>::Error>
    where
        Self: Sized,
    {
        Err(self
            .cx()
            .message("Decoder does not support MapPairs decoding"))
    }
}
