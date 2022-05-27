use core::fmt;
use core::marker;
use core::mem;
use core::str;

use musli::de::{
    Decoder, LengthHint, NumberHint, NumberVisitor, PackDecoder, PairDecoder, PairsDecoder,
    SequenceDecoder, TypeHint, ValueVisitor, VariantDecoder,
};
use musli::error::Error;
#[cfg(feature = "std")]
use musli::mode::Mode;
use musli::never::Never;

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
    pub(crate) fn skip_any(mut self) -> Result<(), ParseError> {
        let start = self.parser.pos();
        let actual = self.parser.peek()?;

        match actual {
            Token::OpenBrace => {
                let mut object = JsonObjectDecoder::new(self.scratch, None, self.parser)?;

                while let Some(mut pair) = object.next()? {
                    pair.first()?.skip_any()?;
                    pair.skip_second()?;
                }
            }
            Token::OpenBracket => {
                let mut seq = JsonSequenceDecoder::new(self.scratch, None, self.parser)?;

                while let Some(item) = SequenceDecoder::next(&mut seq)? {
                    item.skip_any()?;
                }
            }
            Token::Null => {
                return self.parse_null();
            }
            Token::True => {
                return self.parse_true();
            }
            Token::False => {
                return self.parse_false();
            }
            Token::Number => {
                return integer::skip_number(&mut self.parser);
            }
            Token::String => {
                return string::skip_string(&mut self.parser, true);
            }
            actual => {
                return Err(ParseError::spanned(
                    start,
                    self.parser.pos(),
                    ParseErrorKind::ExpectedValue(actual),
                ))
            }
        }

        todo!()
    }

    #[inline]
    fn parse_true(mut self) -> Result<(), ParseError> {
        self.parser.parse_exact(*b"true", |pos| {
            ParseError::at(pos, ParseErrorKind::ExpectedTrue)
        })
    }

    #[inline]
    fn parse_false(mut self) -> Result<(), ParseError> {
        self.parser.parse_exact(*b"false", |pos| {
            ParseError::at(pos, ParseErrorKind::ExpectedFalse)
        })
    }

    #[inline]
    fn parse_null(mut self) -> Result<(), ParseError> {
        self.parser.parse_exact(*b"null", |pos| {
            ParseError::at(pos, ParseErrorKind::ExpectedNull)
        })
    }
}

impl<'de, 'a, P> Decoder<'de> for JsonDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;
    #[cfg(not(feature = "std"))]
    type Buffer = Never<Self::Error>;
    #[cfg(feature = "std")]
    type Buffer = musli_value::AsValueDecoder<Self::Error>;
    type Pack = Never<Self::Error>;
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
    fn type_hint(&mut self) -> Result<TypeHint, Self::Error> {
        Ok(match self.parser.peek()? {
            Token::OpenBrace => TypeHint::Map(LengthHint::Any),
            Token::OpenBracket => TypeHint::Sequence(LengthHint::Any),
            Token::String => TypeHint::String(LengthHint::Any),
            Token::Number => TypeHint::Number(NumberHint::Any),
            Token::Null => TypeHint::Unit,
            Token::True => TypeHint::Bool,
            Token::False => TypeHint::Bool,
            _ => TypeHint::Any,
        })
    }

    #[cfg(feature = "std")]
    #[inline]
    fn decode_buffer<M>(self) -> Result<Self::Buffer, Self::Error>
    where
        M: Mode,
    {
        use musli::de::Decode;
        let value: musli_value::Value = Decode::<M>::decode(self)?;
        Ok(value.into_value_decoder())
    }

    #[inline]
    fn decode_unit(self) -> Result<(), Self::Error> {
        self.skip_any()
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, Self::Error> {
        match self.parser.peek()? {
            Token::True => {
                self.parse_true()?;
                Ok(true)
            }
            Token::False => {
                self.parse_false()?;
                Ok(false)
            }
            actual => Err(ParseError::at(
                self.parser.pos(),
                ParseErrorKind::ExpectedBool(actual),
            )),
        }
    }

    #[inline]
    fn decode_char(mut self) -> Result<char, Self::Error> {
        let start = self.parser.pos();

        let string = match self.parser.parse_string(self.scratch, true)? {
            StringReference::Borrowed(string) => string,
            StringReference::Scratch(string) => string,
        };

        let mut it = string.chars();
        let first = it.next();

        match (first, it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(ParseError::spanned(
                start,
                self.parser.pos(),
                ParseErrorKind::CharEmptyString,
            )),
        }
    }

    #[inline]
    fn decode_u8(mut self) -> Result<u8, Self::Error> {
        integer::parse_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u16(mut self) -> Result<u16, Self::Error> {
        integer::parse_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u32(mut self) -> Result<u32, Self::Error> {
        integer::parse_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u64(mut self) -> Result<u64, Self::Error> {
        integer::parse_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u128(mut self) -> Result<u128, Self::Error> {
        integer::parse_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_i8(mut self) -> Result<i8, Self::Error> {
        integer::parse_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i16(mut self) -> Result<i16, Self::Error> {
        integer::parse_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i32(mut self) -> Result<i32, Self::Error> {
        integer::parse_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i64(mut self) -> Result<i64, Self::Error> {
        integer::parse_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i128(mut self) -> Result<i128, Self::Error> {
        integer::parse_signed(&mut self.parser)
    }

    #[inline]
    fn decode_usize(mut self) -> Result<usize, Self::Error> {
        integer::parse_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_isize(mut self) -> Result<isize, Self::Error> {
        integer::parse_signed(&mut self.parser)
    }

    #[inline]
    fn decode_f32(mut self) -> Result<f32, Self::Error> {
        self.parser.parse_f32()
    }

    #[inline]
    fn decode_f64(mut self) -> Result<f64, Self::Error> {
        self.parser.parse_f64()
    }

    #[inline]
    fn decode_number<V>(mut self, visitor: V) -> Result<V::Ok, Self::Error>
    where
        V: NumberVisitor<Error = Self::Error>,
    {
        self.parser.parse_number(visitor)
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::Some>, Self::Error> {
        if self.parser.peek()?.is_null() {
            self.parse_null()?;
            Ok(None)
        } else {
            Ok(Some(self))
        }
    }

    #[cfg(feature = "std")]
    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let mut seq = self.decode_sequence()?;
        let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or_default());

        while let Some(item) = SequenceDecoder::next(&mut seq)? {
            bytes.push(item.decode_u8()?);
        }

        visitor.visit_owned(bytes)
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        match self.parser.parse_string(self.scratch, true)? {
            StringReference::Borrowed(borrowed) => visitor.visit_borrowed(borrowed),
            StringReference::Scratch(string) => visitor.visit_any(string),
        }
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        JsonSequenceDecoder::new(self.scratch, None, self.parser)
    }

    #[inline]
    fn decode_tuple(self, len: usize) -> Result<Self::Tuple, Self::Error> {
        JsonSequenceDecoder::new(self.scratch, Some(len), self.parser)
    }

    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        JsonObjectDecoder::new(self.scratch, None, self.parser)
    }

    #[inline]
    fn decode_struct(self, len: usize) -> Result<Self::Struct, Self::Error> {
        JsonObjectDecoder::new(self.scratch, Some(len), self.parser)
    }

    #[inline]
    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
        JsonVariantDecoder::new(self.scratch, self.parser)
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
    fn skip_any(self) -> Result<(), ParseError> {
        JsonDecoder::new(self.scratch, self.parser).skip_any()
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
    fn decode_escaped_bytes<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = ParseError>,
    {
        match self.parser.parse_string(self.scratch, true)? {
            StringReference::Borrowed(string) => visitor.visit_borrowed(string.as_bytes()),
            StringReference::Scratch(string) => visitor.visit_any(string.as_bytes()),
        }
    }
}

struct KeyUnsignedVisitor<T> {
    _marker: marker::PhantomData<T>,
}

impl<T> KeyUnsignedVisitor<T> {
    const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, T> ValueVisitor<'de> for KeyUnsignedVisitor<T>
where
    T: Unsigned,
{
    type Target = [u8];
    type Ok = T;
    type Error = ParseError;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a string")
    }

    #[inline]
    fn visit_any(self, bytes: &Self::Target) -> Result<Self::Ok, Self::Error> {
        integer::parse_unsigned(&mut &mut SliceParser::new(bytes))
    }
}

struct KeySignedVisitor<T> {
    _marker: marker::PhantomData<T>,
}

impl<T> KeySignedVisitor<T> {
    const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, T> ValueVisitor<'de> for KeySignedVisitor<T>
where
    T: Signed,
{
    type Target = [u8];
    type Ok = T;
    type Error = ParseError;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a string")
    }

    #[inline]
    fn visit_any(self, bytes: &Self::Target) -> Result<Self::Ok, Self::Error> {
        integer::parse_signed(&mut SliceParser::new(bytes))
    }
}

impl<'de, 'a, P> Decoder<'de> for JsonKeyDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;
    type Buffer = Never<Self::Error>;
    type Pack = Never<Self::Error>;
    type Sequence = Never<Self::Error>;
    type Tuple = Never<Self::Error>;
    type Map = Never<Self::Error>;
    type Some = Never<Self::Error>;
    type Struct = JsonObjectDecoder<'a, P>;
    type Variant = Never<Self::Error>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a object key")
    }

    #[inline]
    fn type_hint(&mut self) -> Result<TypeHint, Self::Error> {
        JsonDecoder::new(self.scratch, &mut self.parser).type_hint()
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.decode_escaped_bytes(KeyUnsignedVisitor::new())
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.decode_escaped_bytes(KeySignedVisitor::new())
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        JsonDecoder::new(self.scratch, self.parser).decode_string(visitor)
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
    pub fn new(
        scratch: &'a mut Scratch,
        len: Option<usize>,
        mut parser: P,
    ) -> Result<Self, ParseError> {
        parser.skip_whitespace()?;

        let actual = parser.peek()?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(ParseError::at(
                parser.pos(),
                ParseErrorKind::ExpectedOpenBrace(actual),
            ));
        }

        parser.skip(1)?;

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
    fn size_hint(&self) -> Option<usize> {
        self.len
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek()?;

            if token.is_string() {
                return Ok(Some(JsonObjectPairDecoder::new(
                    self.scratch,
                    self.parser.borrow_mut(),
                )));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(1)?;
                }
                Token::CloseBrace => {
                    self.parser.skip(1)?;
                    return Ok(None);
                }
                _ => {
                    return Err(Self::Error::message("expected value, or closing brace `}`"));
                }
            }
        }
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
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
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(JsonKeyDecoder::new(
            &mut *self.scratch,
            self.parser.borrow_mut(),
        ))
    }

    #[inline]
    fn second(mut self) -> Result<Self::Second, Self::Error> {
        let actual = self.parser.peek()?;

        if !matches!(actual, Token::Colon) {
            return Err(Self::Error::message(format_args!(
                "expected colon `:`, was {actual}"
            )));
        }

        self.parser.skip(1)?;
        Ok(JsonDecoder::new(self.scratch, self.parser))
    }

    #[inline]
    fn skip_second(mut self) -> Result<bool, Self::Error> {
        let actual = self.parser.peek()?;

        if !matches!(actual, Token::Colon) {
            return Err(Self::Error::message(format_args!(
                "expected colon `:`, was {actual}"
            )));
        }

        self.parser.skip(1)?;
        JsonDecoder::new(self.scratch, self.parser.borrow_mut()).skip_any()?;
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
    pub fn new(
        scratch: &'a mut Scratch,
        len: Option<usize>,
        mut parser: P,
    ) -> Result<Self, ParseError> {
        parser.skip_whitespace()?;

        let actual = parser.peek()?;

        if !matches!(actual, Token::OpenBracket) {
            return Err(ParseError::at(
                parser.pos(),
                ParseErrorKind::ExpectedOpenBracket(actual),
            ));
        }

        parser.skip(1)?;

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
    fn size_hint(&self) -> Option<usize> {
        self.len
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek()?;

            if token.is_value() {
                return Ok(Some(JsonDecoder::new(
                    self.scratch,
                    self.parser.borrow_mut(),
                )));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(1)?;
                    self.terminated = true;
                    return Ok(None);
                }
                _ => {
                    return Err(Self::Error::message(format_args!(
                        "expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end(mut self) -> Result<(), Self::Error> {
        if !self.terminated {
            let actual = self.parser.peek()?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(ParseError::at(
                    self.parser.pos(),
                    ParseErrorKind::ExpectedCloseBracket(actual),
                ));
            }

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
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek()?;

            if token.is_value() {
                return Ok(JsonDecoder::new(self.scratch, self.parser.borrow_mut()));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(1)?;
                    self.terminated = true;

                    return Err(Self::Error::message(format_args!(
                        "encountered short array, but found {token}"
                    )));
                }
                _ => {
                    return Err(Self::Error::message(format_args!(
                        "expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end(mut self) -> Result<(), Self::Error> {
        if !self.terminated {
            let actual = self.parser.peek()?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(ParseError::at(
                    self.parser.pos(),
                    ParseErrorKind::ExpectedCloseBracket(actual),
                ));
            }

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
    pub fn new(scratch: &'a mut Scratch, mut parser: P) -> Result<Self, ParseError> {
        parser.skip_whitespace()?;

        let actual = parser.peek()?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(ParseError::at(
                parser.pos(),
                ParseErrorKind::ExpectedOpenBrace(actual),
            ));
        }

        parser.skip(1)?;

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
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        Ok(JsonKeyDecoder::new(self.scratch, self.parser.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        let actual = self.parser.peek()?;

        if !matches!(actual, Token::Colon) {
            return Err(ParseError::at(
                self.parser.pos(),
                ParseErrorKind::ExpectedColon(actual),
            ));
        }

        self.parser.skip(1)?;
        Ok(JsonDecoder::new(self.scratch, self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_variant(&mut self) -> Result<bool, Self::Error> {
        let this = self.variant()?;
        JsonDecoder::new(this.scratch, this.parser).skip_any()?;
        Ok(true)
    }

    #[inline]
    fn end(mut self) -> Result<(), Self::Error> {
        let actual = self.parser.peek()?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(ParseError::at(
                self.parser.pos(),
                ParseErrorKind::ExpectedCloseBrace(actual),
            ));
        }

        self.parser.skip(1)?;
        Ok(())
    }
}
