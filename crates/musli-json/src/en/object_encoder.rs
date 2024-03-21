use musli::en::{MapEncoder, MapEntriesEncoder, StructEncoder};
use musli::Context;

use super::{JsonEncoder, JsonObjectKeyEncoder, JsonObjectPairEncoder};
use crate::writer::Writer;

/// An object encoder for JSON.
pub(crate) struct JsonObjectEncoder<W> {
    len: usize,
    end: &'static [u8],
    writer: W,
}

impl<W> JsonObjectEncoder<W>
where
    W: Writer,
{
    #[inline]
    pub(super) fn new<C>(cx: &C, writer: W) -> Result<Self, C::Error>
    where
        C: ?Sized + Context,
    {
        Self::with_end(cx, writer, b"}")
    }

    #[inline]
    pub(super) fn with_end<C>(cx: &C, mut writer: W, end: &'static [u8]) -> Result<Self, C::Error>
    where
        C: ?Sized + Context,
    {
        writer.write_byte(cx, b'{')?;

        Ok(Self {
            len: 0,
            end,
            writer,
        })
    }
}

impl<C: ?Sized + Context, W> MapEncoder<C> for JsonObjectEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type EncodeEntry<'this> = JsonObjectPairEncoder<W::Mut<'this>>
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

impl<C: ?Sized + Context, W> MapEntriesEncoder<C> for JsonObjectEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type EncodeMapEntryKey<'this> = JsonObjectKeyEncoder<W::Mut<'this>>
    where
        Self: 'this;
    type EncodeMapEntryValue<'this> = JsonEncoder<W::Mut<'this>> where Self: 'this;

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

impl<C: ?Sized + Context, W> StructEncoder<C> for JsonObjectEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type EncodeField<'this> = JsonObjectPairEncoder<W::Mut<'this>>
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
