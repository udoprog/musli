use core::fmt;
use core::marker;
use core::mem;
use core::str;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, NumberHint, NumberVisitor, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder,
    SizeHint, TypeHint, ValueVisitor, VariantDecoder, Visitor,
};
#[cfg(feature = "musli-value")]
use musli::mode::Mode;
use musli::Context;

use crate::error::{Error, ErrorKind};
#[cfg(not(feature = "parse-full"))]
use crate::reader::integer::{
    parse_signed_base as parse_signed, parse_unsigned_base as parse_unsigned,
};
#[cfg(feature = "parse-full")]
use crate::reader::integer::{
    parse_signed_full as parse_signed, parse_unsigned_full as parse_unsigned,
};
use crate::reader::integer::{Signed, Unsigned};
use crate::reader::SliceParser;
use crate::reader::{integer, string, Parser, StringReference, Token};

use musli_common::options;

const BUFFER_OPTIONS: options::Options = options::new().build();

/// A JSON decoder for Müsli.
pub struct JsonDecoder<P> {
    parser: P,
}

impl<'de, P> JsonDecoder<P>
where
    P: Parser<'de>,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }

    /// Skip over any values.
    pub(crate) fn skip_any<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        let start = cx.mark();
        let actual = self.parser.peek(cx)?;

        match actual {
            Token::OpenBrace => {
                let mut object = JsonObjectDecoder::new(cx, None, self.parser)?;

                while let Some(mut pair) = object.next(cx)? {
                    pair.first(cx)?.skip_any(cx)?;
                    pair.skip_second(cx)?;
                }

                Ok(())
            }
            Token::OpenBracket => {
                let mut seq = JsonSequenceDecoder::new(cx, None, self.parser)?;

                while let Some(item) = SequenceDecoder::next(&mut seq, cx)? {
                    item.skip_any(cx)?;
                }

                Ok(())
            }
            Token::Null => self.parse_null(cx),
            Token::True => self.parse_true(cx),
            Token::False => self.parse_false(cx),
            Token::Number => integer::skip_number(cx, &mut self.parser),
            Token::String => {
                // Skip over opening quote.
                self.parser.skip(cx, 1)?;
                string::skip_string(cx, &mut self.parser, true)
            }
            actual => Err(cx.marked_report(start, Error::new(ErrorKind::ExpectedValue(actual)))),
        }
    }

    #[inline]
    fn parse_true<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        self.parser
            .parse_exact(cx, *b"true", Error::new(ErrorKind::ExpectedTrue))
    }

    #[inline]
    fn parse_false<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        self.parser
            .parse_exact(cx, *b"false", Error::new(ErrorKind::ExpectedFalse))
    }

    #[inline]
    fn parse_null<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        self.parser
            .parse_exact(cx, *b"null", Error::new(ErrorKind::ExpectedNull))
    }
}

#[musli::decoder]
impl<'de, P> Decoder<'de> for JsonDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;
    #[cfg(feature = "musli-value")]
    type Buffer = musli_value::AsValueDecoder<BUFFER_OPTIONS, Self::Error>;
    type Pack = JsonSequenceDecoder<P>;
    type Sequence = JsonSequenceDecoder<P>;
    type Tuple = JsonSequenceDecoder<P>;
    type Map = JsonObjectDecoder<P>;
    type Some = JsonDecoder<P>;
    type Struct = JsonObjectDecoder<P>;
    type Variant = JsonVariantDecoder<P>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSON")
    }

    #[inline]
    fn type_hint<C>(&mut self, cx: &mut C) -> Result<TypeHint, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(match self.parser.peek(cx)? {
            Token::OpenBrace => TypeHint::Map(SizeHint::Any),
            Token::OpenBracket => TypeHint::Sequence(SizeHint::Any),
            Token::String => TypeHint::String(SizeHint::Any),
            Token::Number => TypeHint::Number(NumberHint::Any),
            Token::Null => TypeHint::Unit,
            Token::True => TypeHint::Bool,
            Token::False => TypeHint::Bool,
            _ => TypeHint::Any,
        })
    }

    #[cfg(feature = "musli-value")]
    #[inline]
    fn decode_buffer<M, C>(self, cx: &mut C) -> Result<Self::Buffer, C::Error>
    where
        M: Mode,
        C: Context<Input = Self::Error>,
    {
        use musli::de::Decode;
        let value: musli_value::Value = Decode::<M>::decode(cx, self)?;
        Ok(value.into_value_decoder())
    }

    #[inline]
    fn decode_unit<C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)
    }

    #[inline]
    fn decode_bool<C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self.parser.peek(cx)? {
            Token::True => {
                self.parse_true(cx)?;
                Ok(true)
            }
            Token::False => {
                self.parse_false(cx)?;
                Ok(false)
            }
            actual => Err(cx.report(Error::new(ErrorKind::ExpectedBool(actual)))),
        }
    }

    #[inline]
    fn decode_char<C>(mut self, cx: &mut C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let start = cx.mark();
        let mut scratch = cx.alloc();

        let string = match self.parser.parse_string(cx, true, &mut scratch)? {
            StringReference::Borrowed(string) => string,
            StringReference::Scratch(string) => string,
        };

        let mut it = string.chars();
        let first = it.next();

        match (first, it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(cx.marked_report(start, Error::new(ErrorKind::CharEmptyString))),
        }
    }

    #[inline]
    fn decode_u8<C>(mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u16<C>(mut self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u32<C>(mut self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u64<C>(mut self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u128<C>(mut self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i8<C>(mut self, cx: &mut C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i16<C>(mut self, cx: &mut C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i32<C>(mut self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i64<C>(mut self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i128<C>(mut self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_usize<C>(mut self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_isize<C>(mut self, cx: &mut C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_f32<C>(mut self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.parser.parse_f32(cx)
    }

    #[inline]
    fn decode_f64<C>(mut self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.parser.parse_f64(cx)
    }

    #[inline]
    fn decode_array<C, const N: usize>(self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let mut seq = self.decode_sequence(cx)?;
        let mut bytes = [0; N];
        let mut index = 0;

        while let Some(item) = SequenceDecoder::next(&mut seq, cx)? {
            if index == N {
                return Err(cx.message(format_args!(
                    "Overflowed array at {index} elements, expected {N}"
                )));
            }

            bytes[index] = item.decode_u8(cx)?;
            index += 1;
        }

        Ok(bytes)
    }

    #[inline]
    fn decode_number<C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: NumberVisitor<'de, C>,
    {
        self.parser.parse_number(cx, visitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<C, V>(self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, [u8]>,
    {
        let mut seq = self.decode_sequence(cx)?;
        let mut bytes = Vec::with_capacity(seq.size_hint().or_default());

        while let Some(item) = SequenceDecoder::next(&mut seq, cx)? {
            bytes.push(item.decode_u8(cx)?);
        }

        visitor.visit_owned(cx, bytes)
    }

    #[inline]
    fn decode_string<C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, str>,
    {
        let mut scratch = cx.alloc();

        match self.parser.parse_string(cx, true, &mut scratch)? {
            StringReference::Borrowed(borrowed) => visitor.visit_borrowed(cx, borrowed),
            StringReference::Scratch(string) => visitor.visit_ref(cx, string),
        }
    }

    #[inline]
    fn decode_option<C>(mut self, cx: &mut C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<Input = Error>,
    {
        if self.parser.peek(cx)?.is_null() {
            self.parse_null(cx)?;
            Ok(None)
        } else {
            Ok(Some(self))
        }
    }

    #[inline]
    fn decode_pack<C>(self, cx: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_sequence<C>(self, cx: &mut C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_tuple<C>(self, cx: &mut C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, Some(len), self.parser)
    }

    #[inline]
    fn decode_map<C>(self, cx: &mut C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_struct<C>(self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, Some(len), self.parser)
    }

    #[inline]
    fn decode_variant<C>(self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonVariantDecoder::new(cx, self.parser)
    }

    #[inline]
    fn decode_any<C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Error>,
        V: Visitor<'de, Error = Self::Error>,
    {
        self.parser.skip_whitespace(cx)?;

        match self.parser.peek(cx)? {
            Token::OpenBrace => {
                let decoder = JsonObjectDecoder::new(cx, None, self.parser)?;
                visitor.visit_map(cx, decoder)
            }
            Token::OpenBracket => {
                let decoder = JsonSequenceDecoder::new(cx, None, self.parser)?;
                visitor.visit_sequence(cx, decoder)
            }
            Token::String => {
                let visitor = visitor.visit_string(cx, SizeHint::Any)?;
                self.decode_string(cx, visitor)
            }
            Token::Number => {
                let visitor = visitor.visit_number(cx, NumberHint::Any)?;
                self.decode_number(cx, visitor)
            }
            Token::Null => {
                self.parse_null(cx)?;
                visitor.visit_unit(cx)
            }
            Token::True => {
                self.parse_true(cx)?;
                visitor.visit_bool(cx, true)
            }
            Token::False => {
                self.parse_false(cx)?;
                visitor.visit_bool(cx, false)
            }
            _ => visitor.visit_any(cx, self, TypeHint::Any),
        }
    }
}

/// A JSON object key decoder for Müsli.
pub struct JsonKeyDecoder<P> {
    parser: P,
}

impl<'de, P> JsonKeyDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    fn skip_any<C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
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
    fn decode_escaped_bytes<C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Error>,
        V: ValueVisitor<'de, C, [u8]>,
    {
        let mut scratch = cx.alloc();

        match self.parser.parse_string(cx, true, &mut scratch)? {
            StringReference::Borrowed(string) => visitor.visit_borrowed(cx, string.as_bytes()),
            StringReference::Scratch(string) => visitor.visit_ref(cx, string.as_bytes()),
        }
    }
}

struct KeyUnsignedVisitor<C, T> {
    _marker: marker::PhantomData<(C, T)>,
}

impl<C, T> KeyUnsignedVisitor<C, T> {
    const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, C, T> ValueVisitor<'de, C, [u8]> for KeyUnsignedVisitor<C, T>
where
    C: Context<Input = Error>,
    T: Unsigned,
{
    type Ok = T;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        parse_unsigned(cx, &mut SliceParser::new(bytes))
    }
}

struct KeySignedVisitor<C, T> {
    _marker: marker::PhantomData<(C, T)>,
}

impl<C, T> KeySignedVisitor<C, T> {
    const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, C, T> ValueVisitor<'de, C, [u8]> for KeySignedVisitor<C, T>
where
    C: Context<Input = Error>,
    T: Signed,
{
    type Ok = T;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        parse_signed(cx, &mut SliceParser::new(bytes))
    }
}

#[musli::decoder]
impl<'de, P> Decoder<'de> for JsonKeyDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;
    type Struct = JsonObjectDecoder<P>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a object key")
    }

    #[inline]
    fn type_hint<C>(&mut self, cx: &mut C) -> Result<TypeHint, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonDecoder::new(&mut self.parser).type_hint(cx)
    }

    #[inline]
    fn decode_u8<C>(self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u16<C>(self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u32<C>(self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u64<C>(self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u128<C>(self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_i8<C>(self, cx: &mut C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i16<C>(self, cx: &mut C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i32<C>(self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i64<C>(self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i128<C>(self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_usize<C>(self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_isize<C>(self, cx: &mut C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_string<C, V>(self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
        C: Context<Input = Self::Error>,
    {
        JsonDecoder::new(self.parser).decode_string(cx, visitor)
    }

    #[inline]
    fn decode_any<C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = V::Error>,
        V: Visitor<'de, Error = Self::Error>,
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

pub struct JsonObjectDecoder<P> {
    first: bool,
    len: Option<usize>,
    parser: P,
}

impl<'de, P> JsonObjectDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    pub fn new<C>(cx: &mut C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error>
    where
        C: Context<Input = Error>,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedOpenBrace(actual))));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            first: true,
            len,
            parser,
        })
    }
}

impl<'de, P> PairsDecoder<'de> for JsonObjectDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Decoder<'this> = JsonObjectPairDecoder<P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn next<C>(&mut self, cx: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_string() {
                return Ok(Some(JsonObjectPairDecoder::new(self.parser.borrow_mut())));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(cx, 1)?;
                }
                Token::CloseBrace => {
                    self.parser.skip(cx, 1)?;
                    return Ok(None);
                }
                token => {
                    return Err(cx.message(format_args!(
                        "Expected value, or closing brace `}}` but found {token:?}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

pub struct JsonObjectPairDecoder<P> {
    parser: P,
}

impl<P> JsonObjectPairDecoder<P> {
    #[inline]
    fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<'de, P> PairDecoder<'de> for JsonObjectPairDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type First<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;

    type Second = JsonDecoder<P>;

    #[inline]
    fn first<C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn second<C>(mut self, cx: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser))
    }

    #[inline]
    fn skip_second<C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        JsonDecoder::new(self.parser.borrow_mut()).skip_any(cx)?;
        Ok(true)
    }
}

pub struct JsonSequenceDecoder<P> {
    len: Option<usize>,
    first: bool,
    parser: P,
    terminated: bool,
}

impl<'de, P> JsonSequenceDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    pub fn new<C>(cx: &mut C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error>
    where
        C: Context<Input = Error>,
    {
        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBracket) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedOpenBracket(actual))));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            len,
            first: true,
            parser,
            terminated: false,
        })
    }
}

impl<'de, P> SequenceDecoder<'de> for JsonSequenceDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Decoder<'this> = JsonDecoder<P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn next<C>(&mut self, cx: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_value() {
                return Ok(Some(JsonDecoder::new(self.parser.borrow_mut())));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(cx, 1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(cx, 1)?;
                    self.terminated = true;
                    return Ok(None);
                }
                _ => {
                    return Err(cx.message(format_args!(
                        "Expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.terminated {
            let actual = self.parser.peek(cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(cx.report(Error::new(ErrorKind::ExpectedCloseBracket(actual))));
            }

            self.parser.skip(cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}

impl<'de, P> PackDecoder<'de> for JsonSequenceDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Decoder<'this> = JsonDecoder<P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn next<C>(&mut self, cx: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_value() {
                return Ok(JsonDecoder::new(self.parser.borrow_mut()));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(cx, 1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(cx, 1)?;
                    self.terminated = true;

                    return Err(
                        cx.message(format_args!("Encountered short array, but found {token}"))
                    );
                }
                _ => {
                    return Err(cx.message(format_args!(
                        "Expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.terminated {
            let actual = self.parser.peek(cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(cx.report(Error::new(ErrorKind::ExpectedCloseBracket(actual))));
            }

            self.parser.skip(cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}

pub struct JsonVariantDecoder<P> {
    parser: P,
}

impl<'de, P> JsonVariantDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    pub fn new<C>(cx: &mut C, mut parser: P) -> Result<Self, C::Error>
    where
        C: Context<Input = Error>,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedOpenBrace(actual))));
        }

        parser.skip(cx, 1)?;
        Ok(Self { parser })
    }
}

impl<'de, P> VariantDecoder<'de> for JsonVariantDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Tag<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;

    type Variant<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, cx: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedColon(actual))));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_variant<C>(&mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let this = self.variant(cx)?;
        JsonDecoder::new(this.parser).skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedCloseBrace(actual))));
        }

        self.parser.skip(cx, 1)?;
        Ok(())
    }
}
