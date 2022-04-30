use core::fmt;
use core::marker;
use core::mem;
use core::str;

use musli::de::PackDecoder;
use musli::de::SequenceDecoder;
use musli::de::{Decoder, PairDecoder, PairsDecoder, ValueVisitor};
use musli::error::Error;
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
    type Pack = Never<Self>;
    type Sequence = JsonSequenceDecoder<'a, P>;
    type Tuple = JsonSequenceDecoder<'a, P>;
    type Map = JsonObjectDecoder<'a, P>;
    type Some = JsonDecoder<'a, P>;
    type Struct = JsonObjectDecoder<'a, P>;
    type TupleStruct = JsonObjectDecoder<'a, P>;
    type Variant = Never<Self>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSON")
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
    fn decode_u8(mut self) -> Result<u8, Self::Error> {
        integer::decode_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u16(mut self) -> Result<u16, Self::Error> {
        integer::decode_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u32(mut self) -> Result<u32, Self::Error> {
        integer::decode_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u64(mut self) -> Result<u64, Self::Error> {
        integer::decode_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_u128(mut self) -> Result<u128, Self::Error> {
        integer::decode_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_i8(mut self) -> Result<i8, Self::Error> {
        integer::decode_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i16(mut self) -> Result<i16, Self::Error> {
        integer::decode_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i32(mut self) -> Result<i32, Self::Error> {
        integer::decode_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i64(mut self) -> Result<i64, Self::Error> {
        integer::decode_signed(&mut self.parser)
    }

    #[inline]
    fn decode_i128(mut self) -> Result<i128, Self::Error> {
        integer::decode_signed(&mut self.parser)
    }

    #[inline]
    fn decode_usize(mut self) -> Result<usize, Self::Error> {
        integer::decode_unsigned(&mut self.parser)
    }

    #[inline]
    fn decode_isize(mut self) -> Result<isize, Self::Error> {
        integer::decode_signed(&mut self.parser)
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

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        let actual = self.parser.peek()?;

        if !matches!(actual, Token::String) {
            return Err(V::Error::message(format_args!(
                "expected string, but was {actual}"
            )));
        }

        self.parser.skip(1)?;

        match self.parser.parse_string(self.scratch, true)? {
            StringReference::Borrowed(borrowed) => {
                // SAFETY: safety is guaranteed by the implementation of
                // `parse_string`.
                let string = unsafe { str::from_utf8_unchecked(borrowed) };
                visitor.visit_borrowed(string)
            }
            StringReference::Scratch(string) => {
                // SAFETY: safety is guaranteed by the implementation of
                // `parse_string`.
                let string = unsafe { str::from_utf8_unchecked(string) };
                visitor.visit_any(string)
            }
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
    fn decode_tuple_struct(self, len: usize) -> Result<Self::Struct, Self::Error> {
        JsonObjectDecoder::new(self.scratch, Some(len), self.parser)
    }

    #[inline]
    fn decode_unit_struct(self) -> Result<(), Self::Error> {
        self.skip_any()
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
        let actual = self.parser.peek()?;

        if !matches!(actual, Token::String) {
            return Err(V::Error::message(format_args!(
                "expected string, but was {actual}"
            )));
        }

        self.parser.skip(1)?;

        match self.parser.parse_string(self.scratch, true)? {
            StringReference::Borrowed(bytes) => visitor.visit_borrowed(bytes),
            StringReference::Scratch(bytes) => visitor.visit_any(bytes),
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
        integer::decode_unsigned(&mut &mut SliceParser::new(bytes))
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
        integer::decode_signed(&mut SliceParser::new(bytes))
    }
}

impl<'de, 'a, P> Decoder<'de> for JsonKeyDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;
    type Pack = Never<Self>;
    type Sequence = Never<Self>;
    type Tuple = Never<Self>;
    type Map = Never<Self>;
    type Some = Never<Self>;
    type Struct = JsonObjectDecoder<'a, P>;
    type TupleStruct = Never<Self>;
    type Variant = Never<Self>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a object key")
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
        Ok(JsonDecoder::new(&mut *self.scratch, self.parser))
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
        JsonDecoder::new(self.scratch, self.parser).skip_any()?;
        Ok(true)
    }
}

pub struct JsonSequenceDecoder<'a, P> {
    scratch: &'a mut Scratch,
    len: Option<usize>,
    first: bool,
    parser: P,
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

    fn size_hint(&self) -> Option<usize> {
        self.len
    }

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
}

impl<'de, 'a, P> PackDecoder<'de> for JsonSequenceDecoder<'a, P>
where
    P: Parser<'de>,
{
    type Error = ParseError;

    type Decoder<'this> = JsonDecoder<'this, P::Mut<'this>>
    where
        Self: 'this;

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
}
