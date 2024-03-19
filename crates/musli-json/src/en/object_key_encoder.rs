use core::fmt;

use musli::en::Encoder;
use musli::Context;
use musli_common::writer::Writer;

pub(crate) struct JsonObjectKeyEncoder<W> {
    writer: W,
}

impl<W> JsonObjectKeyEncoder<W> {
    #[inline]
    pub(super) fn new(writer: W) -> Self {
        Self { writer }
    }
}

macro_rules! format_integer {
    ($slf:ident, $cx:expr, $value:ident) => {{
        $slf.writer.write_byte($cx, b'"')?;
        let mut buffer = itoa::Buffer::new();
        $slf.writer
            .write_bytes($cx, buffer.format($value).as_bytes())?;
        $slf.writer.write_byte($cx, b'"')?;
        Ok(())
    }};
}

#[musli::encoder]
impl<C: ?Sized + Context, W> Encoder<C> for JsonObjectKeyEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type WithContext<U> = Self where U: Context;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any type that can be used as an object key")
    }

    #[inline]
    fn encode_u8(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u16(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u32(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u64(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u128(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i8(mut self, cx: &C, value: i8) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i16(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i32(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i64(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i128(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_usize(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_isize(mut self, cx: &C, value: isize) -> Result<Self::Ok, C::Error> {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_string(self, cx: &C, string: &str) -> Result<Self::Ok, C::Error> {
        super::encode_string(cx, self.writer, string.as_bytes())
    }
}
