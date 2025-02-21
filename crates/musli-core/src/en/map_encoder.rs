use crate::Context;

use super::{Encode, EntryEncoder};

/// Encoder for a map.
pub trait MapEncoder {
    /// Context associated with the encoder.
    type Cx: Context<Error = Self::Error>;
    /// Result type of the encoder.
    type Ok;
    /// Error associated with encoding.
    type Error;
    /// The mode of the encoder.
    type Mode: 'static;
    /// Encode the next pair.
    type EncodeEntry<'this>: EntryEncoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = Self::Error,
        Mode = Self::Mode,
    >
    where
        Self: 'this;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// Encode the next map entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error>;

    /// Simplified encoder for a map entry, which ensures that encoding is
    /// always finished.
    #[inline]
    fn encode_entry_fn<F>(&mut self, f: F) -> Result<Self::Ok, Self::Error>
    where
        F: FnOnce(&mut Self::EncodeEntry<'_>) -> Result<(), Self::Error>,
    {
        let mut encoder = self.encode_entry()?;
        f(&mut encoder)?;
        encoder.finish_entry()
    }

    /// Insert a map entry.
    #[inline]
    fn insert_entry<F, S>(&mut self, key: F, value: S) -> Result<(), Self::Error>
    where
        F: Encode<Self::Mode>,
        S: Encode<Self::Mode>,
    {
        self.encode_entry()?.insert_entry(key, value)?;
        Ok(())
    }

    /// Finish encoding a map.
    fn finish_map(self) -> Result<Self::Ok, Self::Error>;
}
