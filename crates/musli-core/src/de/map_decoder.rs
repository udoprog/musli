use crate::Context;

use super::{Decode, Decoder, EntriesDecoder, EntryDecoder, SizeHint};

/// Trait governing how to decode a sequence of pairs.
pub trait MapDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with decoding.
    type Error;
    /// The mode of the decoder.
    type Mode: 'static;
    /// The decoder to use for a key.
    type DecodeEntry<'this>: EntryDecoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Mode = Self::Mode,
    >
    where
        Self: 'this;
    /// Decoder returned by [`MapDecoder::decode_remaining_entries`].
    type DecodeRemainingEntries<'this>: EntriesDecoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Mode = Self::Mode,
    >
    where
        Self: 'this;

    /// Access the context associated with the decoder.
    fn cx(&self) -> Self::Cx;

    /// Get a size hint of known remaining elements.
    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::any()
    }

    /// Decode the next key. This returns `Ok(None)` where there are no more
    /// elements to decode.
    #[must_use = "Decoders must be consumed"]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, Self::Error>;

    /// Return simplified decoder for remaining entries.
    fn decode_remaining_entries(&mut self)
        -> Result<Self::DecodeRemainingEntries<'_>, Self::Error>;

    /// Decode the next map entry as a tuple.
    fn entry<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
    where
        K: Decode<'de, Self::Mode, <Self::Cx as Context>::Allocator>,
        V: Decode<'de, Self::Mode, <Self::Cx as Context>::Allocator>,
    {
        let Some(mut entry) = self.decode_entry()? else {
            return Ok(None);
        };

        let key = entry.decode_key()?.decode()?;
        let value = entry.decode_value()?.decode()?;
        Ok(Some((key, value)))
    }
}
