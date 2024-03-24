use crate::Context;

use super::{Decode, Decoder};

/// A decoder for tuples.
pub trait TupleDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The encoder to use for the tuple element.
    type DecodeNext<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return decoder to unpack the next element.
    #[must_use = "Decoders must be consumed"]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, <Self::Cx as Context>::Error>;

    /// Unpack a value of the given type.
    #[inline]
    fn next<T>(&mut self) -> Result<T, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        T: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        self.decode_next()?.decode()
    }
}
