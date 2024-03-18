use core::fmt;
use core::slice;

#[cfg(feature = "alloc")]
use musli::de::ValueVisitor;
use musli::de::{
    AsDecoder, Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, NumberHint, PackDecoder,
    SequenceDecoder, SizeHint, StructDecoder, StructFieldDecoder, StructFieldsDecoder, TypeHint,
    VariantDecoder, Visitor,
};
use musli::Context;
use musli_storage::de::StorageDecoder;
use musli_storage::options::Options;
use musli_storage::reader::SliceReader;

use crate::error::ErrorMessage;
use crate::value::{Number, Value};
use crate::AsValueDecoder;

/// Encoder for a single value.
pub struct ValueDecoder<'de, const F: Options> {
    value: &'de Value,
}

impl<'de, const F: Options> ValueDecoder<'de, F> {
    #[inline]
    pub(crate) const fn new(value: &'de Value) -> Self {
        Self { value }
    }
}

macro_rules! ensure {
    ($self:expr, $cx:expr, $hint:ident, $ident:ident $tt:tt, $pat:pat => $block:expr) => {
        match $self.value {
            $pat => $block,
            value => {
                let $hint = value.type_hint();
                return Err($cx.message(ErrorMessage::$ident $tt));
            }
        }
    };
}

#[musli::decoder]
impl<'de, C: ?Sized + Context, const F: Options> Decoder<'de, C> for ValueDecoder<'de, F> {
    type WithContext<U> = Self where U: Context;
    type DecodeBuffer = AsValueDecoder<F>;
    type DecodeSome = Self;
    type DecodePack = StorageDecoder<SliceReader<'de>, F>;
    type DecodeSequence = IterValueDecoder<'de, F>;
    type DecodeTuple = IterValueDecoder<'de, F>;
    type DecodeMap = IterValuePairsDecoder<'de, F>;
    type DecodeStruct = IterValuePairsDecoder<'de, F>;
    type DecodeVariant = IterValueVariantDecoder<'de, F>;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot be decoded from value")
    }

    #[inline]
    fn type_hint(&mut self, _: &C) -> Result<TypeHint, C::Error> {
        Ok(self.value.type_hint())
    }

    #[inline]
    fn decode_buffer(self, _: &C) -> Result<Self::DecodeBuffer, C::Error> {
        Ok(AsValueDecoder::new(self.value.clone()))
    }

    #[inline]
    fn decode_unit(self, cx: &C) -> Result<(), C::Error> {
        ensure!(self, cx, hint, ExpectedUnit(hint), Value::Unit => Ok(()))
    }

    #[inline]
    fn decode_bool(self, cx: &C) -> Result<bool, C::Error> {
        ensure!(self, cx, hint, ExpectedBool(hint), Value::Bool(b) => Ok(*b))
    }

    #[inline]
    fn decode_char(self, cx: &C) -> Result<char, C::Error> {
        ensure!(self, cx, hint, ExpectedChar(hint), Value::Char(c) => Ok(*c))
    }

    #[inline]
    fn decode_u8(self, cx: &C) -> Result<u8, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U8, hint), Value::Number(n) => {
            u8::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_u16(self, cx: &C) -> Result<u16, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U16, hint), Value::Number(n) => {
            u16::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U32, hint), Value::Number(n) => {
            u32::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_u64(self, cx: &C) -> Result<u64, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U64, hint), Value::Number(n) => {
            u64::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_u128(self, cx: &C) -> Result<u128, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U128, hint), Value::Number(n) => {
            u128::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_i8(self, cx: &C) -> Result<i8, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I8, hint), Value::Number(n) => {
            i8::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_i16(self, cx: &C) -> Result<i16, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I16, hint), Value::Number(n) => {
            i16::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_i32(self, cx: &C) -> Result<i32, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I32, hint), Value::Number(n) => {
            i32::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_i64(self, cx: &C) -> Result<i64, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I64, hint), Value::Number(n) => {
            i64::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_i128(self, cx: &C) -> Result<i128, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I128, hint), Value::Number(n) => {
            i128::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_usize(self, cx: &C) -> Result<usize, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::Usize, hint), Value::Number(n) => {
            usize::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_isize(self, cx: &C) -> Result<isize, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::Isize, hint), Value::Number(n) => {
            isize::from_number(n).map_err(cx.map_message())
        })
    }

    #[inline]
    fn decode_f32(self, cx: &C) -> Result<f32, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::F32, hint), Value::Number(Number::F32(n)) => Ok(*n))
    }

    #[inline]
    fn decode_f64(self, cx: &C) -> Result<f64, C::Error> {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::F64, hint), Value::Number(Number::F64(n)) => Ok(*n))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_array<const N: usize>(self, cx: &C) -> Result<[u8; N], C::Error> {
        ensure!(self, cx, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            <[u8; N]>::try_from(bytes.as_slice()).map_err(|_| cx.message(ErrorMessage::ArrayOutOfBounds))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        ensure!(self, cx, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            visitor.visit_borrowed(cx, bytes)
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_string<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
    {
        ensure!(self, cx, hint, ExpectedString(hint), Value::String(string) => {
            visitor.visit_borrowed(cx, string)
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_option(self, cx: &C) -> Result<Option<Self::DecodeSome>, C::Error> {
        ensure!(self, cx, hint, ExpectedOption(hint), Value::Option(option) => {
            Ok(option.as_ref().map(|some| ValueDecoder::new(some)))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_pack(self, cx: &C) -> Result<Self::DecodePack, C::Error> {
        ensure!(self, cx, hint, ExpectedPack(hint), Value::Bytes(pack) => {
            Ok(StorageDecoder::new(SliceReader::new(pack)))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_sequence(self, cx: &C) -> Result<Self::DecodeSequence, C::Error> {
        ensure!(self, cx, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            Ok(IterValueDecoder::new(sequence))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_tuple(self, cx: &C, _: usize) -> Result<Self::DecodeTuple, C::Error> {
        ensure!(self, cx, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            Ok(IterValueDecoder::new(sequence))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_map(self, cx: &C) -> Result<Self::DecodeMap, C::Error> {
        ensure!(self, cx, hint, ExpectedMap(hint), Value::Map(map) => {
            Ok(IterValuePairsDecoder::new(map))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_struct(self, cx: &C, _: Option<usize>) -> Result<Self::DecodeStruct, C::Error> {
        ensure!(self, cx, hint, ExpectedMap(hint), Value::Map(st) => {
            Ok(IterValuePairsDecoder::new(st))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_variant(self, cx: &C) -> Result<Self::DecodeVariant, C::Error> {
        ensure!(self, cx, hint, ExpectedVariant(hint), Value::Variant(st) => {
            Ok(IterValueVariantDecoder::new(st))
        })
    }

    #[inline]
    fn decode_any<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, C>,
    {
        match self.value {
            Value::Unit => visitor.visit_unit(cx),
            Value::Bool(value) => visitor.visit_bool(cx, *value),
            Value::Char(value) => visitor.visit_char(cx, *value),
            Value::Number(number) => match number {
                Number::U8(value) => visitor.visit_u8(cx, *value),
                Number::U16(value) => visitor.visit_u16(cx, *value),
                Number::U32(value) => visitor.visit_u32(cx, *value),
                Number::U64(value) => visitor.visit_u64(cx, *value),
                Number::U128(value) => visitor.visit_u128(cx, *value),
                Number::I8(value) => visitor.visit_i8(cx, *value),
                Number::I16(value) => visitor.visit_i16(cx, *value),
                Number::I32(value) => visitor.visit_i32(cx, *value),
                Number::I64(value) => visitor.visit_i64(cx, *value),
                Number::I128(value) => visitor.visit_i128(cx, *value),
                Number::Usize(value) => visitor.visit_usize(cx, *value),
                Number::Isize(value) => visitor.visit_isize(cx, *value),
                Number::F32(value) => visitor.visit_f32(cx, *value),
                Number::F64(value) => visitor.visit_f64(cx, *value),
            },
            #[cfg(feature = "alloc")]
            Value::Bytes(bytes) => {
                let visitor = visitor.visit_bytes(cx, SizeHint::Exact(bytes.len()))?;
                visitor.visit_borrowed(cx, bytes)
            }
            #[cfg(feature = "alloc")]
            Value::String(string) => {
                let visitor = visitor.visit_string(cx, SizeHint::Exact(string.len()))?;
                visitor.visit_borrowed(cx, string)
            }
            #[cfg(feature = "alloc")]
            Value::Sequence(values) => {
                visitor.visit_sequence(cx, IterValueDecoder::<F>::new(values))
            }
            #[cfg(feature = "alloc")]
            Value::Map(values) => visitor.visit_map(cx, IterValuePairsDecoder::<F>::new(values)),
            #[cfg(feature = "alloc")]
            Value::Variant(variant) => {
                visitor.visit_variant(cx, IterValueVariantDecoder::<F>::new(variant))
            }
            #[cfg(feature = "alloc")]
            Value::Option(option) => visitor.visit_option(
                cx,
                option.as_ref().map(|value| ValueDecoder::<F>::new(value)),
            ),
        }
    }
}

impl<'a, C: ?Sized + Context, const F: Options> AsDecoder<C> for ValueDecoder<'a, F> {
    type Decoder<'this> = ValueDecoder<'this, F> where Self: 'this;

    #[inline]
    fn as_decoder(&self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        Ok(ValueDecoder::new(self.value))
    }
}

/// A decoder over a simple value iterator.

pub struct IterValueDecoder<'de, const F: Options> {
    iter: slice::Iter<'de, Value>,
}

#[cfg(feature = "alloc")]
impl<'de, const F: Options> IterValueDecoder<'de, F> {
    #[inline]
    fn new(values: &'de [Value]) -> Self {
        Self {
            iter: values.iter(),
        }
    }
}

impl<'de, C: ?Sized + Context, const F: Options> PackDecoder<'de, C> for IterValueDecoder<'de, F> {
    type DecodeNext<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    #[inline]
    fn decode_next(&mut self, cx: &C) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self.iter.next() {
            Some(value) => Ok(ValueDecoder::new(value)),
            None => Err(cx.message(ErrorMessage::ExpectedPackValue)),
        }
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, C: ?Sized + Context, const F: Options> SequenceDecoder<'de, C>
    for IterValueDecoder<'de, F>
{
    type DecodeNext<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn decode_next(&mut self, _: &C) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        match self.iter.next() {
            Some(value) => Ok(Some(ValueDecoder::new(value))),
            None => Ok(None),
        }
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairsDecoder<'de, const F: Options> {
    iter: slice::Iter<'de, (Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<'de, const F: Options> IterValuePairsDecoder<'de, F> {
    #[inline]
    fn new(values: &'de [(Value, Value)]) -> Self {
        Self {
            iter: values.iter(),
        }
    }
}

#[musli::map_decoder]
impl<'de, C: ?Sized + Context, const F: Options> MapDecoder<'de, C>
    for IterValuePairsDecoder<'de, F>
{
    type DecodeEntry<'this> = IterValuePairDecoder<'de, F>
    where
        Self: 'this;
    type IntoMapEntries = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn into_map_entries(self, _: &C) -> Result<Self::IntoMapEntries, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_entry(&mut self, _: &C) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        Ok(self.iter.next().map(IterValuePairDecoder::new))
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, C: ?Sized + Context, const F: Options> MapEntriesDecoder<'de, C>
    for IterValuePairsDecoder<'de, F>
{
    type DecodeMapEntryKey<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    type DecodeMapEntryValue<'this> = ValueDecoder<'de, F> where Self: 'this;

    #[inline]
    fn decode_map_entry_key(
        &mut self,
        _: &C,
    ) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        let Some((name, _)) = self.iter.clone().next() else {
            return Ok(None);
        };

        Ok(Some(ValueDecoder::new(name)))
    }

    #[inline]
    fn decode_map_entry_value(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        let Some((_, value)) = self.iter.next() else {
            return Err(cx.message(ErrorMessage::ExpectedMapValue));
        };

        Ok(ValueDecoder::new(value))
    }

    #[inline]
    fn skip_map_entry_value(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(true)
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, C: ?Sized + Context, const F: Options> MapEntryDecoder<'de, C>
    for IterValuePairDecoder<'de, F>
{
    type DecodeMapKey<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    type DecodeMapValue = ValueDecoder<'de, F>;

    #[inline]
    fn decode_map_key(&mut self, _: &C) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(ValueDecoder::new(&self.pair.0))
    }

    #[inline]
    fn decode_map_value(self, _: &C) -> Result<Self::DecodeMapValue, C::Error> {
        Ok(ValueDecoder::new(&self.pair.1))
    }

    #[inline]
    fn skip_map_value(self, _: &C) -> Result<bool, C::Error> {
        Ok(true)
    }
}

#[musli::struct_decoder]
impl<'de, C: ?Sized + Context, const F: Options> StructDecoder<'de, C>
    for IterValuePairsDecoder<'de, F>
{
    type DecodeField<'this> = IterValuePairDecoder<'de, F>
    where
        Self: 'this;
    type IntoStructFields = Self;

    #[inline]
    fn size_hint(&self, cx: &C) -> SizeHint {
        MapDecoder::size_hint(self, cx)
    }

    #[inline]
    fn into_struct_fields(self, _: &C) -> Result<Self::IntoStructFields, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_field(&mut self, cx: &C) -> Result<Option<Self::DecodeField<'_>>, C::Error> {
        MapDecoder::decode_entry(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapDecoder::end(self, cx)
    }
}

impl<'de, C: ?Sized + Context, const F: Options> StructFieldsDecoder<'de, C>
    for IterValuePairsDecoder<'de, F>
{
    type DecodeStructFieldName<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    type DecodeStructFieldValue<'this> = ValueDecoder<'de, F> where Self: 'this;

    #[inline]
    fn decode_struct_field_name(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        let Some((name, _)) = self.iter.clone().next() else {
            return Err(cx.message(ErrorMessage::ExpectedFieldName));
        };

        Ok(ValueDecoder::new(name))
    }

    #[inline]
    fn decode_struct_field_value(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        let Some((_, value)) = self.iter.next() else {
            return Err(cx.message(ErrorMessage::ExpectedFieldValue));
        };

        Ok(ValueDecoder::new(value))
    }

    #[inline]
    fn skip_struct_field_value(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(self.iter.next().is_some())
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, C: ?Sized + Context, const F: Options> StructFieldDecoder<'de, C>
    for IterValuePairDecoder<'de, F>
{
    type DecodeFieldName<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    type DecodeFieldValue = ValueDecoder<'de, F>;

    #[inline]
    fn decode_field_name(&mut self, cx: &C) -> Result<Self::DecodeFieldName<'_>, C::Error> {
        MapEntryDecoder::decode_map_key(self, cx)
    }

    #[inline]
    fn decode_field_value(self, cx: &C) -> Result<Self::DecodeFieldValue, C::Error> {
        MapEntryDecoder::decode_map_value(self, cx)
    }

    #[inline]
    fn skip_field_value(self, cx: &C) -> Result<bool, C::Error> {
        MapEntryDecoder::skip_map_value(self, cx)
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairDecoder<'de, const F: Options> {
    pair: &'de (Value, Value),
}

impl<'de, const F: Options> IterValuePairDecoder<'de, F> {
    #[inline]
    const fn new(pair: &'de (Value, Value)) -> Self {
        Self { pair }
    }
}

/// A decoder over a simple value pair as a variant.
pub struct IterValueVariantDecoder<'de, const F: Options> {
    pair: &'de (Value, Value),
}

#[cfg(feature = "alloc")]
impl<'de, const F: Options> IterValueVariantDecoder<'de, F> {
    #[inline]
    const fn new(pair: &'de (Value, Value)) -> Self {
        Self { pair }
    }
}

impl<'de, C: ?Sized + Context, const F: Options> VariantDecoder<'de, C>
    for IterValueVariantDecoder<'de, F>
{
    type DecodeTag<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    type DecodeVariant<'this> = ValueDecoder<'de, F>
    where
        Self: 'this;

    #[inline]
    fn decode_tag(&mut self, _: &C) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(ValueDecoder::new(&self.pair.0))
    }

    #[inline]
    fn decode_value(&mut self, _: &C) -> Result<Self::DecodeVariant<'_>, C::Error> {
        Ok(ValueDecoder::new(&self.pair.1))
    }

    #[inline]
    fn skip_value(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(true)
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

/// Conversion trait for numbers.
trait FromNumber: Sized {
    const NUMBER_HINT: NumberHint;

    fn from_number(number: &Number) -> Result<Self, ErrorMessage>;
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
