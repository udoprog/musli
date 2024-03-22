use core::marker::PhantomData;
use core::mem::take;

use musli::en::SequenceEncoder;
use musli::Context;

use crate::writer::Writer;

use super::JsonEncoder;

/// Encoder for a JSON array.
pub(crate) struct JsonArrayEncoder<W, C: ?Sized> {
    first: bool,
    end: &'static [u8],
    writer: W,
    _marker: PhantomData<C>,
}

impl<W, C> JsonArrayEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &C, writer: W) -> Result<Self, C::Error> {
        Self::with_end(cx, writer, b"]")
    }

    #[inline]
    pub(super) fn with_end(cx: &C, mut writer: W, end: &'static [u8]) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'[')?;

        Ok(Self {
            first: true,
            end,
            writer,
            _marker: PhantomData,
        })
    }
}

impl<W, C> SequenceEncoder for JsonArrayEncoder<W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this> = JsonEncoder<W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self, cx: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        if !take(&mut self.first) {
            self.writer.write_byte(cx, b',')?;
        }

        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(cx, self.end)
    }
}
