use crate::en::EntryEncoder;
use crate::{Context, Writer};

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// Encoder for a JSON object pair.
pub(crate) struct JsonObjectPairEncoder<W, C> {
    cx: C,
    empty: bool,
    writer: W,
}

impl<W, C> JsonObjectPairEncoder<W, C> {
    #[inline]
    pub(super) const fn new(cx: C, empty: bool, writer: W) -> Self {
        Self { cx, empty, writer }
    }
}

impl<W, C> EntryEncoder for JsonObjectPairEncoder<W, C>
where
    W: Writer,
    C: Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeKey<'this>
        = JsonObjectKeyEncoder<W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeValue<'this>
        = JsonEncoder<W::Mut<'this>, C>
    where
        Self: 'this;

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
