use core::marker::PhantomData;

use musli::en::{MapEncoder, MapEntriesEncoder, StructEncoder};
use musli::Context;

use super::{JsonEncoder, JsonObjectKeyEncoder, JsonObjectPairEncoder};
use crate::writer::Writer;

/// An object encoder for JSON.
pub(crate) struct JsonObjectEncoder<W, C: ?Sized> {
    len: usize,
    end: &'static [u8],
    writer: W,
    _marker: PhantomData<C>,
}

impl<W, C> JsonObjectEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &C, writer: W) -> Result<Self, C::Error> {
        Self::with_end(cx, writer, b"}")
    }

    #[inline]
    pub(super) fn with_end(cx: &C, mut writer: W, end: &'static [u8]) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'{')?;

        Ok(Self {
            len: 0,
            end,
            writer,
            _marker: PhantomData,
        })
    }
}

impl<W, C> MapEncoder for JsonObjectEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this> = JsonObjectPairEncoder<W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_entry(&mut self, _: &C) -> Result<Self::EncodeEntry<'_>, C::Error> {
        self.len += 1;

        Ok(JsonObjectPairEncoder::new(
            self.len == 1,
            self.writer.borrow_mut(),
        ))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(cx, self.end)
    }
}

impl<W, C> MapEntriesEncoder for JsonObjectEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = JsonObjectKeyEncoder<W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeMapEntryValue<'this> = JsonEncoder<W::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self, cx: &C) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        if self.len > 0 {
            self.writer.write_byte(cx, b',')?;
        }

        self.len += 1;
        Ok(JsonObjectKeyEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_entry_value(
        &mut self,
        cx: &C,
    ) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        self.writer.write_byte(cx, b':')?;
        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, b'}')
    }
}

impl<W, C> StructEncoder for JsonObjectEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeField<'this> = JsonObjectPairEncoder<W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_field(&mut self, cx: &C) -> Result<Self::EncodeField<'_>, C::Error> {
        MapEncoder::encode_entry(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error> {
        MapEncoder::end(self, cx)
    }
}
