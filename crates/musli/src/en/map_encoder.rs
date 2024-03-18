use crate::Context;

use super::{Encode, MapEntryEncoder};

/// Encoder for a map.
pub trait MapEncoder<C: ?Sized + Context> {
    /// Result type of the encoder.
    type Ok;
    /// Encode the next pair.
    type EncodeEntry<'this>: MapEntryEncoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Encode the next pair.
    fn encode_entry(&mut self, cx: &C) -> Result<Self::EncodeEntry<'_>, C::Error>;

    /// Finish encoding pairs.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;

    /// Insert a pair immediately.
    #[inline]
    fn insert_entry<F, S>(&mut self, cx: &C, key: F, value: S) -> Result<(), C::Error>
    where
        Self: Sized,
        F: Encode<C::Mode>,
        S: Encode<C::Mode>,
    {
        self.encode_entry(cx)?.insert_entry(cx, key, value)?;
        Ok(())
    }
}
