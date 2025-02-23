use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a map entry.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde serialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait EntriesEncoder {
    /// Context associated with the encoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with encoding.
    type Error;
    /// The mode of the encoder.
    type Mode: 'static;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeEntryKey<'this>: Encoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeEntryValue<'this>: Encoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>
    where
        Self: 'this;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error>;

    /// Complete encoding map entries.
    fn finish_entries(self) -> Result<(), Self::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_entry<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error>
    where
        K: Encode<Self::Mode>,
        V: Encode<Self::Mode>,
    {
        self.encode_entry_key()?.encode(key)?;
        self.encode_entry_value()?.encode(value)?;
        Ok(())
    }
}
