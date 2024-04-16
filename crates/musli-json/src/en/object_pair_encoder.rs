use musli::en::MapEntryEncoder;
use musli::Context;
use musli_utils::Writer;

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

impl<'a, W, C> MapEntryEncoder for JsonObjectPairEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = JsonObjectKeyEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeMapValue<'this> = JsonEncoder<'a, W::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        if !self.empty {
            self.writer.write_byte(self.cx, b',')?;
        }

        Ok(JsonObjectKeyEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        self.writer.write_byte(self.cx, b':')?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map_entry(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}
