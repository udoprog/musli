use core::fmt;

use crate::en::{Encode, Encoder};
use crate::{Context, Writer};

pub(crate) struct JsonObjectKeyEncoder<'a, W, C: ?Sized> {
    cx: &'a C,
    writer: W,
}

impl<'a, W, C: ?Sized> JsonObjectKeyEncoder<'a, W, C> {
    #[inline]
    pub(super) fn new(cx: &'a C, writer: W) -> Self {
        Self { cx, writer }
    }
}

macro_rules! format_integer {
    ($slf:ident, $value:ident) => {{
        $slf.writer.write_byte($slf.cx, b'"')?;
        let mut buffer = itoa::Buffer::new();
        $slf.writer
            .write_bytes($slf.cx, buffer.format($value).as_bytes())?;
        $slf.writer.write_byte($slf.cx, b'"')?;
        Ok(())
    }};
}

#[crate::encoder(crate)]
impl<W, C> Encoder for JsonObjectKeyEncoder<'_, W, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U>
        = JsonObjectKeyEncoder<'this, W, U>
    where
        U: 'this + Context;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(JsonObjectKeyEncoder::new(cx, self.writer))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any type that can be used as an object key")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<Self::Ok, Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.as_encode().encode(self.cx, self)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, C::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_string(self, string: &str) -> Result<Self::Ok, C::Error> {
        super::encode_string(self.cx, self.writer, string.as_bytes())
    }

    #[inline]
    fn collect_string<T>(self, value: &T) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let buf = self.cx.collect_string(value)?;
        self.encode_string(buf.as_ref())
    }
}
