use core::mem::take;

use crate::en::SequenceEncoder;
use crate::{Context, Writer};

use super::JsonEncoder;

/// Encoder for a JSON array.
pub(crate) struct JsonArrayEncoder<'a, W, C: ?Sized> {
    cx: &'a C,
    first: bool,
    end: &'static [u8],
    writer: W,
}

impl<'a, W, C> JsonArrayEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    #[inline(always)]
    pub(super) fn new(cx: &'a C, writer: W) -> Result<Self, C::Error> {
        Self::with_end(cx, writer, b"]")
    }

    #[inline(always)]
    pub(super) fn with_end(cx: &'a C, mut writer: W, end: &'static [u8]) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'[')?;

        Ok(Self {
            cx,
            first: true,
            end,
            writer,
        })
    }
}

impl<'a, W, C> SequenceEncoder for JsonArrayEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this>
        = JsonEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline(always)]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        if !take(&mut self.first) {
            self.writer.write_byte(self.cx, b',')?;
        }

        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline(always)]
    fn finish_sequence(mut self) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(self.cx, self.end)
    }
}
