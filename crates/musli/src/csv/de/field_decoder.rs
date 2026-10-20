use core::fmt;
use core::marker::PhantomData;
use core::str::FromStr;

use crate::Context;
use crate::alloc::Vec;
use crate::de::{
    Decoder, EntriesDecoder, MapDecoder, SequenceDecoder, SizeHint, Skip, UnsizedVisitor,
    VariantDecoder, Visitor,
};
use crate::hint::{MapHint, SequenceHint};

use super::super::encoding::Config;
use super::super::parser::{self, Field, Parser};

const NESTED: &str = "CSV fields cannot contain containers such as maps, sequences or structs";
const VARIANT_DATA: &str = "Only unit variants can be stored in CSV fields";

/// A decoder for a single field.
pub(crate) struct FieldDecoder<P, C, M> {
    cx: C,
    parser: P,
    cfg: Config,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> FieldDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(crate) fn new(cx: C, parser: P, cfg: Config) -> Self {
        Self {
            cx,
            parser,
            cfg,
            _marker: PhantomData,
        }
    }

    /// Parse the field as a string and process it with `f`.
    #[inline]
    fn with_str<O>(
        mut self,
        f: impl FnOnce(C, &C::Mark, &str) -> Result<O, C::Error>,
    ) -> Result<O, C::Error> {
        let cx = self.cx;
        let mark = cx.mark();
        let mut scratch = Vec::new_in(cx.alloc());
        let field = parser::parse_field(cx, &mut self.parser, &self.cfg, &mut scratch)?;

        let string = crate::str::from_utf8(field.as_bytes())
            .map_err(|error| cx.custom_at(&mark, error))?;

        f(cx, &mark, string)
    }

    /// Parse the field using [`FromStr`].
    #[inline]
    fn parse<T>(self, what: &'static str) -> Result<T, C::Error>
    where
        T: FromStr,
    {
        self.with_str(|cx, mark, string| match string.parse() {
            Ok(value) => Ok(value),
            Err(..) => Err(cx.message_at(mark, Invalid { what, string })),
        })
    }
}

/// Error raised when a field cannot be parsed.
struct Invalid<'a> {
    what: &'static str,
    string: &'a str,
}

impl fmt::Display for Invalid<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expected {}, but found {:?}", self.what, self.string)
    }
}

#[crate::trait_defaults(crate)]
impl<'de, P, C, M> Decoder<'de> for FieldDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeSome = Self;
    type DecodeVariant = FieldVariantDecoder<P, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a CSV field")
    }

    #[inline]
    fn skip(mut self) -> Result<(), Self::Error> {
        parser::skip_field(self.cx, &mut self.parser, &self.cfg)
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        self.skip()?;
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        self.with_str(|cx, mark, string| {
            if !string.is_empty() {
                return Err(cx.message_at(mark, Invalid {
                    what: "an empty field",
                    string,
                }));
            }

            Ok(())
        })
    }

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        self.with_str(|cx, mark, string| match string {
            "true" => Ok(true),
            "false" => Ok(false),
            string => Err(cx.message_at(mark, Invalid {
                what: "a boolean",
                string,
            })),
        })
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        self.with_str(|cx, mark, string| {
            let mut it = string.chars();

            match (it.next(), it.next()) {
                (Some(c), None) => Ok(c),
                _ => Err(cx.message_at(mark, Invalid {
                    what: "a single character",
                    string,
                })),
            }
        })
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.parse("an unsigned 8-bit integer")
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.parse("an unsigned 16-bit integer")
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.parse("an unsigned 32-bit integer")
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.parse("an unsigned 64-bit integer")
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.parse("an unsigned 128-bit integer")
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.parse("a signed 8-bit integer")
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.parse("a signed 16-bit integer")
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.parse("a signed 32-bit integer")
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.parse("a signed 64-bit integer")
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.parse("a signed 128-bit integer")
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.parse("an unsigned pointer-sized integer")
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.parse("a signed pointer-sized integer")
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.parse("a 32-bit float")
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        self.parse("a 64-bit float")
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;
        let mark = cx.mark();
        let mut scratch = Vec::new_in(cx.alloc());

        match parser::parse_field(cx, &mut self.parser, &self.cfg, &mut scratch)? {
            Field::Borrowed(bytes) => {
                let string =
                    crate::str::from_utf8(bytes).map_err(|error| cx.custom_at(&mark, error))?;
                visitor.visit_borrowed(cx, string)
            }
            Field::Scratch(bytes) => {
                let string =
                    crate::str::from_utf8(bytes).map_err(|error| cx.custom_at(&mark, error))?;
                visitor.visit_ref(cx, string)
            }
        }
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        if parser::peek_empty(&mut self.parser, &self.cfg, false).is_some() {
            self.skip()?;
            return Ok(None);
        }

        Ok(Some(self))
    }

    #[inline]
    fn decode_pack<F, O>(self, _: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn decode_sequence<F, O>(self, _: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn decode_sequence_hint<F, O>(self, _: impl SequenceHint, _: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn decode_map<F, O>(self, _: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn decode_map_hint<F, O>(self, _: impl MapHint, _: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn decode_map_entries<F, O>(self, _: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn decode_variant<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        let start = self.parser.state().fields;

        let mut decoder = FieldVariantDecoder {
            cx: self.cx,
            parser: self.parser,
            cfg: self.cfg,
            _marker: PhantomData,
        };

        let output = f(&mut decoder)?;

        // Skip the tag if it wasn't used.
        if decoder.parser.state().fields == start {
            parser::skip_field(decoder.cx, &mut decoder.parser, &decoder.cfg)?;
        }

        Ok(output)
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let visitor = visitor.visit_string(self.cx, SizeHint::any())?;
        self.decode_string(visitor)
    }
}

/// A decoder for a unit variant stored in a field.
pub(crate) struct FieldVariantDecoder<P, C, M> {
    cx: C,
    parser: P,
    cfg: Config,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> VariantDecoder<'de> for FieldVariantDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeTag<'this>
        = FieldDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = EmptyDecoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error> {
        Ok(FieldDecoder::new(self.cx, self.parser.borrow_mut(), self.cfg))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error> {
        Ok(EmptyDecoder::new(self.cx))
    }
}

/// A decoder for the data of a unit variant, which is always empty.
pub(crate) struct EmptyDecoder<C, M> {
    cx: C,
    _marker: PhantomData<M>,
}

impl<C, M> EmptyDecoder<C, M> {
    #[inline]
    pub(crate) fn new(cx: C) -> Self {
        Self {
            cx,
            _marker: PhantomData,
        }
    }
}

#[crate::trait_defaults(crate)]
impl<'de, C, M> Decoder<'de> for EmptyDecoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodePack = Self;
    type DecodeSequence = Self;
    type DecodeMap = Self;
    type DecodeMapEntries = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{VARIANT_DATA}")
    }

    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn decode_pack<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        f(&mut self)
    }

    #[inline]
    fn decode_sequence<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        f(&mut self)
    }

    #[inline]
    fn decode_sequence_hint<F, O>(mut self, _: impl SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        f(&mut self)
    }

    #[inline]
    fn decode_map<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        f(&mut self)
    }

    #[inline]
    fn decode_map_hint<F, O>(mut self, _: impl MapHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        f(&mut self)
    }

    #[inline]
    fn decode_map_entries<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        f(&mut self)
    }
}

impl<'de, C, M> SequenceDecoder<'de> for EmptyDecoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeNext<'this>
        = EmptyDecoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::exact(0)
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, Self::Error> {
        Ok(None)
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        Err(self.cx.message(VARIANT_DATA))
    }
}

impl<'de, C, M> MapDecoder<'de> for EmptyDecoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntry<'this>
        = crate::__priv::Never<(C, M)>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = EmptyDecoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::exact(0)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, Self::Error> {
        Ok(None)
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, Self::Error> {
        Ok(EmptyDecoder::new(self.cx))
    }
}

impl<'de, C, M> EntriesDecoder<'de> for EmptyDecoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntryKey<'this>
        = EmptyDecoder<C, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = EmptyDecoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error> {
        Ok(None)
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        Err(self.cx.message(VARIANT_DATA))
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
