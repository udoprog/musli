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

use crate::reader::integer::{Signed, Unsigned};
use crate::reader::SliceParser;
use crate::reader::{
    integer, string, ParseError, ParseErrorKind, Parser, Scratch, StringReference, Token,
};

/// A JSON decoder for Müsli.
pub struct JsonDecoder<'a, P> {
    scratch: &'a mut Scratch,
    parser: P,
}

impl<'de, 'a, P> JsonDecoder<'a, P>
where
    P: Parser<'de>,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(scratch: &'a mut Scratch, parser: P) -> Self {
        Self { scratch, parser }
    }

    /// Skip over any values.
    pub(crate) fn skip_any<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        let start = self.parser.pos();
        let actual = self.parser.peek(cx)?;

        match actual {
            Token::OpenBrace => {
                let mut object = JsonObjectDecoder::new(cx, self.scratch, None, self.parser)?;

                while let Some(mut pair) = object.next(cx)? {
                    pair.first(cx)?.skip_any(cx)?;
                    pair.skip_second(cx)?;
                }
            }
            Token::OpenBracket => {
                let mut seq = JsonSequenceDecoder::new(cx, self.scratch, None, self.parser)?;

                while let Some(item) = SequenceDecoder::next(&mut seq, cx)? {
                    item.skip_any(cx)?;
                }
            }
            Token::Null => {
                return self.parse_null(cx);
            }
            Token::True => {
                return self.parse_true(cx);
            }
            Token::False => {
                return self.parse_false(cx);
            }
            Token::Number => {
                return integer::skip_number(cx, &mut self.parser);
            }
            Token::String => {
                return string::skip_string(cx, &mut self.parser, true);
            }
            actual => {
                return Err(cx.report(ParseError::spanned(
                    start,
                    self.parser.pos(),
                    ParseErrorKind::ExpectedValue(actual),
                )));
            }
        }

        todo!()
    }

    #[inline]
    fn parse_true<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        self.parser.parse_exact(cx, *b"true", |pos| {
            ParseError::at(pos, ParseErrorKind::ExpectedTrue)
        })
    }

    #[inline]
    fn parse_false<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        self.parser.parse_exact(cx, *b"false", |pos| {
            ParseError::at(pos, ParseErrorKind::ExpectedFalse)
        })
    }

    #[inline]
    fn parse_null<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        self.parser.parse_exact(cx, *b"null", |pos| {
            ParseError::at(pos, ParseErrorKind::ExpectedNull)
        })
    }
}

#[musli::decoder]
impl<'de, 'a, P> Decoder<'de> for JsonDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;
    #[cfg(feature = "musli-value")]
    type Buffer = musli_value::AsValueDecoder<Self::Error>;
    type Pack = JsonSequenceDecoder<'a, P>;
    type Sequence = JsonSequenceDecoder<'a, P>;
    type Tuple = JsonSequenceDecoder<'a, P>;
    type Map = JsonObjectDecoder<'a, P>;
    type Some = JsonDecoder<'a, P>;
    type Struct = JsonObjectDecoder<'a, P>;
    type Variant = JsonVariantDecoder<'a, P>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSON")
    }

    #[inline]
    fn type_hint<'buf, C>(&mut self, cx: &mut C) -> Result<TypeHint, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
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
    fn decode_buffer<'buf, M, C>(self, cx: &mut C) -> Result<Self::Buffer, C::Error>
    where
        M: Mode,
        C: Context<'buf, Input = Self::Error>,
    {
        use musli::de::Decode;
        let value: musli_value::Value = Decode::<M>::decode(cx, self)?;
        Ok(value.into_value_decoder())
    }

    #[inline]
    fn decode_unit<'buf, C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.skip_any(cx)
    }

    #[inline]
    fn decode_bool<'buf, C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
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
            actual => Err(cx.report(ParseError::at(
                self.parser.pos(),
                ParseErrorKind::ExpectedBool(actual),
            ))),
        }
    }

    #[inline]
    fn decode_char<'buf, C>(mut self, cx: &mut C) -> Result<char, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let start = self.parser.pos();

        let string = match self.parser.parse_string(cx, self.scratch, true)? {
            StringReference::Borrowed(string) => string,
            StringReference::Scratch(string) => string,
        };

        let mut it = string.chars();
        let first = it.next();

        match (first, it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(cx.report(ParseError::spanned(
                start,
                self.parser.pos(),
                ParseErrorKind::CharEmptyString,
            ))),
        }
    }

    #[inline]
    fn decode_u8<'buf, C>(mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u16<'buf, C>(mut self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u32<'buf, C>(mut self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u64<'buf, C>(mut self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u128<'buf, C>(mut self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i8<'buf, C>(mut self, cx: &mut C) -> Result<i8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i16<'buf, C>(mut self, cx: &mut C) -> Result<i16, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i32<'buf, C>(mut self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i64<'buf, C>(mut self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i128<'buf, C>(mut self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_usize<'buf, C>(mut self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_isize<'buf, C>(mut self, cx: &mut C) -> Result<isize, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        integer::parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_f32<'buf, C>(mut self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.parser.parse_f32(cx)
    }

    #[inline]
    fn decode_f64<'buf, C>(mut self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.parser.parse_f64(cx)
    }

    #[inline]
    fn decode_number<'buf, C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: NumberVisitor<'de, 'buf, C>,
    {
        self.parser.parse_number(cx, visitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<'buf, C, V>(self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, [u8]>,
    {
        let mut seq = self.decode_sequence(cx)?;
        let mut bytes = Vec::with_capacity(seq.size_hint().or_default());

        while let Some(item) = SequenceDecoder::next(&mut seq, cx)? {
            bytes.push(item.decode_u8(cx)?);
        }

        visitor.visit_owned(cx, bytes)
    }

    #[inline]
    fn decode_string<'buf, C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, str>,
    {
        match self.parser.parse_string(cx, self.scratch, true)? {
            StringReference::Borrowed(borrowed) => visitor.visit_borrowed(cx, borrowed),
            StringReference::Scratch(string) => visitor.visit_ref(cx, string),
        }
    }

    #[inline]
    fn decode_option<'buf, C>(mut self, cx: &mut C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        if self.parser.peek(cx)?.is_null() {
            self.parse_null(cx)?;
            Ok(None)
        } else {
            Ok(Some(self))
        }
    }

    #[inline]
    fn decode_pack<'buf, C>(self, cx: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, self.scratch, None, self.parser)
    }

    #[inline]
    fn decode_sequence<'buf, C>(self, cx: &mut C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, self.scratch, None, self.parser)
    }

    #[inline]
    fn decode_tuple<'buf, C>(self, cx: &mut C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, self.scratch, Some(len), self.parser)
    }

    #[inline]
    fn decode_map<'buf, C>(self, cx: &mut C) -> Result<Self::Map, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, self.scratch, None, self.parser)
    }

    #[inline]
    fn decode_struct<'buf, C>(self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, self.scratch, Some(len), self.parser)
    }

    #[inline]
    fn decode_variant<'buf, C>(self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonVariantDecoder::new(cx, self.scratch, self.parser)
    }

    #[inline]
    fn decode_any<'buf, C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = ParseError>,
        V: Visitor<'de, Error = Self::Error>,
    {
        self.parser.skip_whitespace(cx)?;

        match self.parser.peek(cx)? {
            Token::OpenBrace => {
                let decoder = JsonObjectDecoder::new(cx, self.scratch, None, self.parser)?;
                visitor.visit_map(cx, decoder)
            }
            Token::OpenBracket => {
                let decoder = JsonSequenceDecoder::new(cx, self.scratch, None, self.parser)?;
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
pub struct JsonKeyDecoder<'a, P> {
    scratch: &'a mut Scratch,
    parser: P,
}

impl<'de, 'a, P> JsonKeyDecoder<'a, P>
where
    P: Parser<'de>,
{
    #[inline]
    fn skip_any<'buf, C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        JsonDecoder::new(self.scratch, self.parser).skip_any(cx)
    }
}

impl<'de, 'a, P> JsonKeyDecoder<'a, P>
where
    P: Parser<'de>,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(scratch: &'a mut Scratch, parser: P) -> Self {
        Self { scratch, parser }
    }

    #[inline]
    fn decode_escaped_bytes<'buf, C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = ParseError>,
        V: ValueVisitor<'de, 'buf, C, [u8]>,
    {
        match self.parser.parse_string(cx, self.scratch, true)? {
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

impl<'de, 'buf, C, T> ValueVisitor<'de, 'buf, C, [u8]> for KeyUnsignedVisitor<C, T>
where
    C: Context<'buf, Input = ParseError>,
    T: Unsigned,
{
    type Ok = T;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        integer::parse_unsigned(cx, &mut SliceParser::new(bytes))
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

impl<'de, 'buf, C, T> ValueVisitor<'de, 'buf, C, [u8]> for KeySignedVisitor<C, T>
where
    C: Context<'buf, Input = ParseError>,
    T: Signed,
{
    type Ok = T;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        integer::parse_signed(cx, &mut SliceParser::new(bytes))
    }
}

#[musli::decoder]
impl<'de, 'a, P> Decoder<'de> for JsonKeyDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;
    type Struct = JsonObjectDecoder<'a, P>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a object key")
    }

    #[inline]
    fn type_hint<'buf, C>(&mut self, cx: &mut C) -> Result<TypeHint, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonDecoder::new(self.scratch, &mut self.parser).type_hint(cx)
    }

    #[inline]
    fn decode_u8<'buf, C>(self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u16<'buf, C>(self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u32<'buf, C>(self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u64<'buf, C>(self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u128<'buf, C>(self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_i8<'buf, C>(self, cx: &mut C) -> Result<i8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i16<'buf, C>(self, cx: &mut C) -> Result<i16, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i32<'buf, C>(self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i64<'buf, C>(self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i128<'buf, C>(self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_usize<'buf, C>(self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_isize<'buf, C>(self, cx: &mut C) -> Result<isize, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.decode_escaped_bytes(cx, KeySignedVisitor::new())
    }

    #[inline]
    fn decode_string<'buf, C, V>(self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, 'buf, C, str>,
        C: Context<'buf, Input = Self::Error>,
    {
        JsonDecoder::new(self.scratch, self.parser).decode_string(cx, visitor)
    }

    #[inline]
    fn decode_any<'buf, C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = V::Error>,
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

pub struct JsonObjectDecoder<'a, P> {
    scratch: &'a mut Scratch,
    first: bool,
    len: Option<usize>,
    parser: P,
}

impl<'de, 'a, P> JsonObjectDecoder<'a, P>
where
    P: Parser<'de>,
{
    #[inline]
    pub fn new<'buf, C>(
        cx: &mut C,
        scratch: &'a mut Scratch,
        len: Option<usize>,
        mut parser: P,
    ) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.report(ParseError::at(
                parser.pos(),
                ParseErrorKind::ExpectedOpenBrace(actual),
            )));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            scratch,
            first: true,
            len,
            parser,
        })
    }
}

impl<'de, 'a, P> PairsDecoder<'de> for JsonObjectDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;

    type Decoder<'this> = JsonObjectPairDecoder<'this, P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn next<'buf, C>(&mut self, cx: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_string() {
                return Ok(Some(JsonObjectPairDecoder::new(
                    self.scratch,
                    self.parser.borrow_mut(),
                )));
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
                        "expected value, or closing brace `}}` {token:?}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

pub struct JsonObjectPairDecoder<'a, P> {
    scratch: &'a mut Scratch,
    parser: P,
}

impl<'a, P> JsonObjectPairDecoder<'a, P> {
    #[inline]
    fn new(scratch: &'a mut Scratch, parser: P) -> Self {
        Self { scratch, parser }
    }
}

impl<'de, 'a, P> PairDecoder<'de> for JsonObjectPairDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;

    type First<'this> = JsonKeyDecoder<'this, P::Mut<'this>>
    where
        Self: 'this;

    type Second = JsonDecoder<'a, P>;

    #[inline]
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(JsonKeyDecoder::new(
            &mut *self.scratch,
            self.parser.borrow_mut(),
        ))
    }

    #[inline]
    fn second<'buf, C>(mut self, cx: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.scratch, self.parser))
    }

    #[inline]
    fn skip_second<'buf, C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        JsonDecoder::new(self.scratch, self.parser.borrow_mut()).skip_any(cx)?;
        Ok(true)
    }
}

pub struct JsonSequenceDecoder<'a, P> {
    scratch: &'a mut Scratch,
    len: Option<usize>,
    first: bool,
    parser: P,
    terminated: bool,
}

impl<'de, 'a, P> JsonSequenceDecoder<'a, P>
where
    P: Parser<'de>,
{
    #[inline]
    pub fn new<'buf, C>(
        cx: &mut C,
        scratch: &'a mut Scratch,
        len: Option<usize>,
        mut parser: P,
    ) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBracket) {
            return Err(cx.report(ParseError::at(
                parser.pos(),
                ParseErrorKind::ExpectedOpenBracket(actual),
            )));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            scratch,
            len,
            first: true,
            parser,
            terminated: false,
        })
    }
}

impl<'de, 'a, P> SequenceDecoder<'de> for JsonSequenceDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;

    type Decoder<'this> = JsonDecoder<'this, P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn next<'buf, C>(&mut self, cx: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_value() {
                return Ok(Some(JsonDecoder::new(
                    self.scratch,
                    self.parser.borrow_mut(),
                )));
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
                        "expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if !self.terminated {
            let actual = self.parser.peek(cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(cx.report(ParseError::at(
                    self.parser.pos(),
                    ParseErrorKind::ExpectedCloseBracket(actual),
                )));
            }

            self.parser.skip(cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}

impl<'de, 'a, P> PackDecoder<'de> for JsonSequenceDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;

    type Decoder<'this> = JsonDecoder<'this, P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, cx: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_value() {
                return Ok(JsonDecoder::new(self.scratch, self.parser.borrow_mut()));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(cx, 1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(cx, 1)?;
                    self.terminated = true;

                    return Err(
                        cx.message(format_args!("encountered short array, but found {token}"))
                    );
                }
                _ => {
                    return Err(cx.message(format_args!(
                        "expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if !self.terminated {
            let actual = self.parser.peek(cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(cx.report(ParseError::at(
                    self.parser.pos(),
                    ParseErrorKind::ExpectedCloseBracket(actual),
                )));
            }

            self.parser.skip(cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}

pub struct JsonVariantDecoder<'a, P> {
    scratch: &'a mut Scratch,
    parser: P,
}

impl<'de, 'a, P> JsonVariantDecoder<'a, P>
where
    P: Parser<'de>,
{
    #[inline]
    pub fn new<'buf, C>(
        cx: &mut C,
        scratch: &'a mut Scratch,
        mut parser: P,
    ) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = ParseError>,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.report(ParseError::at(
                parser.pos(),
                ParseErrorKind::ExpectedOpenBrace(actual),
            )));
        }

        parser.skip(cx, 1)?;
        Ok(Self { scratch, parser })
    }
}

impl<'de, 'a, P> VariantDecoder<'de> for JsonVariantDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;

    type Tag<'this> = JsonKeyDecoder<'this, P::Mut<'this>>
    where
        Self: 'this;

    type Variant<'this> = JsonDecoder<'this, P::Mut<'this>> where Self: 'this;

    #[inline]
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(JsonKeyDecoder::new(self.scratch, self.parser.borrow_mut()))
    }

    #[inline]
    fn variant<'buf, C>(&mut self, cx: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.report(ParseError::at(
                self.parser.pos(),
                ParseErrorKind::ExpectedColon(actual),
            )));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.scratch, self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_variant<'buf, C>(&mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let this = self.variant(cx)?;
        JsonDecoder::new(this.scratch, this.parser).skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(cx.report(ParseError::at(
                self.parser.pos(),
                ParseErrorKind::ExpectedCloseBrace(actual),
            )));
        }

        self.parser.skip(cx, 1)?;
        Ok(())
    }
}
