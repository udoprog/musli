use core::marker::PhantomData;

use musli::en::{MapEntryEncoder, StructFieldEncoder};
use musli::Context;

use crate::writer::Writer;

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// Encoder for a JSON object pair.
pub(crate) struct JsonObjectPairEncoder<W, C: ?Sized> {
    empty: bool,
    writer: W,
    _marker: PhantomData<C>,
}

impl<W, C: ?Sized> JsonObjectPairEncoder<W, C> {
    #[inline]
    pub(super) const fn new(empty: bool, writer: W) -> Self {
        Self {
            empty,
            writer,
            _marker: PhantomData,
        }
    }
}

impl<W, C> MapEntryEncoder for JsonObjectPairEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = JsonObjectKeyEncoder<W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeMapValue<'this> = JsonEncoder<W::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self, cx: &C) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        if !self.empty {
            self.writer.write_byte(cx, b',')?;
        }

        Ok(JsonObjectKeyEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_value(&mut self, cx: &C) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        self.writer.write_byte(cx, b':')?;
        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, C> StructFieldEncoder for JsonObjectPairEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeFieldName<'this> = JsonObjectKeyEncoder<W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeFieldValue<'this> = JsonEncoder<W::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn encode_field_name(&mut self, cx: &C) -> Result<Self::EncodeFieldName<'_>, C::Error> {
        self.encode_map_key(cx)
    }

    #[inline]
    fn encode_field_value(&mut self, cx: &C) -> Result<Self::EncodeFieldValue<'_>, C::Error> {
        self.encode_map_value(cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error> {
        MapEntryEncoder::end(self, cx)
    }
}
