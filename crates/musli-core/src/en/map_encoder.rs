use crate::Context;

use super::{Encode, EntryEncoder};

/// Encoder for a map.
pub trait MapEncoder {
    /// Context associated with the encoder.
    type Cx: Context;
    /// Result type of the encoder.
    type Ok;
    /// The mode of the encoder.
    type Mode: 'static;
    /// Encode the next pair.
    type EncodeEntry<'this>: EntryEncoder<Cx = Self::Cx, Ok = Self::Ok, Mode = Self::Mode>
    where
        Self: 'this;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// Encode the next pair.
    #[must_use = "Encoders must be consumed"]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, <Self::Cx as Context>::Error>;

    /// Simplified encoder for a map entry, which ensures that encoding is
    /// always finished.
    #[inline]
    fn encode_entry_fn<F>(&mut self, f: F) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodeEntry<'_>) -> Result<(), <Self::Cx as Context>::Error>,
    {
        let mut encoder = self.encode_entry()?;
        f(&mut encoder)?;
        encoder.finish_entry()
    }

    /// Insert a pair immediately.
    #[inline]
    fn insert_entry<F, S>(&mut self, key: F, value: S) -> Result<(), <Self::Cx as Context>::Error>
    where
        Self: Sized,
        F: Encode<Self::Mode>,
        S: Encode<Self::Mode>,
    {
        self.encode_entry()?.insert_entry(key, value)?;
        Ok(())
    }

    /// Finish encoding pairs.
    fn finish_map(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;
}
