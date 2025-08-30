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
use super::{AsValueDecoder, Number, Value, ValueKind};

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
    ($self:expr, $opt:expr, $hint_type:ident, $value:ident::$variant:ident($v:ident) => $ty:ty) => {
        match &$self.value.kind {
            $value::$variant($v) => <$ty>::from_number($v).map_err(|e| $self.cx.message(e)),
            ValueKind::String(string)
                if crate::options::is_map_keys_as_numbers::<$opt>() && $self.map_key =>
            {
                match <$ty>::parse_number(string) {
                    Some(value) => Ok(value),
                    None => Err($self.cx.message(ErrorMessage::ExpectedStringAsNumber)),
                }
            }
            _ => {
                let hint = $self.value.type_hint();
                return Err($self
                    .cx
                    .message(ErrorMessage::ExpectedNumber(NumberHint::$hint_type, hint)));
            }
        }
    };
}

macro_rules! ensure {
    ($self:expr, $hint:ident, $ident:ident $tt:tt, $pat:pat => $block:expr) => {
        match &$self.value.kind {
            $pat => $block,
            _ => {
                let $hint = $self.value.type_hint();
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
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = Self;
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
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot be decoded from value")
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(ValueDecoder::new(self.cx, self.value))
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
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, Self::Error> {
        Ok(AsValueDecoder::new(self.cx, self.value))
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        ensure!(self, hint, ExpectedUnit(hint), ValueKind::Unit => Ok(()))
    }

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        ensure!(self, hint, ExpectedBool(hint), ValueKind::Bool(b) => Ok(*b))
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        ensure!(self, hint, ExpectedChar(hint), ValueKind::Char(c) => Ok(*c))
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        ensure_number!(self, OPT, U8, ValueKind::Number(n) => u8)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        ensure_number!(self, OPT, U16, ValueKind::Number(n) => u16)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        ensure_number!(self, OPT, U32, ValueKind::Number(n) => u32)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        ensure_number!(self, OPT, U64, ValueKind::Number(n) => u64)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        ensure_number!(self, OPT, U128, ValueKind::Number(n) => u128)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        ensure_number!(self, OPT, I8, ValueKind::Number(n) => i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        ensure_number!(self, OPT, I16, ValueKind::Number(n) => i16)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        ensure_number!(self, OPT, I32, ValueKind::Number(n) => i32)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        ensure_number!(self, OPT, I64, ValueKind::Number(n) => i64)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        ensure_number!(self, OPT, I128, ValueKind::Number(n) => i128)
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::F32, hint), ValueKind::Number(Number::F32(n)) => Ok(*n))
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::F64, hint), ValueKind::Number(Number::F64(n)) => Ok(*n))
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        ensure_number!(self, OPT, Usize, ValueKind::Number(n) => usize)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        ensure_number!(self, OPT, Isize, ValueKind::Number(n) => isize)
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        ensure!(self, hint, ExpectedBytes(hint), ValueKind::Bytes(bytes) => {
            <[u8; N]>::try_from(bytes.as_slice()).map_err(|_| self.cx.message(ErrorMessage::ArrayOutOfBounds))
        })
    }

    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, [u8], Error = Self::Error, Allocator = Self::Allocator>,
    {
        ensure!(self, hint, ExpectedBytes(hint), ValueKind::Bytes(bytes) => {
            visitor.visit_borrowed(self.cx, bytes)
        })
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        ensure!(self, hint, ExpectedString(hint), ValueKind::String(string) => {
            visitor.visit_borrowed(self.cx, string)
        })
    }

    #[inline]
    fn decode_option(self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        ensure!(self, hint, ExpectedOption(hint), ValueKind::Option(option) => {
            Ok(option.as_ref().map(|some| ValueDecoder::new(self.cx, some)))
        })
    }

    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        ensure!(self, hint, ExpectedPack(hint), ValueKind::Bytes(pack) => {
            f(&mut StorageDecoder::new(self.cx, SliceReader::new(pack)))
        })
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        ensure!(self, hint, ExpectedSequence(hint), ValueKind::Sequence(sequence) => {
            f(&mut IterValueDecoder::new(self.cx, sequence))
        })
    }

    #[inline]
    fn decode_sequence_hint<F, O>(self, _: impl SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        ensure!(self, hint, ExpectedSequence(hint), ValueKind::Sequence(sequence) => {
            f(&mut IterValueDecoder::new(self.cx, sequence))
        })
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        ensure!(self, hint, ExpectedMap(hint), ValueKind::Map(st) => {
            f(&mut IterValuePairsDecoder::new(self.cx, st))
        })
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
        ensure!(self, hint, ExpectedVariant(hint), ValueKind::Variant(st) => {
            f(&mut IterValueVariantDecoder::new(self.cx, st))
        })
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, Self::Cx, Error = Self::Error, Allocator = Self::Allocator>,
    {
        match &self.value.kind {
            ValueKind::Unit => visitor.visit_empty(self.cx),
            ValueKind::Bool(value) => visitor.visit_bool(self.cx, *value),
            ValueKind::Char(value) => visitor.visit_char(self.cx, *value),
            ValueKind::Number(number) => match number {
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
            ValueKind::Bytes(bytes) => {
                let visitor = visitor.visit_bytes(self.cx, SizeHint::exact(bytes.len()))?;
                visitor.visit_borrowed(self.cx, bytes)
            }
            ValueKind::String(string) => {
                let visitor = visitor.visit_string(self.cx, SizeHint::exact(string.len()))?;
                visitor.visit_borrowed(self.cx, string)
            }
            ValueKind::Sequence(values) => {
                visitor.visit_sequence(&mut IterValueDecoder::<OPT, _, _, M>::new(self.cx, values))
            }
            ValueKind::Map(values) => visitor.visit_map(
                &mut IterValuePairsDecoder::<OPT, _, _, M>::new(self.cx, values),
            ),
            ValueKind::Variant(variant) => {
                visitor.visit_variant(&mut IterValueVariantDecoder::<OPT, _, _, M>::new(
                    self.cx, variant,
                ))
            }
            ValueKind::Option(Some(value)) => {
                visitor.visit_some(ValueDecoder::<OPT, _, _, M>::new(self.cx, value))
            }
            ValueKind::Option(None) => visitor.visit_none(self.cx),
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
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type Decoder<'this>
        = ValueDecoder<'this, OPT, C, A, M>
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, Self::Error> {
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
    type Error = C::Error;
    type Allocator = C::Allocator;
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
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, Self::Error> {
        match self.iter.next() {
            Some(value) => Ok(Some(ValueDecoder::new(self.cx, value))),
            None => Ok(None),
        }
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
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
    type Error = C::Error;
    type Allocator = C::Allocator;
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
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, Self::Error> {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };

        Ok(Some(IterValuePairDecoder::new(self.cx, value)))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, Self::Error> {
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
    type Error = C::Error;
    type Allocator = C::Allocator;
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
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error> {
        let Some((name, _)) = self.iter.clone().next() else {
            return Ok(None);
        };

        Ok(Some(ValueDecoder::with_map_key(self.cx, name)))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        let Some((_, value)) = self.iter.next() else {
            return Err(self.cx.message(ErrorMessage::ExpectedMapValue));
        };

        Ok(ValueDecoder::new(self.cx, value))
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
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
    type Error = C::Error;
    type Allocator = C::Allocator;
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
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, Self::Error> {
        Ok(ValueDecoder::with_map_key(self.cx, &self.pair.0))
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, Self::Error> {
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
    type Error = C::Error;
    type Allocator = C::Allocator;
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
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.0))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error> {
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
