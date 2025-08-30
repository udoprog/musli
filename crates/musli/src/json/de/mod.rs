mod object_decoder;
use self::object_decoder::JsonObjectDecoder;

mod object_pair_decoder;
use self::object_pair_decoder::JsonObjectPairDecoder;

mod key_decoder;
use self::key_decoder::JsonKeyDecoder;

mod key_unsigned_visitor;
use self::key_unsigned_visitor::KeyUnsignedVisitor;

mod key_signed_visitor;
use self::key_signed_visitor::KeySignedVisitor;

mod sequence_decoder;
use self::sequence_decoder::JsonSequenceDecoder;

mod variant_decoder;
use self::variant_decoder::JsonVariantDecoder;

use core::fmt;
use core::marker::PhantomData;
use core::str;

use crate::alloc::Vec;
use crate::de::{Decoder, SequenceDecoder, SizeHint, Skip, UnsizedVisitor, Visitor};
use crate::hint::{MapHint, SequenceHint};
use crate::options;
use crate::value::{IntoValueDecoder, Value};
use crate::Context;
use crate::Options;

#[cfg(not(feature = "parse-full"))]
use super::parser::integer::{
    parse_signed_base as parse_signed, parse_unsigned_base as parse_unsigned,
};
#[cfg(feature = "parse-full")]
use super::parser::integer::{
    parse_signed_full as parse_signed, parse_unsigned_full as parse_unsigned,
};
use super::parser::{integer, Parser, StringReference, Token};

const BUFFER_OPTIONS: Options = options::new().map_keys_as_numbers().build();

/// A JSON decoder for Müsli.
pub(crate) struct JsonDecoder<P, C, M> {
    cx: C,
    parser: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonDecoder<P, C, M>
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

    /// Skip over any values.
    pub(crate) fn skip_any(mut self) -> Result<(), C::Error> {
        let start = self.cx.mark();
        let actual = self.parser.lex(self.cx);

        match actual {
            Token::OpenBrace => self.decode_map(|_| Ok(())),
            Token::OpenBracket => self.decode_sequence(|_| Ok(())),
            Token::Null => self.parse_null(),
            Token::True => self.parse_true(),
            Token::False => self.parse_false(),
            Token::Number => integer::skip_number(self.cx, self.parser.borrow_mut()),
            Token::String => {
                // Skip over opening quote.
                self.parser.skip(self.cx, 1)?;
                self.parser.skip_string_inner(self.cx)
            }
            actual => Err(self
                .cx
                .message_at(&start, format_args!("Expected value, found {actual}"))),
        }
    }

    #[inline]
    fn parse_true(mut self) -> Result<(), C::Error> {
        self.parser.parse_exact(self.cx, "true")
    }

    #[inline]
    fn parse_false(mut self) -> Result<(), C::Error> {
        self.parser.parse_exact(self.cx, "false")
    }

    #[inline]
    fn parse_null(mut self) -> Result<(), C::Error> {
        self.parser.parse_exact(self.cx, "null")
    }
}

#[crate::trait_defaults(crate)]
impl<'de, P, C, M> Decoder<'de> for JsonDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = JsonDecoder<P::TryClone, C, M>;
    type DecodeBuffer = IntoValueDecoder<BUFFER_OPTIONS, C, C::Allocator, M>;
    type DecodePack = JsonSequenceDecoder<P, C, M>;
    type DecodeSequence = JsonSequenceDecoder<P, C, M>;
    type DecodeMap = JsonObjectDecoder<P, C, M>;
    type DecodeMapEntries = JsonObjectDecoder<P, C, M>;
    type DecodeSome = JsonDecoder<P, C, M>;
    type DecodeVariant = JsonVariantDecoder<P, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSON")
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(JsonDecoder::new(self.cx, self.parser.try_clone()?))
    }

    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        self.skip_any()
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        self.skip()?;
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, Self::Error> {
        let cx = self.cx;
        let value = self.decode::<Value<Self::Allocator>>()?;
        // JSON: Encodes numbers in objects as strings, so we need to permit
        // treating them as such here as well.
        Ok(value.into_decoder(cx))
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        self.skip()
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, Self::Error> {
        match self.parser.lex(self.cx) {
            Token::True => {
                self.parse_true()?;
                Ok(true)
            }
            Token::False => {
                self.parse_false()?;
                Ok(false)
            }
            actual => Err(self
                .cx
                .message(format_args!("Expected boolean, was {actual}"))),
        }
    }

    #[inline]
    fn decode_char(mut self) -> Result<char, Self::Error> {
        let start = self.cx.mark();
        let mut scratch = Vec::new_in(self.cx.alloc());

        let string = self.parser.parse_string(self.cx, true, &mut scratch)?;
        let string = string.as_str();

        let mut it = string.chars();
        let first = it.next();

        match (first, it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(self
                .cx
                .message_at(&start, "Expected string with a single character")),
        }
    }

    #[inline]
    fn decode_u8(mut self) -> Result<u8, Self::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u16(mut self) -> Result<u16, Self::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u32(mut self) -> Result<u32, Self::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u64(mut self) -> Result<u64, Self::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u128(mut self) -> Result<u128, Self::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i8(mut self) -> Result<i8, Self::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i16(mut self) -> Result<i16, Self::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i32(mut self) -> Result<i32, Self::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i64(mut self) -> Result<i64, Self::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i128(mut self) -> Result<i128, Self::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_usize(mut self) -> Result<usize, Self::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_isize(mut self) -> Result<isize, Self::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_f32(mut self) -> Result<f32, Self::Error> {
        self.parser.skip_whitespace(self.cx);
        self.parser.parse_f32(self.cx)
    }

    #[inline]
    fn decode_f64(mut self) -> Result<f64, Self::Error> {
        self.parser.skip_whitespace(self.cx);
        self.parser.parse_f64(self.cx)
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        let cx = self.cx;
        let mark = cx.mark();

        self.decode_sequence(|seq| {
            let mut bytes = [0; N];
            let mut index = 0;

            while let Some(item) = seq.try_decode_next()? {
                if index <= N {
                    bytes[index] = item.decode_u8()?;
                }

                index += 1;
            }

            if index != N {
                return Err(cx.message_at(
                    &mark,
                    format_args!(
                        "Array with length {index} does not have the expected {N} number of elements"
                    ),
                ));
            }

            Ok(bytes)
        })
    }

    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, [u8], Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;

        self.decode_sequence(|seq| {
            let mut bytes = Vec::with_capacity_in(seq.size_hint().or_default(), cx.alloc())
                .map_err(cx.map())?;

            while let Some(item) = seq.try_decode_next()? {
                let b = item.decode_u8()?;
                bytes.push(b).map_err(cx.map())?;
            }

            visitor.visit_owned(cx, bytes)
        })
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let mut scratch = Vec::new_in(self.cx.alloc());

        match self.parser.parse_string(self.cx, true, &mut scratch)? {
            StringReference::Borrowed(borrowed) => visitor.visit_borrowed(self.cx, borrowed),
            StringReference::Scratch(string) => visitor.visit_ref(self.cx, string),
        }
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        if self.parser.lex(self.cx).is_null() {
            self.parse_null()?;
            Ok(None)
        } else {
            Ok(Some(self))
        }
    }

    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonSequenceDecoder::new(self.cx, None, self.parser)?;
        let output = f(&mut decoder)?;
        decoder.skip_sequence_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonSequenceDecoder::new(self.cx, None, self.parser)?;
        let output = f(&mut decoder)?;
        decoder.skip_sequence_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_sequence_hint<F, O>(self, hint: impl SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        let size = hint.get();
        let mut decoder = JsonSequenceDecoder::new(self.cx, size, self.parser)?;
        let output = f(&mut decoder)?;
        decoder.skip_sequence_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonObjectDecoder::new(self.cx, None, self.parser)?;
        let output = f(&mut decoder)?;
        decoder.skip_object_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_map_hint<F, O>(self, hint: impl MapHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonObjectDecoder::new(self.cx, hint.get(), self.parser)?;
        let output = f(&mut decoder)?;
        decoder.skip_object_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_variant<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonVariantDecoder::new(self.cx, self.parser)?;
        let output = f(&mut decoder)?;
        decoder.end()?;
        Ok(output)
    }

    #[inline]
    fn decode_number<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        self.parser.parse_number(self.cx, visitor)
    }

    #[inline]
    fn decode_any<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;

        match self.parser.lex(cx) {
            Token::OpenBrace => self.decode_map(|decoder| visitor.visit_map(decoder)),
            Token::OpenBracket => self.decode_sequence(|decoder| visitor.visit_sequence(decoder)),
            Token::String => {
                let visitor = visitor.visit_string(cx, SizeHint::any())?;
                self.decode_string(visitor)
            }
            Token::Number => self.decode_number(visitor),
            Token::Null => {
                self.parse_null()?;
                visitor.visit_empty(cx)
            }
            Token::True => {
                self.parse_true()?;
                visitor.visit_bool(cx, true)
            }
            Token::False => {
                self.parse_false()?;
                visitor.visit_bool(cx, false)
            }
            token => Err(cx.message(format_args!("Expected value, found {token:?}"))),
        }
    }
}
