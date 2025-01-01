use core::mem::take;

use crate::en::SequenceEncoder;
use crate::{Context, Writer};

use super::JsonEncoder;

/// Encoder for a JSON array.
pub(crate) struct JsonArrayEncoder<W, C> {
    cx: C,
    first: bool,
    end: &'static [u8],
    writer: W,
}

impl<W, C> JsonArrayEncoder<W, C>
where
    W: Writer,
    C: Context,
{
    #[inline]
    pub(super) fn new(cx: C, writer: W) -> Result<Self, C::Error> {
        Self::with_end(cx, writer, b"]")
    }

    #[inline]
    pub(super) fn with_end(cx: C, mut writer: W, end: &'static [u8]) -> Result<Self, C::Error> {
        writer.write_byte(cx, b'[')?;

        Ok(Self {
            cx,
            first: true,
            end,
            writer,
        })
    }
}

impl<W, C> SequenceEncoder for JsonArrayEncoder<W, C>
where
    W: Writer,
    C: Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this>
        = JsonEncoder<W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        if !take(&mut self.first) {
            self.writer.write_byte(self.cx, b',')?;
        }

        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(mut self) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(self.cx, self.end)
    }
}
