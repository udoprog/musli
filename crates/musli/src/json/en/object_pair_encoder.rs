use crate::Writer;
use musli_core::en::EntryEncoder;
use musli_core::Context;

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// Encoder for a JSON object pair.
pub(crate) struct JsonObjectPairEncoder<'a, W, C: ?Sized> {
    cx: &'a C,
    empty: bool,
    writer: W,
}

impl<'a, W, C: ?Sized> JsonObjectPairEncoder<'a, W, C> {
    #[inline]
    pub(super) const fn new(cx: &'a C, empty: bool, writer: W) -> Self {
        Self { cx, empty, writer }
    }
}

impl<'a, W, C> EntryEncoder for JsonObjectPairEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeKey<'this> = JsonObjectKeyEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeValue<'this> = JsonEncoder<'a, W::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, C::Error> {
        if !self.empty {
            self.writer.write_byte(self.cx, b',')?;
        }

        Ok(JsonObjectKeyEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        self.writer.write_byte(self.cx, b':')?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}
