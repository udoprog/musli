use core::marker::PhantomData;

use musli::en::VariantEncoder;
use musli::Context;

use crate::writer::Writer;

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// A JSON variant encoder.
pub(crate) struct JsonVariantEncoder<W, C: ?Sized> {
    writer: W,
    _marker: PhantomData<C>,
}

impl<W, C> JsonVariantEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &C, mut writer: W) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'{')?;
        Ok(Self {
            writer,
            _marker: PhantomData,
        })
    }
}

impl<W, C> VariantEncoder for JsonVariantEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = JsonObjectKeyEncoder<W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeValue<'this> = JsonEncoder<W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_tag(&mut self, _: &C) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(JsonObjectKeyEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self, cx: &C) -> Result<Self::EncodeValue<'_>, C::Error> {
        self.writer.write_byte(cx, b':')?;
        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, b'}')
    }
}
