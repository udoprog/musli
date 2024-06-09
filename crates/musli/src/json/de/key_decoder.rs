use core::fmt;

use crate::alloc::Vec;
use crate::de::{Decode, DecodeUnsized, Decoder, SizeHint, Skip, UnsizedVisitor, Visitor};
use crate::Context;

use super::super::parser::{Parser, Token};
use super::{JsonDecoder, KeySignedVisitor, KeyUnsignedVisitor, StringReference};

/// A JSON object key decoder for Müsli.
pub(crate) struct JsonKeyDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    parser: P,
}

impl<'a, 'de, P, C> JsonKeyDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, parser: P) -> Self {
        Self { cx, parser }
    }

    #[inline]
    fn decode_escaped_bytes<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: UnsizedVisitor<'de, C, [u8]>,
    {
        let mut scratch = Vec::new_in(self.cx.alloc());

        match self.parser.parse_string(self.cx, true, &mut scratch)? {
            StringReference::Borrowed(string) => visitor.visit_borrowed(self.cx, string.as_bytes()),
            StringReference::Scratch(string) => visitor.visit_ref(self.cx, string.as_bytes()),
        }
    }
}

#[crate::decoder(crate)]
impl<'a, 'de, P, C> Decoder<'de> for JsonKeyDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<'this, U> = JsonKeyDecoder<'this, P, U> where U: 'this + Context;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(JsonKeyDecoder::new(cx, self.parser))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a object key")
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode>,
    {
        self.cx.decode(self)
    }

    #[inline]
    fn decode_unsized<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsized<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        self.cx.decode_unsized(self, f)
    }

    #[inline]
    fn skip(self) -> Result<(), C::Error> {
        JsonDecoder::new(self.cx, self.parser).skip()
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, C::Error> {
        self.skip()?;
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, C::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, C::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, C::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, C::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, C::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, C::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, C::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, C::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, C::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, C::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, C::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, C::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: UnsizedVisitor<'de, C, str>,
    {
        JsonDecoder::new(self.cx, self.parser).decode_string(visitor)
    }

    #[inline]
    fn decode_any<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, C>,
    {
        match self.parser.lex(self.cx) {
            Token::String => {
                let visitor = visitor.visit_string(self.cx, SizeHint::any())?;
                self.decode_string(visitor)
            }
            Token::Number => self.decode_number(visitor),
            token => Err(self
                .cx
                .message(format_args!("Unsupported key type {token:?}"))),
        }
    }
}
