#![allow(clippy::type_complexity)]

use core::fmt;
use core::marker::PhantomData;
use core::slice;

use crate::alloc::Allocator;
use crate::de::UnsizedVisitor;
use crate::de::{
    AsDecoder, Decoder, EntriesDecoder, EntryDecoder, MapDecoder, SequenceDecoder, SizeHint, Skip,
    VariantDecoder, Visitor,
};
use crate::hint::SequenceHint;
use crate::reader::SliceReader;
use crate::storage::de::StorageDecoder;
use crate::{Context, Options};

use super::error::ErrorMessage;
use super::type_hint::{NumberHint, TypeHint};
use super::value::{Number, Value};
use super::AsValueDecoder;

/// Encoder for a single value.
pub struct ValueDecoder<'de, const OPT: Options, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    cx: C,
    value: &'de Value<A>,
    map_key: bool,
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, C, A, M> ValueDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    #[inline]
    pub(crate) const fn new(cx: C, value: &'de Value<A>) -> Self {
        Self {
            cx,
            value,
            map_key: false,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) const fn with_map_key(cx: C, value: &'de Value<A>) -> Self {
        Self {
            cx,
            value,
            map_key: true,
            _marker: PhantomData,
        }
    }
}

macro_rules! ensure_number {
    ($self:expr, $opt:expr, $hint:ident, $ident:ident $tt:tt, Value::$variant:ident($block:ident) => $ty:ty) => {
        match $self.value {
            Value::$variant($block) => <$ty>::from_number($block).map_err(|e| $self.cx.message(e)),
            Value::String(string) if crate::options::is_map_keys_as_numbers::<$opt>() && $self.map_key => {
                match <$ty>::parse_number(string) {
                    Some(value) => Ok(value),
                    None => Err($self.cx.message(ErrorMessage::ExpectedStringAsNumber)),
                }
            }
            value => {
                let $hint = value.type_hint();
                return Err($self.cx.message(ErrorMessage::$ident $tt));
            }
        }
    };
}

macro_rules! ensure {
    ($self:expr, $hint:ident, $ident:ident $tt:tt, $pat:pat => $block:expr) => {
        match $self.value {
            $pat => $block,
            value => {
                let $hint = value.type_hint();
                return Err($self.cx.message(ErrorMessage::$ident $tt));
            }
        }
    };
}

#[crate::decoder(crate)]
impl<'de, const OPT: Options, C, A, M> Decoder<'de> for ValueDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type Allocator = C::Allocator;
    type WithContext<U>
        = ValueDecoder<'de, OPT, U, A, M>
    where
        U: Context<Allocator = Self::Allocator>;
    type DecodeBuffer = AsValueDecoder<'de, OPT, C, A, M>;
    type DecodeSome = Self;
    type DecodePack = StorageDecoder<OPT, true, SliceReader<'de>, C, M>;
    type DecodeSequence = IterValueDecoder<'de, OPT, C, A, M>;
    type DecodeMap = IterValuePairsDecoder<'de, OPT, C, A, M>;
    type DecodeMapEntries = IterValuePairsDecoder<'de, OPT, C, A, M>;
    type DecodeVariant = IterValueVariantDecoder<'de, OPT, C, A, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: U) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context<Allocator = Self::Allocator>,
    {
        Ok(ValueDecoder::new(cx, self.value))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot be decoded from value")
    }

    #[inline]
    fn skip(self) -> Result<(), C::Error> {
        Ok(())
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, C::Error> {
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, C::Error> {
        Ok(AsValueDecoder::new(self.cx, self.value))
    }

    #[inline]
    fn decode_empty(self) -> Result<(), C::Error> {
        ensure!(self, hint, ExpectedUnit(hint), Value::Unit => Ok(()))
    }

    #[inline]
    fn decode_bool(self) -> Result<bool, C::Error> {
        ensure!(self, hint, ExpectedBool(hint), Value::Bool(b) => Ok(*b))
    }

    #[inline]
    fn decode_char(self) -> Result<char, C::Error> {
        ensure!(self, hint, ExpectedChar(hint), Value::Char(c) => Ok(*c))
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::U8, hint), Value::Number(n) => u8)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::U16, hint), Value::Number(n) => u16)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::U32, hint), Value::Number(n) => u32)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::U64, hint), Value::Number(n) => u64)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::U128, hint), Value::Number(n) => u128)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::I8, hint), Value::Number(n) => i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::I16, hint), Value::Number(n) => i16)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::I32, hint), Value::Number(n) => i32)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::I64, hint), Value::Number(n) => i64)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::I128, hint), Value::Number(n) => i128)
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::F32, hint), Value::Number(Number::F32(n)) => Ok(*n))
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::F64, hint), Value::Number(Number::F64(n)) => Ok(*n))
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::Usize, hint), Value::Number(n) => usize)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, C::Error> {
        ensure_number!(self, OPT, hint, ExpectedNumber(NumberHint::Isize, hint), Value::Number(n) => isize)
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], C::Error> {
        ensure!(self, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            <[u8; N]>::try_from(bytes.as_slice()).map_err(|_| self.cx.message(ErrorMessage::ArrayOutOfBounds))
        })
    }

    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: UnsizedVisitor<'de, C, [u8]>,
    {
        ensure!(self, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            visitor.visit_borrowed(self.cx, bytes)
        })
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: UnsizedVisitor<'de, C, str>,
    {
        ensure!(self, hint, ExpectedString(hint), Value::String(string) => {
            visitor.visit_borrowed(self.cx, string)
        })
    }

    #[inline]
    fn decode_option(self) -> Result<Option<Self::DecodeSome>, C::Error> {
        ensure!(self, hint, ExpectedOption(hint), Value::Option(option) => {
            Ok(option.as_ref().map(|some| ValueDecoder::new(self.cx, some)))
        })
    }

    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, C::Error>,
    {
        ensure!(self, hint, ExpectedPack(hint), Value::Bytes(pack) => {
            f(&mut StorageDecoder::new(self.cx, SliceReader::new(pack)))
        })
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, <Self::Cx as Context>::Error>,
    {
        ensure!(self, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            f(&mut IterValueDecoder::new(self.cx, sequence))
        })
    }

    #[inline]
    fn decode_sequence_hint<F, O>(self, _: &SequenceHint, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, C::Error>,
    {
        ensure!(self, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            f(&mut IterValueDecoder::new(self.cx, sequence))
        })
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, C::Error>,
    {
        ensure!(self, hint, ExpectedMap(hint), Value::Map(st) => {
            f(&mut IterValuePairsDecoder::new(self.cx, st))
        })
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, C::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_variant<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, C::Error>,
    {
        ensure!(self, hint, ExpectedVariant(hint), Value::Variant(st) => {
            f(&mut IterValueVariantDecoder::new(self.cx, st))
        })
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, Self::Cx>,
    {
        match self.value {
            Value::Unit => visitor.visit_empty(self.cx),
            Value::Bool(value) => visitor.visit_bool(self.cx, *value),
            Value::Char(value) => visitor.visit_char(self.cx, *value),
            Value::Number(number) => match number {
                Number::U8(value) => visitor.visit_u8(self.cx, *value),
                Number::U16(value) => visitor.visit_u16(self.cx, *value),
                Number::U32(value) => visitor.visit_u32(self.cx, *value),
                Number::U64(value) => visitor.visit_u64(self.cx, *value),
                Number::U128(value) => visitor.visit_u128(self.cx, *value),
                Number::I8(value) => visitor.visit_i8(self.cx, *value),
                Number::I16(value) => visitor.visit_i16(self.cx, *value),
                Number::I32(value) => visitor.visit_i32(self.cx, *value),
                Number::I64(value) => visitor.visit_i64(self.cx, *value),
                Number::I128(value) => visitor.visit_i128(self.cx, *value),
                Number::Usize(value) => visitor.visit_usize(self.cx, *value),
                Number::Isize(value) => visitor.visit_isize(self.cx, *value),
                Number::F32(value) => visitor.visit_f32(self.cx, *value),
                Number::F64(value) => visitor.visit_f64(self.cx, *value),
            },
            Value::Bytes(bytes) => {
                let visitor = visitor.visit_bytes(self.cx, SizeHint::exact(bytes.len()))?;
                visitor.visit_borrowed(self.cx, bytes)
            }
            Value::String(string) => {
                let visitor = visitor.visit_string(self.cx, SizeHint::exact(string.len()))?;
                visitor.visit_borrowed(self.cx, string)
            }
            Value::Sequence(values) => {
                visitor.visit_sequence(&mut IterValueDecoder::<OPT, _, _, M>::new(self.cx, values))
            }
            Value::Map(values) => visitor.visit_map(
                &mut IterValuePairsDecoder::<OPT, _, _, M>::new(self.cx, values),
            ),
            Value::Variant(variant) => {
                visitor.visit_variant(&mut IterValueVariantDecoder::<OPT, _, _, M>::new(
                    self.cx, variant,
                ))
            }
            Value::Option(option) => visitor.visit_option(
                self.cx,
                option
                    .as_ref()
                    .map(|value| ValueDecoder::<OPT, _, _, M>::new(self.cx, value)),
            ),
        }
    }
}

impl<const OPT: Options, C, A, M> AsDecoder for ValueDecoder<'_, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    type Cx = C;
    type Mode = M;
    type Decoder<'this>
        = ValueDecoder<'this, OPT, C, A, M>
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, C::Error> {
        Ok(ValueDecoder::new(self.cx, self.value))
    }
}

/// A decoder over a simple value iterator.
pub struct IterValueDecoder<'de, const OPT: Options, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    cx: C,
    iter: slice::Iter<'de, Value<A>>,
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, C, A, M> IterValueDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    #[inline]
    fn new(cx: C, values: &'de [Value<A>]) -> Self {
        Self {
            cx,
            iter: values.iter(),
            _marker: PhantomData,
        }
    }
}

impl<'de, const OPT: Options, C, A, M> SequenceDecoder<'de> for IterValueDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    type Cx = C;
    type Mode = M;
    type DecodeNext<'this>
        = ValueDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        match self.iter.next() {
            Some(value) => Ok(Some(ValueDecoder::new(self.cx, value))),
            None => Ok(None),
        }
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self.iter.next() {
            Some(value) => Ok(ValueDecoder::new(self.cx, value)),
            None => Err(self.cx.message(ErrorMessage::ExpectedPackValue)),
        }
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairsDecoder<'de, const OPT: Options, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    cx: C,
    iter: slice::Iter<'de, (Value<A>, Value<A>)>,
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, C, A, M> IterValuePairsDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    #[inline]
    fn new(cx: C, values: &'de [(Value<A>, Value<A>)]) -> Self {
        Self {
            cx,
            iter: values.iter(),
            _marker: PhantomData,
        }
    }
}

impl<'de, const OPT: Options, C, A, M> MapDecoder<'de> for IterValuePairsDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    type Cx = C;
    type Mode = M;
    type DecodeEntry<'this>
        = IterValuePairDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = IterValuePairsDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };

        Ok(Some(IterValuePairDecoder::new(self.cx, value)))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, <Self::Cx as Context>::Error> {
        Ok(IterValuePairsDecoder::new(self.cx, self.iter.as_slice()))
    }
}

impl<'de, const OPT: Options, C, A, M> EntriesDecoder<'de>
    for IterValuePairsDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    type Cx = C;
    type Mode = M;
    type DecodeEntryKey<'this>
        = ValueDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = ValueDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, C::Error> {
        let Some((name, _)) = self.iter.clone().next() else {
            return Ok(None);
        };

        Ok(Some(ValueDecoder::with_map_key(self.cx, name)))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, C::Error> {
        let Some((_, value)) = self.iter.next() else {
            return Err(self.cx.message(ErrorMessage::ExpectedMapValue));
        };

        Ok(ValueDecoder::new(self.cx, value))
    }

    #[inline]
    fn end_entries(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, const OPT: Options, C, A, M> EntryDecoder<'de> for IterValuePairDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    type Cx = C;
    type Mode = M;
    type DecodeKey<'this>
        = ValueDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;
    type DecodeValue = ValueDecoder<'de, OPT, C, A, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, C::Error> {
        Ok(ValueDecoder::with_map_key(self.cx, &self.pair.0))
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, C::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.1))
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairDecoder<'de, const OPT: Options, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    cx: C,
    pair: &'de (Value<A>, Value<A>),
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, C, A, M> IterValuePairDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    #[inline]
    const fn new(cx: C, pair: &'de (Value<A>, Value<A>)) -> Self {
        Self {
            cx,
            pair,
            _marker: PhantomData,
        }
    }
}

/// A decoder over a simple value pair as a variant.
pub struct IterValueVariantDecoder<'de, const OPT: Options, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    cx: C,
    pair: &'de (Value<A>, Value<A>),
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, C, A, M> IterValueVariantDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    #[inline]
    const fn new(cx: C, pair: &'de (Value<A>, Value<A>)) -> Self {
        Self {
            cx,
            pair,
            _marker: PhantomData,
        }
    }
}

impl<'de, const OPT: Options, C, A, M> VariantDecoder<'de>
    for IterValueVariantDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    type Cx = C;
    type Mode = M;
    type DecodeTag<'this>
        = ValueDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = ValueDecoder<'de, OPT, C, A, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.0))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.1))
    }
}

/// Conversion trait for numbers.
trait FromNumber: Sized {
    const NUMBER_HINT: NumberHint;

    fn from_number(number: &Number) -> Result<Self, ErrorMessage>;

    fn parse_number(string: &str) -> Option<Self>;
}

macro_rules! integer_from {
    ($ty:ty, $variant:ident) => {
        impl FromNumber for $ty {
            const NUMBER_HINT: NumberHint = NumberHint::$variant;

            #[inline]
            fn from_number(number: &Number) -> Result<Self, ErrorMessage> {
                let out = match number {
                    Number::U8(n) => Self::try_from(*n).ok(),
                    Number::U16(n) => Self::try_from(*n).ok(),
                    Number::U32(n) => Self::try_from(*n).ok(),
                    Number::U64(n) => Self::try_from(*n).ok(),
                    Number::U128(n) => Self::try_from(*n).ok(),
                    Number::I8(n) => Self::try_from(*n).ok(),
                    Number::I16(n) => Self::try_from(*n).ok(),
                    Number::I32(n) => Self::try_from(*n).ok(),
                    Number::I64(n) => Self::try_from(*n).ok(),
                    Number::I128(n) => Self::try_from(*n).ok(),
                    Number::Usize(n) => Self::try_from(*n).ok(),
                    Number::Isize(n) => Self::try_from(*n).ok(),
                    Number::F32(v) => Some(*v as $ty),
                    Number::F64(v) => Some(*v as $ty),
                };

                match out {
                    Some(out) => Ok(out),
                    None => Err(ErrorMessage::ExpectedNumber(
                        Self::NUMBER_HINT,
                        TypeHint::Number(number.type_hint()),
                    )),
                }
            }

            #[inline]
            fn parse_number(string: &str) -> Option<Self> {
                string.parse().ok()
            }
        }
    };
}

macro_rules! float_from {
    ($ty:ty, $variant:ident) => {
        impl FromNumber for $ty {
            const NUMBER_HINT: NumberHint = NumberHint::$variant;

            #[inline]
            fn from_number(number: &Number) -> Result<Self, ErrorMessage> {
                let out = match number {
                    Number::U8(n) => Some(*n as $ty),
                    Number::U16(n) => Some(*n as $ty),
                    Number::U32(n) => Some(*n as $ty),
                    Number::U64(n) => Some(*n as $ty),
                    Number::U128(n) => Some(*n as $ty),
                    Number::I8(n) => Some(*n as $ty),
                    Number::I16(n) => Some(*n as $ty),
                    Number::I32(n) => Some(*n as $ty),
                    Number::I64(n) => Some(*n as $ty),
                    Number::I128(n) => Some(*n as $ty),
                    Number::Usize(n) => Some(*n as $ty),
                    Number::Isize(n) => Some(*n as $ty),
                    Number::F32(v) => Some(*v as $ty),
                    Number::F64(v) => Some(*v as $ty),
                };

                match out {
                    Some(out) => Ok(out),
                    None => Err(ErrorMessage::ExpectedNumber(
                        Self::NUMBER_HINT,
                        TypeHint::Number(number.type_hint()),
                    )),
                }
            }

            #[inline]
            fn parse_number(string: &str) -> Option<Self> {
                string.parse().ok()
            }
        }
    };
}

integer_from!(u8, U8);
integer_from!(u16, U16);
integer_from!(u32, U32);
integer_from!(u64, U64);
integer_from!(u128, U128);
integer_from!(i8, I8);
integer_from!(i16, I16);
integer_from!(i32, I32);
integer_from!(i64, I64);
integer_from!(i128, I128);
integer_from!(usize, Usize);
integer_from!(isize, Isize);
float_from!(f32, F32);
float_from!(f64, F64);
