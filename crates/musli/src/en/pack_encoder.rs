use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a pack.
pub trait PackEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the pack encoder.
    type EncodePacked<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return encoder for the next element.
    #[must_use = "Encoders must be consumed"]
    fn encode_packed(&mut self) -> Result<Self::EncodePacked<'_>, <Self::Cx as Context>::Error>;

    /// Finish encoding the pack.
    fn finish_pack(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Push an element into the pack.
    #[inline]
    fn push<T>(&mut self, value: T) -> Result<(), <Self::Cx as Context>::Error>
    where
        T: Encode<<Self::Cx as Context>::Mode>,
    {
        self.encode_packed()?.encode(value)?;
        Ok(())
    }
}
