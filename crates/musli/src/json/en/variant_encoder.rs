use crate::en::VariantEncoder;
use crate::{Context, Writer};

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// A JSON variant encoder.
pub(crate) struct JsonVariantEncoder<W, C> {
    cx: C,
    writer: W,
}

impl<W, C> JsonVariantEncoder<W, C>
where
    W: Writer,
    C: Context,
{
    #[inline]
    pub(super) fn new(cx: C, mut writer: W) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'{')?;
        Ok(Self { cx, writer })
    }
}

impl<W, C> VariantEncoder for JsonVariantEncoder<W, C>
where
    W: Writer,
    C: Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this>
        = JsonObjectKeyEncoder<W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeData<'this>
        = JsonEncoder<W::Mut<'this>, C>
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
