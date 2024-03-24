use core::fmt;
use core::slice;

#[cfg(feature = "alloc")]
use musli::de::ValueVisitor;
use musli::de::{
    AsDecoder, Decode, Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, NumberHint,
    PackDecoder, SequenceDecoder, SizeHint, StructDecoder, StructFieldDecoder, StructFieldsDecoder,
    TypeHint, VariantDecoder, Visitor,
};
use musli::Context;
use musli_storage::de::StorageDecoder;
use musli_storage::options::Options;
use musli_storage::reader::SliceReader;

use crate::error::ErrorMessage;
use crate::value::{Number, Value};
use crate::AsValueDecoder;

/// Encoder for a single value.
pub struct ValueDecoder<'a, 'de, const F: Options, C: ?Sized> {
    cx: &'a C,
    value: &'de Value,
}

impl<'a, 'de, const F: Options, C: ?Sized> ValueDecoder<'a, 'de, F, C> {
    #[inline]
    pub(crate) const fn new(cx: &'a C, value: &'de Value) -> Self {
        Self { cx, value }
    }
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

#[musli::decoder]
impl<'a, 'de, C: ?Sized + Context, const F: Options> Decoder<'de> for ValueDecoder<'a, 'de, F, C> {
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<'this, U> = ValueDecoder<'this, 'de, F, U> where U: 'this + Context;
    type DecodeBuffer = AsValueDecoder<'a, F, C>;
    type DecodeSome = Self;
    type DecodePack = StorageDecoder<'a, SliceReader<'de>, F, C>;
    type DecodeSequence = IterValueDecoder<'a, 'de, F, C>;
    type DecodeTuple = IterValueDecoder<'a, 'de, F, C>;
    type DecodeMap = IterValuePairsDecoder<'a, 'de, F, C>;
    type DecodeStruct = IterValuePairsDecoder<'a, 'de, F, C>;
    type DecodeVariant = IterValueVariantDecoder<'a, 'de, F, C>;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(ValueDecoder::new(cx, self.value))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot be decoded from value")
    }

    #[inline]
    fn decode<T>(self) -> Result<T, C::Error>
    where
        T: Decode<'de, Self::Mode>,
    {
        T::decode(self.cx, self)
    }

    #[inline]
    fn type_hint(&mut self) -> Result<TypeHint, C::Error> {
        Ok(self.value.type_hint())
    }

    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, C::Error> {
        Ok(AsValueDecoder::new(self.cx, self.value.clone()))
    }

    #[inline]
    fn decode_unit(self) -> Result<(), C::Error> {
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
        ensure!(self, hint, ExpectedNumber(NumberHint::U8, hint), Value::Number(n) => {
            u8::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::U16, hint), Value::Number(n) => {
            u16::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::U32, hint), Value::Number(n) => {
            u32::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::U64, hint), Value::Number(n) => {
            u64::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::U128, hint), Value::Number(n) => {
            u128::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::I8, hint), Value::Number(n) => {
            i8::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::I16, hint), Value::Number(n) => {
            i16::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::I32, hint), Value::Number(n) => {
            i32::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::I64, hint), Value::Number(n) => {
            i64::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::I128, hint), Value::Number(n) => {
            i128::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::Usize, hint), Value::Number(n) => {
            usize::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::Isize, hint), Value::Number(n) => {
            isize::from_number(n).map_err(self.cx.map_message())
        })
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::F32, hint), Value::Number(Number::F32(n)) => Ok(*n))
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, C::Error> {
        ensure!(self, hint, ExpectedNumber(NumberHint::F64, hint), Value::Number(Number::F64(n)) => Ok(*n))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], C::Error> {
        ensure!(self, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            <[u8; N]>::try_from(bytes.as_slice()).map_err(|_| self.cx.message(ErrorMessage::ArrayOutOfBounds))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        ensure!(self, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            visitor.visit_borrowed(self.cx, bytes)
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
    {
        ensure!(self, hint, ExpectedString(hint), Value::String(string) => {
            visitor.visit_borrowed(self.cx, string)
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_option(self) -> Result<Option<Self::DecodeSome>, C::Error> {
        ensure!(self, hint, ExpectedOption(hint), Value::Option(option) => {
            Ok(option.as_ref().map(|some| ValueDecoder::new(self.cx, some)))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_pack(self) -> Result<Self::DecodePack, C::Error> {
        ensure!(self, hint, ExpectedPack(hint), Value::Bytes(pack) => {
            Ok(StorageDecoder::new(self.cx, SliceReader::new(pack)))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_sequence(self) -> Result<Self::DecodeSequence, C::Error> {
        ensure!(self, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            Ok(IterValueDecoder::new(self.cx, sequence))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_tuple(self, _: usize) -> Result<Self::DecodeTuple, C::Error> {
        ensure!(self, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            Ok(IterValueDecoder::new(self.cx, sequence))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_map(self) -> Result<Self::DecodeMap, C::Error> {
        ensure!(self, hint, ExpectedMap(hint), Value::Map(map) => {
            Ok(IterValuePairsDecoder::new(self.cx, map))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_struct(self, _: Option<usize>) -> Result<Self::DecodeStruct, C::Error> {
        ensure!(self, hint, ExpectedMap(hint), Value::Map(st) => {
            Ok(IterValuePairsDecoder::new(self.cx, st))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_variant(self) -> Result<Self::DecodeVariant, C::Error> {
        ensure!(self, hint, ExpectedVariant(hint), Value::Variant(st) => {
            Ok(IterValueVariantDecoder::new(self.cx, st))
        })
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, Self::Cx>,
    {
        match self.value {
            Value::Unit => visitor.visit_unit(self.cx),
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
            #[cfg(feature = "alloc")]
            Value::Bytes(bytes) => {
                let visitor = visitor.visit_bytes(self.cx, SizeHint::Exact(bytes.len()))?;
                visitor.visit_borrowed(self.cx, bytes)
            }
            #[cfg(feature = "alloc")]
            Value::String(string) => {
                let visitor = visitor.visit_string(self.cx, SizeHint::Exact(string.len()))?;
                visitor.visit_borrowed(self.cx, string)
            }
            #[cfg(feature = "alloc")]
            Value::Sequence(values) => {
                visitor.visit_sequence(self.cx, IterValueDecoder::<F, _>::new(self.cx, values))
            }
            #[cfg(feature = "alloc")]
            Value::Map(values) => {
                visitor.visit_map(self.cx, IterValuePairsDecoder::<F, _>::new(self.cx, values))
            }
            #[cfg(feature = "alloc")]
            Value::Variant(variant) => visitor.visit_variant(
                self.cx,
                IterValueVariantDecoder::<F, _>::new(self.cx, variant),
            ),
            #[cfg(feature = "alloc")]
            Value::Option(option) => visitor.visit_option(
                self.cx,
                option
                    .as_ref()
                    .map(|value| ValueDecoder::<F, _>::new(self.cx, value)),
            ),
        }
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> AsDecoder for ValueDecoder<'a, 'de, F, C> {
    type Cx = C;
    type Decoder<'this> = ValueDecoder<'a, 'this, F, C> where Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, C::Error> {
        Ok(ValueDecoder::new(self.cx, self.value))
    }
}

/// A decoder over a simple value iterator.

pub struct IterValueDecoder<'a, 'de, const F: Options, C: ?Sized> {
    cx: &'a C,
    iter: slice::Iter<'de, Value>,
}

#[cfg(feature = "alloc")]
impl<'a, 'de, const F: Options, C: ?Sized> IterValueDecoder<'a, 'de, F, C> {
    #[inline]
    fn new(cx: &'a C, values: &'de [Value]) -> Self {
        Self {
            cx,
            iter: values.iter(),
        }
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> PackDecoder<'de>
    for IterValueDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeNext<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self.iter.next() {
            Some(value) => Ok(ValueDecoder::new(self.cx, value)),
            None => Err(self.cx.message(ErrorMessage::ExpectedPackValue)),
        }
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> SequenceDecoder<'de>
    for IterValueDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeNext<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        match self.iter.next() {
            Some(value) => Ok(Some(ValueDecoder::new(self.cx, value))),
            None => Ok(None),
        }
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairsDecoder<'a, 'de, const F: Options, C: ?Sized> {
    cx: &'a C,
    iter: slice::Iter<'de, (Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<'a, 'de, const F: Options, C: ?Sized> IterValuePairsDecoder<'a, 'de, F, C> {
    #[inline]
    fn new(cx: &'a C, values: &'de [(Value, Value)]) -> Self {
        Self {
            cx,
            iter: values.iter(),
        }
    }
}

#[musli::map_decoder]
impl<'a, 'de, C: ?Sized + Context, const F: Options> MapDecoder<'de>
    for IterValuePairsDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeEntry<'this> = IterValuePairDecoder<'a, 'de, F, C>
    where
        Self: 'this;
    type IntoMapEntries = Self;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn into_map_entries(self) -> Result<Self::IntoMapEntries, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };

        Ok(Some(IterValuePairDecoder::new(self.cx, value)))
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> MapEntriesDecoder<'de>
    for IterValuePairsDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeMapEntryKey<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;
    type DecodeMapEntryValue<'this> = ValueDecoder<'a, 'de, F, C> where Self: 'this;

    #[inline]
    fn decode_map_entry_key(&mut self) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        let Some((name, _)) = self.iter.clone().next() else {
            return Ok(None);
        };

        Ok(Some(ValueDecoder::new(self.cx, name)))
    }

    #[inline]
    fn decode_map_entry_value(&mut self) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        let Some((_, value)) = self.iter.next() else {
            return Err(self.cx.message(ErrorMessage::ExpectedMapValue));
        };

        Ok(ValueDecoder::new(self.cx, value))
    }

    #[inline]
    fn skip_map_entry_value(&mut self) -> Result<bool, C::Error> {
        Ok(true)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> MapEntryDecoder<'de>
    for IterValuePairDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeMapKey<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;
    type DecodeMapValue = ValueDecoder<'a, 'de, F, C>;

    #[inline]
    fn decode_map_key(&mut self) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.0))
    }

    #[inline]
    fn decode_map_value(self) -> Result<Self::DecodeMapValue, C::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.1))
    }

    #[inline]
    fn skip_map_value(self) -> Result<bool, C::Error> {
        Ok(true)
    }
}

#[musli::struct_decoder]
impl<'a, 'de, C: ?Sized + Context, const F: Options> StructDecoder<'de>
    for IterValuePairsDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeField<'this> = IterValuePairDecoder<'a, 'de, F, C>
    where
        Self: 'this;
    type IntoStructFields = Self;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        MapDecoder::size_hint(self)
    }

    #[inline]
    fn into_struct_fields(self) -> Result<Self::IntoStructFields, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_field(&mut self) -> Result<Option<Self::DecodeField<'_>>, C::Error> {
        MapDecoder::decode_entry(self)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        MapDecoder::end(self)
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> StructFieldsDecoder<'de>
    for IterValuePairsDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeStructFieldName<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;
    type DecodeStructFieldValue<'this> = ValueDecoder<'a, 'de, F, C> where Self: 'this;

    #[inline]
    fn decode_struct_field_name(&mut self) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        let Some((name, _)) = self.iter.clone().next() else {
            return Err(self.cx.message(ErrorMessage::ExpectedFieldName));
        };

        Ok(ValueDecoder::new(self.cx, name))
    }

    #[inline]
    fn decode_struct_field_value(&mut self) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        let Some((_, value)) = self.iter.next() else {
            return Err(self.cx.message(ErrorMessage::ExpectedFieldValue));
        };

        Ok(ValueDecoder::new(self.cx, value))
    }

    #[inline]
    fn skip_struct_field_value(&mut self) -> Result<bool, C::Error> {
        Ok(self.iter.next().is_some())
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> StructFieldDecoder<'de>
    for IterValuePairDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeFieldName<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;

    type DecodeFieldValue = ValueDecoder<'a, 'de, F, C>;

    #[inline]
    fn decode_field_name(&mut self) -> Result<Self::DecodeFieldName<'_>, C::Error> {
        MapEntryDecoder::decode_map_key(self)
    }

    #[inline]
    fn decode_field_value(self) -> Result<Self::DecodeFieldValue, C::Error> {
        MapEntryDecoder::decode_map_value(self)
    }

    #[inline]
    fn skip_field_value(self) -> Result<bool, C::Error> {
        MapEntryDecoder::skip_map_value(self)
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairDecoder<'a, 'de, const F: Options, C: ?Sized> {
    cx: &'a C,
    pair: &'de (Value, Value),
}

impl<'a, 'de, const F: Options, C: ?Sized> IterValuePairDecoder<'a, 'de, F, C> {
    #[inline]
    const fn new(cx: &'a C, pair: &'de (Value, Value)) -> Self {
        Self { cx, pair }
    }
}

/// A decoder over a simple value pair as a variant.
pub struct IterValueVariantDecoder<'a, 'de, const F: Options, C: ?Sized> {
    cx: &'a C,
    pair: &'de (Value, Value),
}

#[cfg(feature = "alloc")]
impl<'a, 'de, const F: Options, C: ?Sized> IterValueVariantDecoder<'a, 'de, F, C> {
    #[inline]
    const fn new(cx: &'a C, pair: &'de (Value, Value)) -> Self {
        Self { cx, pair }
    }
}

impl<'a, 'de, C: ?Sized + Context, const F: Options> VariantDecoder<'de>
    for IterValueVariantDecoder<'a, 'de, F, C>
{
    type Cx = C;
    type DecodeTag<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;
    type DecodeValue<'this> = ValueDecoder<'a, 'de, F, C>
    where
        Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.0))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        Ok(ValueDecoder::new(self.cx, &self.pair.1))
    }

    #[inline]
    fn skip_value(&mut self) -> Result<bool, C::Error> {
        Ok(true)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
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
