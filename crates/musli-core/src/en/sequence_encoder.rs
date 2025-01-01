use crate::Context;

use super::{utils, Encode, Encoder};

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

    /// Perform an operation while accessing the context.
    ///
    /// This access the context while providing a mutable reference to the
    /// sequence encoder.
    fn cx_mut<F, O>(&mut self, f: F) -> O
    where
        F: FnOnce(&Self::Cx, &mut Self) -> O;

    /// Return encoder for the next element.
    #[must_use = "Encoders must be consumed"]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, <Self::Cx as Context>::Error>;

    /// Push an element into the sequence.
    #[inline]
    fn push<T>(&mut self, value: T) -> Result<(), <Self::Cx as Context>::Error>
    where
        T: Encode<<Self::Cx as Context>::Mode>,
    {
        self.encode_next()?.encode(value)?;
        Ok(())
    }

    /// Encode a slice of values.
    ///
    /// This can be called multiple types and has the same effect as calling
    /// `push` for each value.
    #[inline]
    fn encode_slice<T>(
        &mut self,
        slice: impl AsRef<[T]>,
    ) -> Result<(), <Self::Cx as Context>::Error>
    where
        T: Encode<<Self::Cx as Context>::Mode>,
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
    ) -> Result<(), <Self::Cx as Context>::Error>
    where
        T: Encode<<Self::Cx as Context>::Mode>,
    {
        utils::default_sequence_encode_slices(self, slices)
    }

    /// Finish encoding the sequence.
    fn finish_sequence(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;
}
