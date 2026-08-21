use core::marker::PhantomData;

use crate::Context;
use crate::de::{SequenceDecoder, SizeHint};

use super::super::cursor::Cursor;
use super::JsonbDecoder;

/// A decoder over the payload of a JSONB array, which is simply the elements
/// it contains written after one another.
pub(crate) struct JsonbSequenceDecoder<P, C, M> {
    cx: C,
    cursor: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonbSequenceDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, cursor: P) -> Self {
        Self {
            cx,
            cursor,
            _marker: PhantomData,
        }
    }

    /// Finish decoding the array.
    ///
    /// Any elements which have not been decoded are contained in the payload
    /// which the enclosing decoder has already moved past, so there is nothing
    /// left to skip over.
    #[inline]
    pub(super) fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, P, C, M> SequenceDecoder<'de> for JsonbSequenceDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeNext<'this>
        = JsonbDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::any()
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if self.cursor.remaining() == 0 {
            return Ok(None);
        }

        Ok(Some(JsonbDecoder::new(self.cx, self.cursor.borrow_mut())))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        if self.cursor.remaining() == 0 {
            return Err(self.cx.message("Encountered short array"));
        }

        Ok(JsonbDecoder::new(self.cx, self.cursor.borrow_mut()))
    }
}
