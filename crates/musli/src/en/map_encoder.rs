use crate::Context;

use super::{Encode, MapEntryEncoder};

/// Encoder for a map.
pub trait MapEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// Encode the next pair.
    type EncodeMapEntry<'this>: MapEntryEncoder<Cx = Self::Cx, Ok = Self::Ok>
    where
        Self: 'this;

    /// Encode the next pair.
    #[must_use = "Encoders must be consumed"]
    fn encode_map_entry(
        &mut self,
    ) -> Result<Self::EncodeMapEntry<'_>, <Self::Cx as Context>::Error>;

    /// Simplified encoder for a map entry, which ensures that encoding is
    /// always finished.
    #[inline]
    fn encode_map_entry_fn<F>(&mut self, f: F) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodeMapEntry<'_>) -> Result<(), <Self::Cx as Context>::Error>,
    {
        let mut encoder = self.encode_map_entry()?;
        f(&mut encoder)?;
        encoder.finish_map_entry()
    }

    /// Finish encoding pairs.
    fn finish_map(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Insert a pair immediately.
    #[inline]
    fn insert_entry<F, S>(&mut self, key: F, value: S) -> Result<(), <Self::Cx as Context>::Error>
    where
        Self: Sized,
        F: Encode<<Self::Cx as Context>::Mode>,
        S: Encode<<Self::Cx as Context>::Mode>,
    {
        self.encode_map_entry()?.insert_entry(key, value)?;
        Ok(())
    }
}
