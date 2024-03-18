use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a map entry.
pub trait MapEntryEncoder<C: ?Sized + Context> {
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeMapKey<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeMapValue<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_map_key(&mut self, cx: &C) -> Result<Self::EncodeMapKey<'_>, C::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_map_value(&mut self, cx: &C) -> Result<Self::EncodeMapValue<'_>, C::Error>;

    /// Stop encoding this pair.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_entry<K, V>(mut self, cx: &C, key: K, value: V) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        K: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        key.encode(cx, self.encode_map_key(cx)?)?;
        value.encode(cx, self.encode_map_value(cx)?)?;
        self.end(cx)
    }
}
