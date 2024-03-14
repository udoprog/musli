use core::fmt;
use core::marker;
use core::slice;

#[cfg(feature = "alloc")]
use musli::de::ValueVisitor;
use musli::de::{
    AsDecoder, Decoder, NumberHint, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder,
    SizeHint, TypeHint, VariantDecoder, Visitor,
};
use musli::mode::Mode;
use musli::Context;
use musli_storage::de::StorageDecoder;
use musli_storage::options::Options;
use musli_storage::reader::{SliceReader, SliceUnderflow};

use crate::error::ErrorKind;
use crate::value::{Number, Value};
use crate::AsValueDecoder;

/// Encoder for a single value.
pub struct ValueDecoder<'de, const F: Options, E> {
    value: &'de Value,
    _marker: marker::PhantomData<E>,
}

impl<'de, const F: Options, E> ValueDecoder<'de, F, E> {
    #[inline]
    pub(crate) const fn new(value: &'de Value) -> Self {
        Self {
            value,
            _marker: marker::PhantomData,
        }
    }
}

macro_rules! ensure {
    ($self:expr, $cx:expr, $hint:ident, $ident:ident $tt:tt, $pat:pat => $block:expr) => {
        match $self.value {
            $pat => $block,
            value => {
                let $hint = value.type_hint();
                return Err($cx.report(ErrorKind::$ident $tt));
            }
        }
    };
}

#[musli::decoder]
impl<'de, const F: Options, E> Decoder<'de> for ValueDecoder<'de, F, E>
where
    E: musli::error::Error + From<ErrorKind> + From<SliceUnderflow>,
{
    type Error = E;
    type Buffer = AsValueDecoder<F, E>;
    type Some = Self;
    type Pack = StorageDecoder<SliceReader<'de>, F, E>;
    type Sequence = IterValueDecoder<'de, F, E>;
    type Tuple = IterValueDecoder<'de, F, E>;
    type Map = IterValuePairsDecoder<'de, F, E>;
    type Struct = IterValuePairsDecoder<'de, F, E>;
    type Variant = IterValueVariantDecoder<'de, F, E>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot be decoded from value")
    }

    #[inline]
    fn type_hint<C>(&mut self, _: &C) -> Result<TypeHint, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self.value.type_hint())
    }

    #[inline]
    fn decode_buffer<M, C>(self, _: &C) -> Result<Self::Buffer, C::Error>
    where
        C: Context<Input = Self::Error>,
        M: Mode,
    {
        Ok(AsValueDecoder::new(self.value.clone()))
    }

    #[inline]
    fn decode_unit<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedUnit(hint), Value::Unit => Ok(()))
    }

    #[inline]
    fn decode_bool<C>(self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedBool(hint), Value::Bool(b) => Ok(*b))
    }

    #[inline]
    fn decode_char<C>(self, cx: &C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedChar(hint), Value::Char(c) => Ok(*c))
    }

    #[inline]
    fn decode_u8<C>(self, cx: &C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U8, hint), Value::Number(n) => {
            u8::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_u16<C>(self, cx: &C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U16, hint), Value::Number(n) => {
            u16::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_u32<C>(self, cx: &C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U32, hint), Value::Number(n) => {
            u32::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_u64<C>(self, cx: &C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U64, hint), Value::Number(n) => {
            u64::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_u128<C>(self, cx: &C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::U128, hint), Value::Number(n) => {
            u128::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_i8<C>(self, cx: &C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I8, hint), Value::Number(n) => {
            i8::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_i16<C>(self, cx: &C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I16, hint), Value::Number(n) => {
            i16::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_i32<C>(self, cx: &C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I32, hint), Value::Number(n) => {
            i32::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_i64<C>(self, cx: &C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I64, hint), Value::Number(n) => {
            i64::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_i128<C>(self, cx: &C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::I128, hint), Value::Number(n) => {
            i128::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_usize<C>(self, cx: &C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::Usize, hint), Value::Number(n) => {
            usize::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_isize<C>(self, cx: &C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::Isize, hint), Value::Number(n) => {
            isize::from_number(n).map_err(cx.map())
        })
    }

    #[inline]
    fn decode_f32<C>(self, cx: &C) -> Result<f32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::F32, hint), Value::Number(Number::F32(n)) => Ok(*n))
    }

    #[inline]
    fn decode_f64<C>(self, cx: &C) -> Result<f64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedNumber(NumberHint::F64, hint), Value::Number(Number::F64(n)) => Ok(*n))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_array<C, const N: usize>(self, cx: &C) -> Result<[u8; N], C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            <[u8; N]>::try_from(bytes.as_slice()).map_err(|_| cx.report(ErrorKind::ArrayOutOfBounds))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_bytes<C, V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, [u8]>,
    {
        ensure!(self, cx, hint, ExpectedBytes(hint), Value::Bytes(bytes) => {
            visitor.visit_borrowed(cx, bytes)
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_string<C, V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, str>,
    {
        ensure!(self, cx, hint, ExpectedString(hint), Value::String(string) => {
            visitor.visit_borrowed(cx, string)
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_option<C>(self, cx: &C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedOption(hint), Value::Option(option) => {
            Ok(option.as_ref().map(|some| ValueDecoder::new(some)))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_pack<C>(self, cx: &C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedPack(hint), Value::Bytes(pack) => {
            Ok(StorageDecoder::new(SliceReader::new(pack)))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_sequence<C>(self, cx: &C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            Ok(IterValueDecoder::new(sequence))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_tuple<C>(self, cx: &C, _: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedSequence(hint), Value::Sequence(sequence) => {
            Ok(IterValueDecoder::new(sequence))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_map<C>(self, cx: &C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedMap(hint), Value::Map(map) => {
            Ok(IterValuePairsDecoder::new(map))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_struct<C>(self, cx: &C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedMap(hint), Value::Map(st) => {
            Ok(IterValuePairsDecoder::new(st))
        })
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_variant<C>(self, cx: &C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        ensure!(self, cx, hint, ExpectedVariant(hint), Value::Variant(st) => {
            Ok(IterValueVariantDecoder::new(st))
        })
    }

    #[inline]
    fn decode_any<C, V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: Visitor<'de, Error = Self::Error>,
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
                visitor.visit_sequence(cx, IterValueDecoder::<F, _>::new(values))
            }
            #[cfg(feature = "alloc")]
            Value::Map(values) => visitor.visit_map(cx, IterValuePairsDecoder::<F, _>::new(values)),
            #[cfg(feature = "alloc")]
            Value::Variant(variant) => {
                visitor.visit_variant(cx, IterValueVariantDecoder::<F, _>::new(variant))
            }
            #[cfg(feature = "alloc")]
            Value::Option(option) => visitor.visit_option(
                cx,
                option
                    .as_ref()
                    .map(|value| ValueDecoder::<F, _>::new(value)),
            ),
        }
    }
}

impl<'a, const F: Options, E> AsDecoder for ValueDecoder<'a, F, E>
where
    E: musli::error::Error + From<ErrorKind> + From<SliceUnderflow>,
{
    type Error = E;
    type Decoder<'this> = ValueDecoder<'this, F, E> where Self: 'this;

    #[inline]
    fn as_decoder<C>(&self, _: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(ValueDecoder::new(self.value))
    }
}

/// A decoder over a simple value iterator.

pub struct IterValueDecoder<'de, const F: Options, E> {
    iter: slice::Iter<'de, Value>,
    _marker: marker::PhantomData<E>,
}

impl<'de, const F: Options, E> IterValueDecoder<'de, F, E> {
    #[cfg(feature = "alloc")]
    #[inline]
    fn new(values: &'de [Value]) -> Self {
        Self {
            iter: values.iter(),
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, const F: Options, E> PackDecoder<'de> for IterValueDecoder<'de, F, E>
where
    E: musli::error::Error + From<ErrorKind> + From<SliceUnderflow>,
{
    type Error = E;

    type Decoder<'this> = ValueDecoder<'de, F, E>
    where
        Self: 'this;

    #[inline]
    fn next<C>(&mut self, cx: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self.iter.next() {
            Some(value) => Ok(ValueDecoder::new(value)),
            None => Err(cx.report(ErrorKind::ExpectedPackValue)),
        }
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, const F: Options, E> SequenceDecoder<'de> for IterValueDecoder<'de, F, E>
where
    E: musli::error::Error + From<ErrorKind> + From<SliceUnderflow>,
{
    type Error = E;

    type Decoder<'this> = ValueDecoder<'de, F, E>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self.iter.next() {
            Some(value) => Ok(Some(ValueDecoder::new(value))),
            None => Ok(None),
        }
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairsDecoder<'de, const F: Options, E> {
    iter: slice::Iter<'de, (Value, Value)>,
    _marker: marker::PhantomData<E>,
}

impl<'de, const F: Options, E> IterValuePairsDecoder<'de, F, E> {
    #[cfg(feature = "alloc")]
    #[inline]
    fn new(values: &'de [(Value, Value)]) -> Self {
        Self {
            iter: values.iter(),
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, const F: Options, E> PairsDecoder<'de> for IterValuePairsDecoder<'de, F, E>
where
    E: musli::error::Error + From<ErrorKind> + From<SliceUnderflow>,
{
    type Error = E;

    type Decoder<'this> = IterValuePairDecoder<'de, F, E>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.iter.size_hint().1)
    }

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self.iter.next().map(IterValuePairDecoder::new))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

/// A decoder over a simple value pair iterator.
pub struct IterValuePairDecoder<'de, const F: Options, E> {
    pair: &'de (Value, Value),
    _marker: marker::PhantomData<E>,
}

impl<'de, const F: Options, E> IterValuePairDecoder<'de, F, E> {
    #[inline]
    const fn new(pair: &'de (Value, Value)) -> Self {
        Self {
            pair,
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, const F: Options, E> PairDecoder<'de> for IterValuePairDecoder<'de, F, E>
where
    E: musli::error::Error + From<ErrorKind> + From<SliceUnderflow>,
{
    type Error = E;

    type First<'this> = ValueDecoder<'de, F, E>
    where
        Self: 'this;

    type Second = ValueDecoder<'de, F, E>;

    #[inline]
    fn first<C>(&mut self, _: &C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(ValueDecoder::new(&self.pair.0))
    }

    #[inline]
    fn second<C>(self, _: &C) -> Result<Self::Second, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(ValueDecoder::new(&self.pair.1))
    }

    #[inline]
    fn skip_second<C>(self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(true)
    }
}

/// A decoder over a simple value pair as a variant.
pub struct IterValueVariantDecoder<'de, const F: Options, E> {
    pair: &'de (Value, Value),
    _marker: marker::PhantomData<E>,
}

impl<'de, const F: Options, E> IterValueVariantDecoder<'de, F, E> {
    #[cfg(feature = "alloc")]
    #[inline]
    const fn new(pair: &'de (Value, Value)) -> Self {
        Self {
            pair,
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, const F: Options, E> VariantDecoder<'de> for IterValueVariantDecoder<'de, F, E>
where
    E: musli::error::Error + From<ErrorKind> + From<SliceUnderflow>,
{
    type Error = E;

    type Tag<'this> = ValueDecoder<'de, F, E>
    where
        Self: 'this;

    type Variant<'this> = ValueDecoder<'de, F, E>
    where
        Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(ValueDecoder::new(&self.pair.0))
    }

    #[inline]
    fn variant<C>(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(ValueDecoder::new(&self.pair.1))
    }

    #[inline]
    fn skip_variant<C>(&mut self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(true)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

/// Conversion trait for numbers.
trait FromNumber: Sized {
    const NUMBER_HINT: NumberHint;

    fn from_number(number: &Number) -> Result<Self, ErrorKind>;
}

macro_rules! integer_from {
    ($ty:ty, $variant:ident) => {
        impl FromNumber for $ty {
            const NUMBER_HINT: NumberHint = NumberHint::$variant;

            #[inline]
            fn from_number(number: &Number) -> Result<Self, ErrorKind> {
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
                    None => Err(ErrorKind::ExpectedNumber(
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
            fn from_number(number: &Number) -> Result<Self, ErrorKind> {
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
                    None => Err(ErrorKind::ExpectedNumber(
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
