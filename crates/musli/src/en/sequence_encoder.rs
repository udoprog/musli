use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a sequence.
pub trait SequenceEncoder<C: ?Sized + Context> {
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the sequence encoder.
    type EncodeNext<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Return encoder for the next element.
    #[must_use = "Encoder must be consumed"]
    fn encode_next(&mut self, cx: &C) -> Result<Self::EncodeNext<'_>, C::Error>;

    /// Finish encoding the sequence.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;

    /// Push an element into the sequence.
    #[inline]
    fn push<T>(&mut self, cx: &C, value: T) -> Result<(), C::Error>
    where
        T: Encode<C::Mode>,
    {
        let encoder = self.encode_next(cx)?;
        value.encode(cx, encoder)?;
        Ok(())
    }
}
