use core::fmt;
use core::marker::PhantomData;

use crate::en::{Encode, Encoder};
use crate::{Context, Writer};

use super::super::tag::TEXT;
use super::{encode_string, encode_text};

/// Encoder for the key of a JSONB object entry.
///
/// The key of an object entry has to be one of the string element types, so
/// anything which is not already a string is rendered as one.
pub(crate) struct JsonbObjectKeyEncoder<W, C, M> {
    cx: C,
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonbObjectKeyEncoder<W, C, M> {
    #[inline]
    pub(super) fn new(cx: C, writer: W) -> Self {
        Self {
            cx,
            writer,
            _marker: PhantomData,
        }
    }
}

macro_rules! format_integer {
    ($slf:ident, $value:ident) => {{
        let mut buffer = itoa::Buffer::new();
        encode_text($slf.cx, &mut $slf.writer, TEXT, buffer.format($value))
    }};
}

#[crate::trait_defaults(crate)]
impl<W, C, M> Encoder for JsonbObjectKeyEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any type that can be used as an object key")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.as_encode().encode(self)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<(), Self::Error> {
        format_integer!(self, value)
    }

    #[inline]
    fn encode_char(mut self, value: char) -> Result<(), Self::Error> {
        encode_string(
            self.cx,
            &mut self.writer,
            value.encode_utf8(&mut [0, 0, 0, 0]),
        )
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<(), Self::Error> {
        encode_string(self.cx, &mut self.writer, string)
    }
}
