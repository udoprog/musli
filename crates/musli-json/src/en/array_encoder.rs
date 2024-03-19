use musli::en::SequenceEncoder;
use musli::Context;
use musli_common::writer::Writer;

use super::JsonEncoder;

/// Encoder for a JSON array.
pub(crate) struct JsonArrayEncoder<W> {
    first: bool,
    variant: bool,
    writer: W,
}

impl<W> JsonArrayEncoder<W>
where
    W: Writer,
{
    #[inline]
    pub(super) fn new<C>(cx: &C, writer: W) -> Result<Self, C::Error>
    where
        C: ?Sized + Context,
    {
        Self::with_variant(cx, writer, false)
    }

    #[inline]
    pub(super) fn with_variant<C>(cx: &C, mut writer: W, variant: bool) -> Result<Self, C::Error>
    where
        C: ?Sized + Context,
    {
        writer.write_byte(cx, b'[')?;

        Ok(Self {
            first: true,
            variant,
            writer,
        })
    }
}

impl<C: ?Sized + Context, W> SequenceEncoder<C> for JsonArrayEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type EncodeNext<'this> = JsonEncoder<W::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self, cx: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        if !self.first {
            self.writer.write_byte(cx, b',')?;
        }

        self.first = false;
        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, b']')?;

        if self.variant {
            self.writer.write_byte(cx, b'}')?;
        }

        Ok(())
    }
}
