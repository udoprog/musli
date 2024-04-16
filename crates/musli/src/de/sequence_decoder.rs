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
    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Any
    }

    /// Decode the next element.
    #[must_use = "Decoders must be consumed"]
    fn decode_next(&mut self)
        -> Result<Option<Self::DecodeNext<'_>>, <Self::Cx as Context>::Error>;

    /// Decode the next element, erroring in case it's absent.
    #[must_use = "Decoders must be consumed"]
    #[inline]
    fn decode_next_required(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeNext<'_>, <Self::Cx as Context>::Error> {
        let Some(decoder) = self.decode_next()? else {
            return Err(cx.message("Missing required sequence element"));
        };

        Ok(decoder)
    }

    /// Decode the next element of the given type.
    #[inline]
    fn next<T>(&mut self) -> Result<Option<T>, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        T: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        let Some(decoder) = self.decode_next()? else {
            return Ok(None);
        };

        Ok(Some(decoder.decode()?))
    }

    /// Decode the next element of the given type, erroring in case it's absent.
    #[inline]
    fn next_required<T>(&mut self, cx: &Self::Cx) -> Result<T, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        T: Decode<'de, <Self::Cx as Context>::Mode>,
    {
        Ok(self.decode_next_required(cx)?.decode()?)
    }
}
