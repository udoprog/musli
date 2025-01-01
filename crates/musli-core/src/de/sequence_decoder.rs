use crate::Context;

use super::{Decode, Decoder, SizeHint};

/// Trait governing how to decode a sequence.
pub trait SequenceDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context;
    /// The decoder for individual items.
    type DecodeNext<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Access the context associated with the decoder.
    fn cx(&self) -> Self::Cx;

    /// Get a size hint of known remaining elements.
    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::any()
    }

    /// Return decoder to decode the next element.
    ///
    /// This will error or provide garbled data in case the next element is not
    /// available.
    #[must_use = "Decoders must be consumed"]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, <Self::Cx as Context>::Error>;

    /// Try to decode the next element.
    #[must_use = "Decoders must be consumed"]
    fn try_decode_next(
        &mut self,
    ) -> Result<Option<Self::DecodeNext<'_>>, <Self::Cx as Context>::Error>;

    /// Decode the next element of the given type, erroring in case it's absent.
    #[inline]
    fn next<T>(&mut self) -> Result<T, <Self::Cx as Context>::Error>
    where
        T: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        self.decode_next()?.decode()
    }

    /// Decode the next element of the given type.
    #[inline]
    fn try_next<T>(&mut self) -> Result<Option<T>, <Self::Cx as Context>::Error>
    where
        T: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        let Some(decoder) = self.try_decode_next()? else {
            return Ok(None);
        };

        Ok(Some(decoder.decode()?))
    }
}
