use core::fmt;

use musli::de::{Decoder, NumberHint, SizeHint, TypeHint, ValueVisitor, Visitor};
use musli::Context;

use crate::reader::{Parser, Token};

use super::{
    JsonDecoder, JsonObjectDecoder, KeySignedVisitor, KeyUnsignedVisitor, StringReference,
};

/// A JSON object key decoder for Müsli.
pub(crate) struct JsonKeyDecoder<P> {
    parser: P,
}

impl<'de, P> JsonKeyDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    pub(super) fn skip_any<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        JsonDecoder::new(self.parser).skip_any(cx)
    }
}

impl<'de, P> JsonKeyDecoder<P>
where
    P: Parser<'de>,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }

    #[inline]
    fn decode_escaped_bytes<C, V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: ?Sized + Context,
        V: ValueVisitor<'de, C, [u8]>,
    {
        let Some(mut scratch) = cx.alloc() else {
            return Err(cx.message("Failed to allocate scratch buffer"));
        };

        match self.parser.parse_string(cx, true, &mut scratch)? {
            StringReference::Borrowed(string) => visitor.visit_borrowed(cx, string.as_bytes()),
            StringReference::Scratch(string) => visitor.visit_ref(cx, string.as_bytes()),
        }
    }
}

#[musli::decoder]
impl<'de, C, P> Decoder<'de, C> for JsonKeyDecoder<P>
where
    C: ?Sized + Context,
    P: Parser<'de>,
{
    type Decoder<U> = Self where U: Context;
    type Struct = JsonObjectDecoder<P>;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::Decoder<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a object key")
    }

    #[inline]
    fn type_hint(&mut self, cx: &C) -> Result<TypeHint, C::Error> {
        JsonDecoder::new(&mut self.parser).type_hint(cx)
    }

    #[inline]
    fn decode_u8(self, cx: &C) -> Result<u8, C::Error> {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u16(self, cx: &C) -> Result<u16, C::Error> {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u64(self, cx: &C) -> Result<u64, C::Error> {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u128(self, cx: &C) -> Result<u128, C::Error> {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_i8(self, cx: &C) -> Result<i8, C::Error> {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i16(self, cx: &C) -> Result<i16, C::Error> {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i32(self, cx: &C) -> Result<i32, C::Error> {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i64(self, cx: &C) -> Result<i64, C::Error> {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i128(self, cx: &C) -> Result<i128, C::Error> {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_usize(self, cx: &C) -> Result<usize, C::Error> {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_isize(self, cx: &C) -> Result<isize, C::Error> {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_string<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
    {
        JsonDecoder::new(self.parser).decode_string(cx, visitor)
    }

    #[inline]
    fn decode_any<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, C>,
    {
        match self.parser.peek(cx)? {
            Token::String => {
                let visitor = visitor.visit_string(cx, SizeHint::Any)?;
                self.decode_string(cx, visitor)
            }
            Token::Number => {
                let visitor = visitor.visit_number(cx, NumberHint::Any)?;
                self.decode_number(cx, visitor)
            }
            _ => visitor.visit_any(cx, self, TypeHint::Any),
        }
    }
}
