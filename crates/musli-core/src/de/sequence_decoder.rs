use crate::Context;

use super::{Decode, Decoder, SizeHint};

/// Trait governing how to decode a sequence.
pub trait SequenceDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with decoding.
    type Error;
    /// The mode of the decoder.
    type Mode: 'static;
    /// The decoder for individual items.
    type DecodeNext<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Mode = Self::Mode,
        Allocator = <Self::Cx as Context>::Allocator,
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
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error>;

    /// Try to decode the next element.
    #[must_use = "Decoders must be consumed"]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, Self::Error>;

    /// Decode the next element of the given type, erroring in case it's absent.
    #[inline]
    fn next<T>(&mut self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode, <Self::Cx as Context>::Allocator>,
    {
        self.decode_next()?.decode()
    }

    /// Decode the next element of the given type.
    #[inline]
    fn try_next<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Decode<'de, Self::Mode, <Self::Cx as Context>::Allocator>,
    {
        let Some(decoder) = self.try_decode_next()? else {
            return Ok(None);
        };

        Ok(Some(decoder.decode()?))
    }
}
