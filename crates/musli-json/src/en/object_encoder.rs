use musli::en::{MapEncoder, MapEntriesEncoder, StructEncoder};
use musli::Context;

use super::{JsonEncoder, JsonObjectKeyEncoder, JsonObjectPairEncoder};
use crate::writer::Writer;

/// An object encoder for JSON.
pub(crate) struct JsonObjectEncoder<'a, W, C: ?Sized> {
    cx: &'a C,
    len: usize,
    end: &'static [u8],
    writer: W,
}

impl<'a, W, C> JsonObjectEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &'a C, writer: W) -> Result<Self, C::Error> {
        Self::with_end(cx, writer, b"}")
    }

    #[inline]
    pub(super) fn with_end(cx: &'a C, mut writer: W, end: &'static [u8]) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'{')?;

        Ok(Self {
            cx,
            len: 0,
            end,
            writer,
        })
    }
}

impl<'a, W, C> MapEncoder for JsonObjectEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntry<'this> = JsonObjectPairEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_map_entry(&mut self) -> Result<Self::EncodeMapEntry<'_>, C::Error> {
        self.len += 1;

        Ok(JsonObjectPairEncoder::new(
            self.cx,
            self.len == 1,
            self.writer.borrow_mut(),
        ))
    }

    #[inline]
    fn end_map(mut self) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(self.cx, self.end)
    }
}

impl<'a, W, C> MapEntriesEncoder for JsonObjectEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = JsonObjectKeyEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;
    type EncodeMapEntryValue<'this> = JsonEncoder<'a, W::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        if self.len > 0 {
            self.writer.write_byte(self.cx, b',')?;
        }

        self.len += 1;
        Ok(JsonObjectKeyEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_entry_value(&mut self) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        self.writer.write_byte(self.cx, b':')?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end_map_entries(mut self) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, b'}')
    }
}

impl<'a, W, C> StructEncoder for JsonObjectEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeStructField<'this> = JsonObjectPairEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_struct_field(&mut self) -> Result<Self::EncodeStructField<'_>, C::Error> {
        MapEncoder::encode_map_entry(self)
    }

    #[inline]
    fn end_struct(self) -> Result<Self::Ok, C::Error> {
        MapEncoder::end_map(self)
    }
}
