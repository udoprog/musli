use core::mem::take;

use musli::en::{PackEncoder, SequenceEncoder};
use musli::Context;
use musli_utils::Writer;

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
    #[inline]
    pub(super) fn new(cx: &'a C, writer: W) -> Result<Self, C::Error> {
        Self::with_end(cx, writer, b"]")
    }

    #[inline]
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
    type EncodeElement<'this> = JsonEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_element(&mut self) -> Result<Self::EncodeElement<'_>, C::Error> {
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

impl<'a, W, C> PackEncoder for JsonArrayEncoder<'a, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodePacked<'this> = JsonEncoder<'a, W::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_packed(&mut self) -> Result<Self::EncodePacked<'_>, C::Error> {
        SequenceEncoder::encode_element(self)
    }

    #[inline]
    fn finish_pack(self) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::finish_sequence(self)
    }
}
