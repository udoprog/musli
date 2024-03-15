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

use crate::error::{Error, ErrorKind};
#[cfg(not(feature = "parse-full"))]
use crate::reader::integer::{
    parse_signed_base as parse_signed, parse_unsigned_base as parse_unsigned,
};
#[cfg(feature = "parse-full")]
use crate::reader::integer::{
    parse_signed_full as parse_signed, parse_unsigned_full as parse_unsigned,
};
use crate::reader::{integer, string, Parser, StringReference, Token};

#[cfg(feature = "musli-value")]
const BUFFER_OPTIONS: musli_common::options::Options = musli_common::options::new().build();

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
        C: Context<Input = Error>,
    {
        let start = cx.mark();
        let actual = self.parser.peek(cx)?;

        match actual {
            Token::OpenBrace => {
                let mut object = JsonObjectDecoder::new(cx, None, self.parser)?;

                while let Some(mut pair) = object.entry(cx)? {
                    pair.map_key(cx)?.skip_any(cx)?;
                    pair.skip_map_value(cx)?;
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
    fn parse_true<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        self.parser
            .parse_exact(cx, *b"true", ErrorKind::ExpectedTrue)
    }

    #[inline]
    fn parse_false<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        self.parser
            .parse_exact(cx, *b"false", ErrorKind::ExpectedFalse)
    }

    #[inline]
    fn parse_null<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        self.parser
            .parse_exact(cx, *b"null", ErrorKind::ExpectedNull)
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
    type MapPairs = JsonObjectDecoder<P>;
    type Some = JsonDecoder<P>;
    type Struct = JsonObjectDecoder<P>;
    type StructPairs = JsonObjectDecoder<P>;
    type Variant = JsonVariantDecoder<P>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSON")
    }

    #[inline]
    fn type_hint<C>(&mut self, cx: &C) -> Result<TypeHint, C::Error>
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
    fn decode_buffer<C>(self, cx: &C) -> Result<Self::Buffer, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        use musli::de::Decode;
        let value = musli_value::Value::decode(cx, self)?;
        Ok(value.into_value_decoder())
    }

    #[inline]
    fn decode_unit<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)
    }

    #[inline]
    fn decode_bool<C>(mut self, cx: &C) -> Result<bool, C::Error>
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
    fn decode_char<C>(mut self, cx: &C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
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
            _ => Err(cx.marked_report(start, Error::new(ErrorKind::CharEmptyString))),
        }
    }

    #[inline]
    fn decode_u8<C>(mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u16<C>(mut self, cx: &C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u32<C>(mut self, cx: &C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u64<C>(mut self, cx: &C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_u128<C>(mut self, cx: &C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i8<C>(mut self, cx: &C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i16<C>(mut self, cx: &C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i32<C>(mut self, cx: &C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i64<C>(mut self, cx: &C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_i128<C>(mut self, cx: &C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_usize<C>(mut self, cx: &C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_unsigned(cx, &mut self.parser)
    }

    #[inline]
    fn decode_isize<C>(mut self, cx: &C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        parse_signed(cx, &mut self.parser)
    }

    #[inline]
    fn decode_f32<C>(mut self, cx: &C) -> Result<f32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.parser.parse_f32(cx)
    }

    #[inline]
    fn decode_f64<C>(mut self, cx: &C) -> Result<f64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.parser.parse_f64(cx)
    }

    #[inline]
    fn decode_array<C, const N: usize>(self, cx: &C) -> Result<[u8; N], C::Error>
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
    fn decode_number<C, V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: NumberVisitor<'de, C>,
    {
        self.parser.parse_number(cx, visitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<C, V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
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
    fn decode_string<C, V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
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
    fn decode_option<C>(mut self, cx: &C) -> Result<Option<Self::Some>, C::Error>
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
    fn decode_pack<C>(self, cx: &C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_sequence<C>(self, cx: &C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_tuple<C>(self, cx: &C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonSequenceDecoder::new(cx, Some(len), self.parser)
    }

    #[inline]
    fn decode_map<C>(self, cx: &C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_map_pairs<C>(self, cx: &C) -> Result<Self::MapPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, None, self.parser)
    }

    #[inline]
    fn decode_struct<C>(self, cx: &C, len: Option<usize>) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, len, self.parser)
    }

    #[inline]
    fn decode_struct_pairs<C>(
        self,
        cx: &C,
        len: Option<usize>,
    ) -> Result<Self::StructPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonObjectDecoder::new(cx, len, self.parser)
    }

    #[inline]
    fn decode_variant<C>(self, cx: &C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        JsonVariantDecoder::new(cx, self.parser)
    }

    #[inline]
    fn decode_any<C, V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
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
