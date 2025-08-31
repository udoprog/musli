use crate::Context;

use super::{Encode, Encoder, utils};

/// Trait governing how to encode a sequence.
pub trait SequenceEncoder {
    /// Context associated with the encoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with encoding.
    type Error;
    /// The mode of the encoder.
    type Mode: 'static;
    /// The encoder returned when advancing the sequence encoder.
    type EncodeNext<'this>: Encoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>
    where
        Self: 'this;

    /// Access the associated context.
    fn cx(&self) -> Self::Cx;

    /// Return encoder for the next element.
    #[must_use = "Encoders must be consumed"]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error>;

    /// Push an element into the sequence.
    #[inline]
    fn push<T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        self.encode_next()?.encode(value)?;
        Ok(())
    }

    /// Encode a slice of values.
    ///
    /// This can be called multiple types and has the same effect as calling
    /// `push` for each value.
    #[inline]
    fn encode_slice<T>(&mut self, slice: impl AsRef<[T]>) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        utils::default_sequence_encode_slice(self, slice)
    }

    /// Encode an iterator of contiguous slices of values.
    ///
    /// This can be called multiple types and has the same effect as calling
    /// `push` for each value.
    #[inline]
    fn encode_slices<T>(
        &mut self,
        slices: impl IntoIterator<Item: AsRef<[T]>>,
    ) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        utils::default_sequence_encode_slices(self, slices)
    }

    /// Finish encoding the sequence.
    fn finish_sequence(self) -> Result<(), Self::Error>;
}
