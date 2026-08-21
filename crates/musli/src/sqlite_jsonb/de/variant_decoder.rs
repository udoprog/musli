use core::marker::PhantomData;

use crate::Context;
use crate::de::VariantDecoder;

use super::super::cursor::Cursor;
use super::{JsonbDecoder, JsonbKeyDecoder};

/// A decoder for an externally tagged variant, which is encoded as an object
/// with a single entry.
pub(crate) struct JsonbVariantDecoder<P, C, M> {
    cx: C,
    cursor: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonbVariantDecoder<P, C, M>
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

    #[inline]
    pub(super) fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, P, C, M> VariantDecoder<'de> for JsonbVariantDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeTag<'this>
        = JsonbKeyDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = JsonbDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(JsonbKeyDecoder::new(self.cx, self.cursor.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        Ok(JsonbDecoder::new(self.cx, self.cursor.borrow_mut()))
    }
}
