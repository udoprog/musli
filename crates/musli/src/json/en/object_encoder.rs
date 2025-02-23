use core::marker::PhantomData;

use crate::en::{EntriesEncoder, MapEncoder};
use crate::{Context, Writer};

use super::{JsonEncoder, JsonObjectKeyEncoder, JsonObjectPairEncoder};

/// An object encoder for JSON.
pub(crate) struct JsonObjectEncoder<W, C, M> {
    cx: C,
    len: usize,
    end: &'static [u8],
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, writer: W) -> Result<Self, C::Error> {
        Self::with_end(cx, writer, b"}")
    }

    #[inline]
    pub(super) fn with_end(cx: C, mut writer: W, end: &'static [u8]) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'{')?;

        Ok(Self {
            cx,
            len: 0,
            end,
            writer,
            _marker: PhantomData,
        })
    }
}

impl<W, C, M> MapEncoder for JsonObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = JsonObjectPairEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        self.len += 1;

        Ok(JsonObjectPairEncoder::new(
            self.cx,
            self.len == 1,
            self.writer.borrow_mut(),
        ))
    }

    #[inline]
    fn finish_map(mut self) -> Result<(), Self::Error> {
        self.writer.write_bytes(self.cx, self.end)
    }
}

impl<W, C, M> EntriesEncoder for JsonObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = JsonObjectKeyEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = JsonEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        if self.len > 0 {
            self.writer.write_byte(self.cx, b',')?;
        }

        self.len += 1;
        Ok(JsonObjectKeyEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        self.writer.write_byte(self.cx, b':')?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entries(mut self) -> Result<(), Self::Error> {
        self.writer.write_byte(self.cx, b'}')
    }
}
