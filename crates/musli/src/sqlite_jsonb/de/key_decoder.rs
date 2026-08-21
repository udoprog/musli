use core::fmt;
use core::marker::PhantomData;

use crate::Context;
use crate::de::{Decoder, SizeHint, Skip, UnsizedVisitor, Visitor};

use super::super::cursor::Cursor;
use super::super::parse::Integer;
use super::super::tag::is_text;
use super::{JsonbDecoder, decode_key_integer};

/// A decoder for the key of a JSONB object entry.
///
/// Object keys are always one of the string element types, so anything which is
/// not a string is decoded from the text the key contains.
pub(crate) struct JsonbKeyDecoder<P, C, M> {
    cx: C,
    cursor: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonbKeyDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, cursor: P) -> Self {
        Self {
            cx,
            cursor,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn decode_integer<T>(mut self) -> Result<T, C::Error>
    where
        T: Integer,
    {
        decode_key_integer(self.cx, &mut self.cursor)
    }
}

#[crate::trait_defaults(crate)]
impl<'de, P, C, M> Decoder<'de> for JsonbKeyDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = JsonbKeyDecoder<P::TryClone, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from an object key")
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(JsonbKeyDecoder::new(self.cx, self.cursor.try_clone()?))
    }

    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        JsonbDecoder::<_, _, M>::new(self.cx, self.cursor).skip()
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        self.skip()?;
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        JsonbDecoder::<_, _, M>::new(self.cx, self.cursor).decode_char()
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        JsonbDecoder::<_, _, M>::new(self.cx, self.cursor).decode_string(visitor)
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;

        let Some(kind) = self.cursor.peek().map(|b| b & 0x0f) else {
            return Err(cx.message("Expected object key in input"));
        };

        if !is_text(kind) {
            return Err(cx.message(format_args!(
                "Expected object key, but was {}",
                super::super::tag::Kind(kind)
            )));
        }

        let visitor = visitor.visit_string(cx, SizeHint::any())?;
        self.decode_string(visitor)
    }
}
