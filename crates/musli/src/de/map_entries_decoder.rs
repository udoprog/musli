use crate::Context;

use super::Decoder;

/// Trait governing how to decode a sequence of map pairs.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde deserialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait MapEntriesDecoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for a tuple field index.
    type DecodeMapEntryKey<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeMapEntryValue<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Try to return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_map_entry_key(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Option<Self::DecodeMapEntryKey<'_>>, <Self::Cx as Context>::Error>;

    /// Decode the value in the map.
    #[must_use = "Decoders must be consumed"]
    fn decode_map_entry_value(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeMapEntryValue<'_>, <Self::Cx as Context>::Error>;

    /// Indicate that the value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_map_entry_value(&mut self, cx: &Self::Cx)
        -> Result<bool, <Self::Cx as Context>::Error>;

    /// End pair decoding.
    #[inline]
    fn end(mut self, cx: &Self::Cx) -> Result<(), <Self::Cx as Context>::Error> {
        loop {
            let Some(item) = self.decode_map_entry_key(cx)? else {
                break;
            };

            item.skip(cx)?;
            self.skip_map_entry_value(cx)?;
        }

        Ok(())
    }
}
