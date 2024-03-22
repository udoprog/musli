use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a map entry.
pub trait MapEntryEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeMapKey<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeMapValue<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_map_key(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::EncodeMapKey<'_>, <Self::Cx as Context>::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_map_value(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::EncodeMapValue<'_>, <Self::Cx as Context>::Error>;

    /// Stop encoding this pair.
    fn end(self, cx: &Self::Cx) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_entry<K, V>(
        mut self,
        cx: &Self::Cx,
        key: K,
        value: V,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        K: Encode<<Self::Cx as Context>::Mode>,
        V: Encode<<Self::Cx as Context>::Mode>,
    {
        key.encode(cx, self.encode_map_key(cx)?)?;
        value.encode(cx, self.encode_map_value(cx)?)?;
        self.end(cx)
    }
}
