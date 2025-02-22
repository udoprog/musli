use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a map entry.
pub trait EntryEncoder: Sized {
    /// Context associated with the encoder.
    type Cx: Context<Error = Self::Error>;
    /// Result type of the encoder.
    type Ok;
    /// Error associated with encoding.
    type Error;
    /// The mode of the encoder.
    type Mode: 'static;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeKey<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = Self::Error,
        Mode = Self::Mode,
    >
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeValue<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = Self::Error,
        Mode = Self::Mode,
    >
    where
        Self: 'this;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error>;

    /// Stop encoding this pair.
    fn finish_entry(self) -> Result<Self::Ok, Self::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_entry<K, V>(mut self, key: K, value: V) -> Result<Self::Ok, Self::Error>
    where
        K: Encode<Self::Mode>,
        V: Encode<Self::Mode>,
    {
        self.encode_key()?.encode(key)?;
        self.encode_value()?.encode(value)?;
        self.finish_entry()
    }
}
