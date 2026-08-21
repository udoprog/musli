use core::marker::PhantomData;

use crate::Context;
use crate::de::EntryDecoder;

use super::super::cursor::Cursor;
use super::{JsonbDecoder, JsonbKeyDecoder};

/// A decoder for a single JSONB object entry, which is its key immediately
/// followed by its value.
pub(crate) struct JsonbObjectPairDecoder<P, C, M> {
    cx: C,
    cursor: P,
    _marker: PhantomData<M>,
}

impl<P, C, M> JsonbObjectPairDecoder<P, C, M> {
    #[inline]
    pub(super) fn new(cx: C, cursor: P) -> Self {
        Self {
            cx,
            cursor,
            _marker: PhantomData,
        }
    }
}

impl<'de, P, C, M> EntryDecoder<'de> for JsonbObjectPairDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeKey<'this>
        = JsonbKeyDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue = JsonbDecoder<P, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, C::Error> {
        Ok(JsonbKeyDecoder::new(self.cx, self.cursor.borrow_mut()))
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, C::Error> {
        Ok(JsonbDecoder::new(self.cx, self.cursor))
    }
}
