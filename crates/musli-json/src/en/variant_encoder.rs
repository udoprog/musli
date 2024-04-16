use musli::en::VariantEncoder;
use musli::Context;
use musli_utils::Writer;

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// A JSON variant encoder.
pub(crate) struct JsonVariantEncoder<'a, W, C: ?Sized> {
    cx: &'a C,
    writer: W,
}

impl<'a, W, C> JsonVariantEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &'a C, mut writer: W) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'{')?;
        Ok(Self { cx, writer })
    }
}

impl<'a, W, C> VariantEncoder for JsonVariantEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = JsonObjectKeyEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeData<'this> = JsonEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;

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
    fn finish_variant(mut self) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, b'}')
    }
}
