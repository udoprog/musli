use crate::Context;

use super::{Decode, Decoder, SizeHint};

/// Trait governing how to decode a sequence.
pub trait SequenceDecoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder for individual items.
    type DecodeNext<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self, cx: &Self::Cx) -> SizeHint;

    /// Decode the next element.
    #[must_use = "Decoders must be consumed"]
    fn decode_next(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Option<Self::DecodeNext<'_>>, <Self::Cx as Context>::Error>;

    /// Stop decoding the current sequence.
    ///
    /// This is required to call after a sequence has finished decoding.
    #[inline]
    fn end(mut self, cx: &Self::Cx) -> Result<(), <Self::Cx as Context>::Error> {
        while let Some(item) = self.decode_next(cx)? {
            item.skip(cx)?;
        }

        Ok(())
    }

    /// Decode the next element of the given type.
    #[inline]
    fn next<T>(&mut self, cx: &Self::Cx) -> Result<Option<T>, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        T: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        let Some(decoder) = self.decode_next(cx)? else {
            return Ok(None);
        };

        Ok(Some(T::decode(cx, decoder)?))
    }
}
