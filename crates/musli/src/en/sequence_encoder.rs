use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a sequence.
pub trait SequenceEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the sequence encoder.
    type EncodeNext<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return encoder for the next element.
    #[must_use = "Encoder must be consumed"]
    fn encode_next(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::EncodeNext<'_>, <Self::Cx as Context>::Error>;

    /// Finish encoding the sequence.
    fn end(self, cx: &Self::Cx) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Push an element into the sequence.
    #[inline]
    fn push<T>(&mut self, cx: &Self::Cx, value: T) -> Result<(), <Self::Cx as Context>::Error>
    where
        T: Encode<<Self::Cx as Context>::Mode>,
    {
        let encoder = self.encode_next(cx)?;
        value.encode(cx, encoder)?;
        Ok(())
    }
}
