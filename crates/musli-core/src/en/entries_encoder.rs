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
    type Cx: Context;
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeEntryKey<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeEntryValue<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_entry_key(
        &mut self,
    ) -> Result<Self::EncodeEntryKey<'_>, <Self::Cx as Context>::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_entry_value(
        &mut self,
    ) -> Result<Self::EncodeEntryValue<'_>, <Self::Cx as Context>::Error>;

    /// Complete encoding map entries.
    fn finish_entries(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_entry<K, V>(&mut self, key: K, value: V) -> Result<(), <Self::Cx as Context>::Error>
    where
        K: Encode<<Self::Cx as Context>::Mode>,
        V: Encode<<Self::Cx as Context>::Mode>,
    {
        self.encode_entry_key()?.encode(key)?;
        self.encode_entry_value()?.encode(value)?;
        Ok(())
    }
}
