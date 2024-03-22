use crate::Context;

use super::{Decode, Decoder};

/// A pack that can construct decoders.
pub trait PackDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The encoder to use for the pack.
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
    fn decode_next(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeNext<'_>, <Self::Cx as Context>::Error>;

    /// Stop decoding the current pack.
    ///
    /// This is required to call after a pack has finished decoding.
    fn end(self, cx: &Self::Cx) -> Result<(), <Self::Cx as Context>::Error>;

    /// Unpack a value of the given type.
    #[inline]
    fn next<T>(&mut self, cx: &Self::Cx) -> Result<T, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        T: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        T::decode(cx, self.decode_next(cx)?)
    }
}
