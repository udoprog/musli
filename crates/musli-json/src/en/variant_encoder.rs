use musli::en::VariantEncoder;
use musli::Context;

use crate::writer::Writer;

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// A JSON variant encoder.
pub(crate) struct JsonVariantEncoder<W> {
    writer: W,
}

impl<W> JsonVariantEncoder<W>
where
    W: Writer,
{
    #[inline]
    pub(super) fn new<C>(cx: &C, mut writer: W) -> Result<Self, C::Error>
    where
        C: ?Sized + Context,
    {
        writer.write_byte(cx, b'{')?;
        Ok(Self { writer })
    }
}

impl<C: ?Sized + Context, W> VariantEncoder<C> for JsonVariantEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type EncodeTag<'this> = JsonObjectKeyEncoder<W::Mut<'this>>
    where
        Self: 'this;
    type EncodeValue<'this> = JsonEncoder<W::Mut<'this>>
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
