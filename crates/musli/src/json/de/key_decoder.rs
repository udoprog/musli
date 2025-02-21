use core::fmt;
use core::marker::PhantomData;

use crate::alloc::Vec;
use crate::de::{Decoder, SizeHint, Skip, UnsizedVisitor, Visitor};
use crate::Context;

use super::super::parser::{Parser, Token};
use super::{JsonDecoder, KeySignedVisitor, KeyUnsignedVisitor, StringReference};

/// A JSON object key decoder for Müsli.
pub(crate) struct JsonKeyDecoder<P, C, M> {
    cx: C,
    parser: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonKeyDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: C, parser: P) -> Self {
        Self {
            cx,
            parser,
            _marker: PhantomData,
        }
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
impl<'de, P, C, M> Decoder<'de> for JsonKeyDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type Allocator = C::Allocator;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a object key")
    }

    #[inline]
    fn skip(self) -> Result<(), C::Error> {
        JsonDecoder::<_, _, M>::new(self.cx, self.parser).skip()
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
        JsonDecoder::<_, _, M>::new(self.cx, self.parser).decode_string(visitor)
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
