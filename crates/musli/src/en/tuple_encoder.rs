use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a tuple.
pub trait TupleEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the tuple encoder.
    type EncodeTupleField<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return encoder for the next element.
    #[must_use = "Encoders must be consumed"]
    fn encode_tuple_field(
        &mut self,
    ) -> Result<Self::EncodeTupleField<'_>, <Self::Cx as Context>::Error>;

    /// Push an element into the tuple.
    #[inline]
    fn push_tuple_field<T>(&mut self, value: T) -> Result<(), <Self::Cx as Context>::Error>
    where
        T: Encode<<Self::Cx as Context>::Mode>,
    {
        self.encode_tuple_field()?.encode(value)?;
        Ok(())
    }

    /// Finish encoding the tuple.
    fn finish_tuple(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;
}
