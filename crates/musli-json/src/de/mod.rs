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
use core::str;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, MapDecoder, MapEntryDecoder, NumberHint, NumberVisitor, SequenceDecoder, SizeHint,
    TypeHint, ValueVisitor, Visitor,
};
use musli::Context;

#[cfg(not(feature = "parse-full"))]
use crate::parser::integer::{
    parse_signed_base as parse_signed, parse_unsigned_base as parse_unsigned,
};
#[cfg(feature = "parse-full")]
use crate::parser::integer::{
    parse_signed_full as parse_signed, parse_unsigned_full as parse_unsigned,
};
use crate::parser::{integer, string, Parser, StringReference, Token};

#[cfg(feature = "musli-value")]
const BUFFER_OPTIONS: crate::options::Options = crate::options::new().build();

/// A JSON decoder for Müsli.
pub(crate) struct JsonDecoder<P> {
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
    pub(crate) fn skip_any<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        let start = cx.mark();
        let actual = self.parser.peek(cx)?;

        match actual {
            Token::OpenBrace => {
                let mut object = JsonObjectDecoder::new(cx, None, self.parser)?;

                while let Some(mut pair) = object.decode_entry(cx)? {
                    pair.decode_map_key(cx)?.skip_any(cx)?;
                    pair.skip_map_value(cx)?;
                }

                Ok(())
            }
            Token::OpenBracket => {
                let mut seq = JsonSequenceDecoder::new(cx, None, self.parser)?;

                while let Some(item) = SequenceDecoder::decode_next(&mut seq, cx)? {
                    item.skip_any(cx)?;
                }

                Ok(())
            }
            Token::Null => self.parse_null(cx),
            Token::True => self.parse_true(cx),
            Token::False => self.parse_false(cx),
            Token::Number => integer::skip_number(cx, self.parser.borrow_mut()),
            Token::String => {
                // Skip over opening quote.
                self.parser.skip(cx, 1)?;
                string::skip_string(cx, self.parser.borrow_mut(), true)
            }
            actual => Err(cx.marked_message(start, format_args!("Expected value, found {actual}"))),
        }
    }

    #[inline]
    fn parse_true<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.parser.parse_exact(cx, "true")
    }

    #[inline]
    fn parse_false<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.parser.parse_exact(cx, "false")
    }

    #[inline]
    fn parse_null<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.parser.parse_exact(cx, "null")
    }
}

#[musli::decoder]
impl<'de, C, P> Decoder<'de, C> for JsonDecoder<P>
where
    C: ?Sized + Context,
    P: Parser<'de>,
{
    type WithContext<U> = Self where U: Context;
    #[cfg(feature = "musli-value")]
    type DecodeBuffer = musli_value::AsValueDecoder<BUFFER_OPTIONS>;
    type DecodePack = JsonSequenceDecoder<P>;
    type DecodeSequence = JsonSequenceDecoder<P>;
    type DecodeTuple = JsonSequenceDecoder<P>;
    type DecodeMap = JsonObjectDecoder<P>;
    type DecodeSome = JsonDecoder<P>;
    type DecodeStruct = JsonObjectDecoder<P>;
    type DecodeVariant = JsonVariantDecoder<P>;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSON")
    }

    #[inline]
    fn type_hint(&mut self, cx: &C) -> Result<TypeHint, C::Error> {
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
    fn decode_buffer(self, cx: &C) -> Result<Self::DecodeBuffer, C::Error> {
        use musli::de::Decode;
        let value = musli_value::Value::decode(cx, self)?;
        Ok(value.into_value_decoder())
    }

    #[inline]
    fn decode_unit(self, cx: &C) -> Result<(), C::Error> {
        self.skip_any(cx)
    }

    #[inline]
    fn decode_bool(mut self, cx: &C) -> Result<bool, C::Error> {
        match self.parser.peek(cx)? {
            Token::True => {
                self.parse_true(cx)?;
                Ok(true)
            }
            Token::False => {
                self.parse_false(cx)?;
                Ok(false)
            }
            actual => Err(cx.message(format_args!("Expected boolean, was {actual}"))),
        }
    }

    #[inline]
    fn decode_char(mut self, cx: &C) -> Result<char, C::Error> {
        let start = cx.mark();

        let Some(mut scratch) = cx.alloc() else {
            return Err(cx.message("Failed to allocate scratch buffer"));
        };

        let string = match self.parser.parse_string(cx, true, &mut scratch)? {
            StringReference::Borrowed(string) => string,
            StringReference::Scratch(string) => string,
        };

        let mut it = string.chars();
        let first = it.next();

        match (first, it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(cx.marked_message(start, "Expected string with a single character")),
        }
    }

    #[inline]
    fn decode_u8(mut self, cx: &C) -> Result<u8, C::Error> {
        parse_unsigned(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u16(mut self, cx: &C) -> Result<u16, C::Error> {
        parse_unsigned(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u32(mut self, cx: &C) -> Result<u32, C::Error> {
        parse_unsigned(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u64(mut self, cx: &C) -> Result<u64, C::Error> {
        parse_unsigned(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u128(mut self, cx: &C) -> Result<u128, C::Error> {
        parse_unsigned(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i8(mut self, cx: &C) -> Result<i8, C::Error> {
        parse_signed(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i16(mut self, cx: &C) -> Result<i16, C::Error> {
        parse_signed(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i32(mut self, cx: &C) -> Result<i32, C::Error> {
        parse_signed(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i64(mut self, cx: &C) -> Result<i64, C::Error> {
        parse_signed(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i128(mut self, cx: &C) -> Result<i128, C::Error> {
        parse_signed(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_usize(mut self, cx: &C) -> Result<usize, C::Error> {
        parse_unsigned(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_isize(mut self, cx: &C) -> Result<isize, C::Error> {
        parse_signed(cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_f32(mut self, cx: &C) -> Result<f32, C::Error> {
        self.parser.parse_f32(cx)
    }

    #[inline]
    fn decode_f64(mut self, cx: &C) -> Result<f64, C::Error> {
        self.parser.parse_f64(cx)
    }

    #[inline]
    fn decode_array<const N: usize>(self, cx: &C) -> Result<[u8; N], C::Error> {
        let mut seq = self.decode_sequence(cx)?;
        let mut bytes = [0; N];
        let mut index = 0;

        while let Some(item) = SequenceDecoder::decode_next(&mut seq, cx)? {
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
    fn decode_number<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: NumberVisitor<'de, C>,
    {
        self.parser.parse_number(cx, visitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let mut seq = self.decode_sequence(cx)?;
        let mut bytes = Vec::with_capacity(seq.size_hint(cx).or_default());

        while let Some(item) = SequenceDecoder::decode_next(&mut seq, cx)? {
            bytes.push(item.decode_u8(cx)?);
        }

        visitor.visit_owned(cx, bytes)
    }

    #[inline]
    fn decode_string<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
    {
        let Some(mut scratch) = cx.alloc() else {
            return Err(cx.message("Failed to allocate scratch buffer"));
        };

        match self.parser.parse_string(cx, true, &mut scratch)? {
            StringReference::Borrowed(borrowed) => visitor.visit_borrowed(cx, borrowed),
            StringReference::Scratch(string) => visitor.visit_ref(cx, string),
        }
    }

    #[inline]
    fn decode_option(mut self, cx: &C) -> Result<Option<Self::DecodeSome>, C::Error> {
        if self.parser.peek(cx)?.is_null() {
            self.parse_null(cx)?;
            Ok(None)
        } else {
            Ok(Some(self))
        }
    }

    #[inline]
    fn decode_pack(self, cx: &C) -> Result<Self::DecodePack, C::Error> {
        JsonSequenceDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_sequence(self, cx: &C) -> Result<Self::DecodeSequence, C::Error> {
        JsonSequenceDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_tuple(self, cx: &C, len: usize) -> Result<Self::DecodeTuple, C::Error> {
        JsonSequenceDecoder::new(cx, Some(len), self.parser)
    }

    #[inline]
    fn decode_map(self, cx: &C) -> Result<Self::DecodeMap, C::Error> {
        JsonObjectDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_struct(self, cx: &C, len: Option<usize>) -> Result<Self::DecodeStruct, C::Error> {
        JsonObjectDecoder::new(cx, len, self.parser)
    }

    #[inline]
    fn decode_variant(self, cx: &C) -> Result<Self::DecodeVariant, C::Error> {
        JsonVariantDecoder::new(cx, self.parser)
    }

    #[inline]
    fn decode_any<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, C>,
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
