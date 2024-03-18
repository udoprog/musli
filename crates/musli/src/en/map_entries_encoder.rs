use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a map entry.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde serialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait MapEntriesEncoder<C: ?Sized + Context> {
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeMapEntryKey<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeMapEntryValue<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_map_entry_key(&mut self, cx: &C) -> Result<Self::EncodeMapEntryKey<'_>, C::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_map_entry_value(&mut self, cx: &C)
        -> Result<Self::EncodeMapEntryValue<'_>, C::Error>;

    /// Stop encoding this pair.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_entry<K, V>(&mut self, cx: &C, key: K, value: V) -> Result<(), C::Error>
    where
        K: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        key.encode(cx, self.encode_map_entry_key(cx)?)?;
        value.encode(cx, self.encode_map_entry_value(cx)?)?;
        Ok(())
    }
}
