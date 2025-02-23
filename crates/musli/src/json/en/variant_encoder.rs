use core::marker::PhantomData;

use crate::en::VariantEncoder;
use crate::{Context, Writer};

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// A JSON variant encoder.
pub(crate) struct JsonVariantEncoder<W, C, M> {
    cx: C,
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonVariantEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, mut writer: W) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'{')?;

        Ok(Self {
            cx,
            writer,
            _marker: PhantomData,
        })
    }
}

impl<W, C, M> VariantEncoder for JsonVariantEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeTag<'this>
        = JsonObjectKeyEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeData<'this>
        = JsonEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(JsonObjectKeyEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, C::Error> {
        self.writer.write_byte(self.cx, b':')?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_variant(mut self) -> Result<(), C::Error> {
        self.writer.write_byte(self.cx, b'}')
    }
}
