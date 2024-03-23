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
    Decode, Decoder, MapDecoder, MapEntryDecoder, NumberHint, NumberVisitor, SequenceDecoder,
    SizeHint, TypeHint, ValueVisitor, Visitor,
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
pub(crate) struct JsonDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    parser: P,
}

impl<'a, 'de, P, C> JsonDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, parser: P) -> Self {
        Self { cx, parser }
    }

    /// Skip over any values.
    pub(crate) fn skip_any(mut self) -> Result<(), C::Error> {
        let start = self.cx.mark();
        let actual = self.parser.peek(self.cx)?;

        match actual {
            Token::OpenBrace => {
                let mut object = JsonObjectDecoder::new(self.cx, None, self.parser)?;

                while let Some(mut pair) = object.decode_entry()? {
                    pair.decode_map_key()?.skip()?;
                    pair.skip_map_value()?;
                }

                Ok(())
            }
            Token::OpenBracket => {
                let mut seq = JsonSequenceDecoder::new(self.cx, None, self.parser)?;

                while let Some(item) = SequenceDecoder::decode_next(&mut seq)? {
                    item.skip_any()?;
                }

                Ok(())
            }
            Token::Null => self.parse_null(),
            Token::True => self.parse_true(),
            Token::False => self.parse_false(),
            Token::Number => integer::skip_number(self.cx, self.parser.borrow_mut()),
            Token::String => {
                // Skip over opening quote.
                self.parser.skip(self.cx, 1)?;
                string::skip_string(self.cx, self.parser.borrow_mut(), true)
            }
            actual => Err(self
                .cx
                .marked_message(start, format_args!("Expected value, found {actual}"))),
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

#[musli::decoder]
impl<'a, 'de, P, C> Decoder<'de> for JsonDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<'this, U> = JsonDecoder<'this, P, U> where U: 'this + Context;
    #[cfg(feature = "musli-value")]
    type DecodeBuffer = musli_value::AsValueDecoder<'a, BUFFER_OPTIONS, C>;
    type DecodePack = JsonSequenceDecoder<'a, P, C>;
    type DecodeSequence = JsonSequenceDecoder<'a, P, C>;
    type DecodeTuple = JsonSequenceDecoder<'a, P, C>;
    type DecodeMap = JsonObjectDecoder<'a, P, C>;
    type DecodeSome = JsonDecoder<'a, P, C>;
    type DecodeStruct = JsonObjectDecoder<'a, P, C>;
    type DecodeVariant = JsonVariantDecoder<'a, P, C>;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(JsonDecoder::new(cx, self.parser))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSON")
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode>,
    {
        T::decode(self.cx, self)
    }

    #[inline]
    fn type_hint(&mut self) -> Result<TypeHint, C::Error> {
        Ok(match self.parser.peek(self.cx)? {
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
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, C::Error> {
        let cx = self.cx;
        let value = self.decode::<musli_value::Value>()?;
        Ok(value.into_value_decoder(cx))
    }

    #[inline]
    fn decode_unit(self) -> Result<(), C::Error> {
        self.skip_any()
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, C::Error> {
        match self.parser.peek(self.cx)? {
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
    fn decode_char(mut self) -> Result<char, C::Error> {
        let start = self.cx.mark();

        let Some(mut scratch) = self.cx.alloc() else {
            return Err(self.cx.message("Failed to allocate scratch buffer"));
        };

        let string = match self.parser.parse_string(self.cx, true, &mut scratch)? {
            StringReference::Borrowed(string) => string,
            StringReference::Scratch(string) => string,
        };

        let mut it = string.chars();
        let first = it.next();

        match (first, it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(self
                .cx
                .marked_message(start, "Expected string with a single character")),
        }
    }

    #[inline]
    fn decode_u8(mut self) -> Result<u8, C::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u16(mut self) -> Result<u16, C::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u32(mut self) -> Result<u32, C::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u64(mut self) -> Result<u64, C::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_u128(mut self) -> Result<u128, C::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i8(mut self) -> Result<i8, C::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i16(mut self) -> Result<i16, C::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i32(mut self) -> Result<i32, C::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i64(mut self) -> Result<i64, C::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_i128(mut self) -> Result<i128, C::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_usize(mut self) -> Result<usize, C::Error> {
        parse_unsigned(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_isize(mut self) -> Result<isize, C::Error> {
        parse_signed(self.cx, self.parser.borrow_mut())
    }

    #[inline]
    fn decode_f32(mut self) -> Result<f32, C::Error> {
        self.parser.parse_f32(self.cx)
    }

    #[inline]
    fn decode_f64(mut self) -> Result<f64, C::Error> {
        self.parser.parse_f64(self.cx)
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], C::Error> {
        let cx = self.cx;
        let mark = cx.mark();
        let mut seq = self.decode_sequence()?;
        let mut bytes = [0; N];
        let mut index = 0;

        while let Some(item) = SequenceDecoder::decode_next(&mut seq)? {
            if index <= N {
                bytes[index] = item.decode_u8()?;
            }

            index += 1;
        }

        if index != N {
            return Err(cx.marked_message(
                mark,
                format_args!(
                    "Array with length {index} does not have the expected {N} number of elements"
                ),
            ));
        }

        Ok(bytes)
    }

    #[inline]
    fn decode_number<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: NumberVisitor<'de, C>,
    {
        self.parser.parse_number(self.cx, visitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let cx = self.cx;
        let mut seq = self.decode_sequence()?;
        let mut bytes = Vec::with_capacity(seq.size_hint().or_default());

        while let Some(item) = SequenceDecoder::decode_next(&mut seq)? {
            bytes.push(item.decode_u8()?);
        }

        visitor.visit_owned(cx, bytes)
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
    {
        let Some(mut scratch) = self.cx.alloc() else {
            return Err(self.cx.message("Failed to allocate scratch buffer"));
        };

        match self.parser.parse_string(self.cx, true, &mut scratch)? {
            StringReference::Borrowed(borrowed) => visitor.visit_borrowed(self.cx, borrowed),
            StringReference::Scratch(string) => visitor.visit_ref(self.cx, string),
        }
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, C::Error> {
        if self.parser.peek(self.cx)?.is_null() {
            self.parse_null()?;
            Ok(None)
        } else {
            Ok(Some(self))
        }
    }

    #[inline]
    fn decode_pack(self) -> Result<Self::DecodePack, C::Error> {
        JsonSequenceDecoder::new(self.cx, None, self.parser)
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::DecodeSequence, C::Error> {
        JsonSequenceDecoder::new(self.cx, None, self.parser)
    }

    #[inline]
    fn decode_tuple(self, len: usize) -> Result<Self::DecodeTuple, C::Error> {
        JsonSequenceDecoder::new(self.cx, Some(len), self.parser)
    }

    #[inline]
    fn decode_map(self) -> Result<Self::DecodeMap, C::Error> {
        JsonObjectDecoder::new(self.cx, None, self.parser)
    }

    #[inline]
    fn decode_struct(self, len: Option<usize>) -> Result<Self::DecodeStruct, C::Error> {
        JsonObjectDecoder::new(self.cx, len, self.parser)
    }

    #[inline]
    fn decode_variant(self) -> Result<Self::DecodeVariant, C::Error> {
        JsonVariantDecoder::new(self.cx, self.parser)
    }

    #[inline]
    fn decode_any<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, C>,
    {
        let cx = self.cx;
        self.parser.skip_whitespace(cx)?;

        match self.parser.peek(cx)? {
            Token::OpenBrace => {
                let decoder = JsonObjectDecoder::new(self.cx, None, self.parser)?;
                visitor.visit_map(cx, decoder)
            }
            Token::OpenBracket => {
                let decoder = JsonSequenceDecoder::new(self.cx, None, self.parser)?;
                visitor.visit_sequence(cx, decoder)
            }
            Token::String => {
                let visitor = visitor.visit_string(cx, SizeHint::Any)?;
                self.decode_string(visitor)
            }
            Token::Number => {
                let visitor = visitor.visit_number(cx, NumberHint::Any)?;
                self.decode_number(visitor)
            }
            Token::Null => {
                self.parse_null()?;
                visitor.visit_unit(cx)
            }
            Token::True => {
                self.parse_true()?;
                visitor.visit_bool(cx, true)
            }
            Token::False => {
                self.parse_false()?;
                visitor.visit_bool(cx, false)
            }
            _ => visitor.visit_any(cx, self, TypeHint::Any),
        }
    }
}
