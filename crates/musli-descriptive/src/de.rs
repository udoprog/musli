use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, NumberHint, NumberVisitor, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder,
    SizeHint, TypeHint, ValueVisitor, VariantDecoder, Visitor,
};
use musli::Context;
use musli_common::int::{continuation as c, UsizeEncoding, Variable};
use musli_common::reader::PosReader;
use musli_storage::de::StorageDecoder;

use crate::integer_encoding::{decode_typed_signed, decode_typed_unsigned};
use crate::tag::{Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, U128, U16, U32, U64, U8};

/// A very simple decoder.
pub struct SelfDecoder<R> {
    reader: R,
}

impl<R> SelfDecoder<R> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }
}

pub struct SelfPackDecoder<R> {
    reader: R,
    remaining: usize,
}

impl<R> SelfPackDecoder<R> {
    #[inline]
    pub(crate) fn new(reader: R, end: usize) -> Self {
        Self {
            reader,
            remaining: end,
        }
    }
}

pub struct SelfTupleDecoder<R> {
    reader: R,
}

impl<R> SelfTupleDecoder<R> {
    #[inline]
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<'de, R> SelfDecoder<R>
where
    R: PosReader<'de>,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any<C>(&mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = R::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Number => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, _, u128>(cx, self.reader.borrow_mut())?;
                }
            }
            Kind::Mark => {
                if let Mark::Variant = tag.mark() {
                    self.skip_any(cx)?;
                    self.skip_any(cx)?;
                }
            }
            Kind::Bytes => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    Variable::decode_usize(cx, self.reader.borrow_mut())?
                };

                self.reader.skip(cx, len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    Variable::decode_usize(cx, self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any(cx)?;
                }
            }
            Kind::Map => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    Variable::decode_usize(cx, self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any(cx)?;
                    self.skip_any(cx)?;
                }
            }
            kind => {
                return Err(cx.message(format_args!("unsupported kind {kind:?}")));
            }
        }

        Ok(())
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_map<C>(mut self, cx: &mut C) -> Result<RemainingSelfDecoder<R>, C::Error>
    where
        C: Context<Input = R::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, Kind::Map, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence<C>(mut self, cx: &mut C) -> Result<RemainingSelfDecoder<R>, C::Error>
    where
        C: Context<Input = R::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, Kind::Sequence, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix<C>(&mut self, cx: &mut C, kind: Kind, pos: usize) -> Result<usize, C::Error>
    where
        C: Context<Input = R::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag.kind() != kind {
            return Err(cx.message(Expected {
                expected: kind,
                actual: tag,
                pos,
            }));
        }

        Ok(if let Some(len) = tag.data() {
            len as usize
        } else {
            Variable::decode_usize(cx, self.reader.borrow_mut())?
        })
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct RemainingSelfDecoder<R> {
    remaining: usize,
    decoder: SelfDecoder<R>,
}

#[musli::decoder]
impl<'de, R> Decoder<'de> for SelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Pack = SelfPackDecoder<R>;
    type Some = Self;
    type Sequence = RemainingSelfDecoder<R>;
    type Tuple = SelfTupleDecoder<R>;
    type Map = RemainingSelfDecoder<R>;
    type Struct = RemainingSelfDecoder<R>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the descriptive decoder")
    }

    #[inline]
    fn type_hint<C>(&mut self, cx: &mut C) -> Result<TypeHint, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let tag = match self.reader.peek(cx)? {
            Some(b) => Tag::from_byte(b),
            None => return Ok(TypeHint::Any),
        };

        match tag.kind() {
            Kind::Number => Ok(TypeHint::Number(match tag.data() {
                Some(U8) => NumberHint::U8,
                Some(U16) => NumberHint::U16,
                Some(U32) => NumberHint::U32,
                Some(U64) => NumberHint::U64,
                Some(U128) => NumberHint::U128,
                Some(I8) => NumberHint::I8,
                Some(I16) => NumberHint::I16,
                Some(I32) => NumberHint::I32,
                Some(I64) => NumberHint::I64,
                Some(I128) => NumberHint::I128,
                Some(F32) => NumberHint::F32,
                Some(F64) => NumberHint::F64,
                _ => NumberHint::Any,
            })),
            Kind::Sequence => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Sequence(hint))
            }
            Kind::Map => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Map(hint))
            }
            Kind::Bytes => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Bytes(hint))
            }
            Kind::String => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::String(hint))
            }
            Kind::Mark => Ok(match tag.mark() {
                Mark::True | Mark::False => TypeHint::Bool,
                Mark::Variant => TypeHint::Variant,
                Mark::Some | Mark::None => TypeHint::Option,
                Mark::Char => TypeHint::Char,
                Mark::Unit => TypeHint::Unit,
                _ => TypeHint::Any,
            }),
            _ => Ok(TypeHint::Any),
        }
    }

    #[inline]
    fn decode_unit<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(())
    }

    #[inline]
    fn decode_pack<C>(mut self, cx: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, Kind::Bytes, pos)?;
        Ok(SelfPackDecoder::new(self.reader, len))
    }

    #[inline]
    fn decode_array<C, const N: usize>(mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, Kind::Bytes, pos)?;

        if len != N {
            return Err(cx.message(format_args! {
                "bad length, got {len} but expect {N} (at {pos})"
            }));
        }

        self.reader.read_array(cx)
    }

    #[inline]
    fn decode_bytes<V>(
        mut self,
        cx: &mut V::Context,
        visitor: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, Kind::Bytes, pos)?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(
        mut self,
        cx: &mut V::Context,
        visitor: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        struct Visitor<V>(V);

        impl<'de, V> ValueVisitor<'de> for Visitor<V>
        where
            V: ValueVisitor<'de, Target = str>,
        {
            type Target = [u8];
            type Ok = V::Ok;
            type Error = V::Error;
            type Context = V::Context;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline]
            fn visit_owned(
                self,
                cx: &mut Self::Context,
                bytes: Vec<u8>,
            ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
                let string =
                    musli_common::str::from_utf8_owned(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_owned(cx, string)
            }

            #[inline]
            fn visit_borrowed(
                self,
                cx: &mut Self::Context,
                bytes: &'de [u8],
            ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline]
            fn visit_ref(
                self,
                cx: &mut Self::Context,
                bytes: &[u8],
            ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_ref(cx, string)
            }
        }

        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, Kind::String, pos)?;
        self.reader.read_bytes(cx, len, Visitor(visitor))
    }

    #[inline]
    fn decode_bool<C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const FALSE: Tag = Tag::from_mark(Mark::False);
        const TRUE: Tag = Tag::from_mark(Mark::True);

        let pos = self.reader.pos();
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(cx.message(format_args! {
                "bad boolean, got {tag:?} (at {pos})"
            })),
        }
    }

    #[inline]
    fn decode_char<C>(mut self, cx: &mut C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const CHAR: Tag = Tag::from_mark(Mark::Char);

        let pos = self.reader.pos();
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag != CHAR {
            return Err(cx.message(format_args!("expected {CHAR:?}, got {tag:?} (at {pos})")));
        }

        let num = c::decode(cx, self.reader.borrow_mut())?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.message(format_args!("bad character (at {pos}"))),
        }
    }

    #[inline]
    fn decode_number<V>(
        mut self,
        cx: &mut V::Context,
        visitor: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: NumberVisitor<'de, Error = Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Number => match tag.data() {
                Some(U8) => {
                    let value = self.decode_u8(cx)?;
                    visitor.visit_u8(cx, value)
                }
                Some(U16) => {
                    let value = self.decode_u16(cx)?;
                    visitor.visit_u16(cx, value)
                }
                Some(U32) => {
                    let value = self.decode_u32(cx)?;
                    visitor.visit_u32(cx, value)
                }
                Some(U64) => {
                    let value = self.decode_u64(cx)?;
                    visitor.visit_u64(cx, value)
                }
                Some(U128) => {
                    let value = self.decode_u128(cx)?;
                    visitor.visit_u128(cx, value)
                }
                Some(I8) => {
                    let value = self.decode_i8(cx)?;
                    visitor.visit_i8(cx, value)
                }
                Some(I16) => {
                    let value = self.decode_i16(cx)?;
                    visitor.visit_i16(cx, value)
                }
                Some(I32) => {
                    let value = self.decode_i32(cx)?;
                    visitor.visit_i32(cx, value)
                }
                Some(I64) => {
                    let value = self.decode_i64(cx)?;
                    visitor.visit_i64(cx, value)
                }
                Some(I128) => {
                    let value = self.decode_i128(cx)?;
                    visitor.visit_i128(cx, value)
                }
                Some(F32) => {
                    let value = self.decode_f32(cx)?;
                    visitor.visit_f32(cx, value)
                }
                Some(F64) => {
                    let value = self.decode_f64(cx)?;
                    visitor.visit_f64(cx, value)
                }
                _ => Err(cx.message(format_args!("unsupported number tag, got {tag:?}"))),
            },
            _ => Err(cx.message(format_args!("expected number, but got {tag:?}"))),
        }
    }

    #[inline]
    fn decode_u8<C>(self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u16<C>(self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u32<C>(self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u64<C>(self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u128<C>(self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_i8<C>(self, cx: &mut C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i16<C>(self, cx: &mut C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i32<C>(self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i64<C>(self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i128<C>(self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_usize<C>(self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_isize<C>(self, cx: &mut C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        decode_typed_signed(cx, self.reader)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32<C>(self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64<C>(self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option<C>(mut self, cx: &mut C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::from_mark(Mark::None);
        const SOME: Tag = Tag::from_mark(Mark::Some);

        let pos = self.reader.pos();
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(cx.message(format_args! {
                "expected option, was {tag:?} (at {pos})"
            })),
        }
    }

    #[inline]
    fn decode_sequence<C>(self, cx: &mut C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.shared_decode_sequence(cx)
    }

    #[inline]
    fn decode_tuple<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let actual = self.decode_prefix(cx, Kind::Sequence, pos)?;

        if len != actual {
            return Err(cx.message(format_args!(
                "tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(SelfTupleDecoder::new(self.reader))
    }

    #[inline]
    fn decode_map<C>(self, cx: &mut C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.shared_decode_map(cx)
    }

    #[inline]
    fn decode_struct<C>(self, cx: &mut C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.shared_decode_map(cx)
    }

    #[inline]
    fn decode_variant<C>(mut self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);

        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag != VARIANT {
            return Err(cx.message(Expected {
                expected: Kind::Mark,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        Ok(self)
    }

    #[inline]
    fn decode_any<C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: Visitor<'de, Error = Self::Error>,
    {
        let tag = match self.reader.peek(cx)? {
            Some(b) => Tag::from_byte(b),
            None => return visitor.visit_any(cx, self, TypeHint::Any),
        };

        match tag.kind() {
            Kind::Number => match tag.data() {
                Some(U8) => {
                    let value = self.decode_u8(cx)?;
                    visitor.visit_u8(cx, value)
                }
                Some(U16) => {
                    let value = self.decode_u16(cx)?;
                    visitor.visit_u16(cx, value)
                }
                Some(U32) => {
                    let value = self.decode_u32(cx)?;
                    visitor.visit_u32(cx, value)
                }
                Some(U64) => {
                    let value = self.decode_u64(cx)?;
                    visitor.visit_u64(cx, value)
                }
                Some(U128) => {
                    let value = self.decode_u128(cx)?;
                    visitor.visit_u128(cx, value)
                }
                Some(I8) => {
                    let value = self.decode_i8(cx)?;
                    visitor.visit_i8(cx, value)
                }
                Some(I16) => {
                    let value = self.decode_i16(cx)?;
                    visitor.visit_i16(cx, value)
                }
                Some(I32) => {
                    let value = self.decode_i32(cx)?;
                    visitor.visit_i32(cx, value)
                }
                Some(I64) => {
                    let value = self.decode_i64(cx)?;
                    visitor.visit_i64(cx, value)
                }
                Some(I128) => {
                    let value = self.decode_i128(cx)?;
                    visitor.visit_i128(cx, value)
                }
                Some(F32) => {
                    let value = self.decode_f32(cx)?;
                    visitor.visit_f32(cx, value)
                }
                Some(F64) => {
                    let value = self.decode_f64(cx)?;
                    visitor.visit_f64(cx, value)
                }
                _ => {
                    let visitor = visitor.visit_number(cx, NumberHint::Any)?;
                    visitor.visit_any(cx, self, TypeHint::Number(NumberHint::Any))
                }
            },
            Kind::Sequence => {
                let sequence = self.shared_decode_sequence(cx)?;
                visitor.visit_sequence(cx, sequence)
            }
            Kind::Map => {
                let map = self.shared_decode_map(cx)?;
                visitor.visit_map(cx, map)
            }
            Kind::Bytes => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_bytes(cx, hint)?;
                self.decode_bytes(cx, visitor)
            }
            Kind::String => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_string(cx, hint)?;
                self.decode_string(cx, visitor)
            }
            Kind::Mark => match tag.mark() {
                Mark::True | Mark::False => {
                    let value = self.decode_bool(cx)?;
                    visitor.visit_bool(cx, value)
                }
                Mark::Variant => {
                    let value = self.decode_variant(cx)?;
                    visitor.visit_variant(cx, value)
                }
                Mark::Some | Mark::None => {
                    let value = self.decode_option(cx)?;
                    visitor.visit_option(cx, value)
                }
                Mark::Char => {
                    let value = self.decode_char(cx)?;
                    visitor.visit_char(cx, value)
                }
                Mark::Unit => {
                    self.decode_unit(cx)?;
                    visitor.visit_unit(cx)
                }
                _ => visitor.visit_any(cx, self, TypeHint::Any),
            },
            _ => visitor.visit_any(cx, self, TypeHint::Any),
        }
    }
}

impl<'de, R> PackDecoder<'de> for SelfPackDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, Variable, Variable> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, cx: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.remaining = match self.remaining.checked_sub(1) {
            Some(remaining) => remaining,
            None => return Err(cx.message("tried to decode past the pack")),
        };

        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R> PackDecoder<'de> for SelfTupleDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Decoder<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R> RemainingSelfDecoder<R>
where
    R: PosReader<'de>,
{
    #[inline]
    fn new(remaining: usize, decoder: SelfDecoder<R>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'de, R> SequenceDecoder<'de> for RemainingSelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Decoder<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(SelfDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        // Skip remaining elements.
        while let Some(mut item) = SequenceDecoder::next(&mut self, cx)? {
            item.skip_any(cx)?;
        }

        Ok(())
    }
}

impl<'de, R> PairDecoder<'de> for SelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type First<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;
    type Second = Self;

    #[inline]
    fn first<C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn second<C>(self, _: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn skip_second<C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(true)
    }
}

impl<'de, R> VariantDecoder<'de> for SelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Tag<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;
    type Variant<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn skip_variant<C>(&mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R> PairsDecoder<'de> for RemainingSelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;

    type Decoder<'this> = SelfDecoder<R::PosMut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(SelfDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        // Skip remaining elements.
        while let Some(mut item) = PairsDecoder::next(&mut self, cx)? {
            item.skip_any(cx)?;
        }

        Ok(())
    }
}

struct Expected {
    expected: Kind,
    actual: Tag,
    pos: usize,
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            expected,
            actual,
            pos,
        } = *self;

        write!(f, "Expected {expected:?} but was {actual:?} (at {pos})",)
    }
}
