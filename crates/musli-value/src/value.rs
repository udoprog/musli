#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{AsDecoder, Decode, Decoder, NumberVisitor, Visitor};
#[cfg(feature = "alloc")]
use musli::de::{
    MapDecoder, MapEntryDecoder, SequenceDecoder, SizeHint, ValueVisitor, VariantDecoder,
};
use musli::en::{Encode, Encoder};
#[cfg(feature = "alloc")]
use musli::en::{MapEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;
use musli_utils::Options;

use crate::de::ValueDecoder;
use crate::type_hint::{NumberHint, TypeHint};

/// A dynamic value capable of representing any [Müsli] type whether it be
/// complex or simple.
///
/// [Müsli]: https://github.com/udoprog/musli
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Value {
    /// The default unit value.
    Unit,
    /// A boolean value.
    Bool(bool),
    /// A character.
    Char(char),
    /// A number.
    Number(Number),
    /// An array.
    #[cfg(feature = "alloc")]
    Bytes(Vec<u8>),
    /// A string in a value.
    #[cfg(feature = "alloc")]
    String(String),
    /// A unit value.
    #[cfg(feature = "alloc")]
    Sequence(Vec<Value>),
    /// A pair stored in the value.
    #[cfg(feature = "alloc")]
    Map(Vec<(Value, Value)>),
    /// A variant pair. The first value identifies the variant, the second value
    /// contains the value of the variant.
    #[cfg(feature = "alloc")]
    Variant(Box<(Value, Value)>),
    /// An optional value.
    #[cfg(feature = "alloc")]
    Option(Option<Box<Value>>),
}

impl Value {
    /// Construct a [AsValueDecoder] implementation out of this value which
    /// emits the specified error `E`.
    #[inline]
    pub fn into_value_decoder<const OPT: Options, C: ?Sized>(
        self,
        cx: &C,
    ) -> AsValueDecoder<'_, OPT, C> {
        AsValueDecoder::new(cx, self)
    }

    /// Get a decoder associated with a value.
    #[inline]
    pub(crate) fn decoder<'a, 'de, const OPT: Options, C: ?Sized>(
        &'de self,
        cx: &'a C,
    ) -> ValueDecoder<'a, 'de, OPT, C> {
        ValueDecoder::new(cx, self)
    }

    /// Get the type hint corresponding to the value.
    pub(crate) fn type_hint(&self) -> TypeHint {
        match self {
            Value::Unit => TypeHint::Unit,
            Value::Bool(..) => TypeHint::Bool,
            Value::Char(..) => TypeHint::Char,
            Value::Number(number) => TypeHint::Number(number.type_hint()),
            #[cfg(feature = "alloc")]
            Value::Bytes(bytes) => TypeHint::Bytes(SizeHint::Exact(bytes.len())),
            #[cfg(feature = "alloc")]
            Value::String(string) => TypeHint::String(SizeHint::Exact(string.len())),
            #[cfg(feature = "alloc")]
            Value::Sequence(sequence) => TypeHint::Sequence(SizeHint::Exact(sequence.len())),
            #[cfg(feature = "alloc")]
            Value::Map(map) => TypeHint::Map(SizeHint::Exact(map.len())),
            #[cfg(feature = "alloc")]
            Value::Variant(..) => TypeHint::Variant,
            #[cfg(feature = "alloc")]
            Value::Option(..) => TypeHint::Option,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Number {
    /// `u8`
    U8(u8),
    /// `u16`
    U16(u16),
    /// `u32`
    U32(u32),
    /// `u64`
    U64(u64),
    /// `u128`
    U128(u128),
    /// `u8`
    I8(i8),
    /// `u16`
    I16(i16),
    /// `u32`
    I32(i32),
    /// `u64`
    I64(i64),
    /// `u128`
    I128(i128),
    /// `usize`
    Usize(usize),
    /// `isize`
    Isize(isize),
    /// `f32`
    F32(f32),
    /// `f64`
    F64(f64),
}

macro_rules! from {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for Number {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }
    };
}

from!(u8, U8);
from!(u16, U16);
from!(u32, U32);
from!(u64, U64);
from!(u128, U128);
from!(i8, I8);
from!(i16, I16);
from!(i32, I32);
from!(i64, I64);
from!(i128, I128);
from!(usize, Usize);
from!(isize, Isize);
from!(f32, F32);
from!(f64, F64);

impl<M> Encode<M> for Number {
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        match self {
            Number::U8(n) => encoder.encode_u8(*n),
            Number::U16(n) => encoder.encode_u16(*n),
            Number::U32(n) => encoder.encode_u32(*n),
            Number::U64(n) => encoder.encode_u64(*n),
            Number::U128(n) => encoder.encode_u128(*n),
            Number::I8(n) => encoder.encode_i8(*n),
            Number::I16(n) => encoder.encode_i16(*n),
            Number::I32(n) => encoder.encode_i32(*n),
            Number::I64(n) => encoder.encode_i64(*n),
            Number::I128(n) => encoder.encode_i128(*n),
            Number::Usize(n) => encoder.encode_usize(*n),
            Number::Isize(n) => encoder.encode_isize(*n),
            Number::F32(n) => encoder.encode_f32(*n),
            Number::F64(n) => encoder.encode_f64(*n),
        }
    }
}

impl Number {
    /// Get the type hint for the number.
    pub(crate) fn type_hint(&self) -> NumberHint {
        match self {
            Number::U8(_) => NumberHint::U8,
            Number::U16(_) => NumberHint::U16,
            Number::U32(_) => NumberHint::U32,
            Number::U64(_) => NumberHint::U64,
            Number::U128(_) => NumberHint::U128,
            Number::I8(_) => NumberHint::I8,
            Number::I16(_) => NumberHint::I16,
            Number::I32(_) => NumberHint::I32,
            Number::I64(_) => NumberHint::I64,
            Number::I128(_) => NumberHint::I128,
            Number::Usize(_) => NumberHint::Usize,
            Number::Isize(_) => NumberHint::Isize,
            Number::F32(_) => NumberHint::F32,
            Number::F64(_) => NumberHint::F64,
        }
    }
}

struct AnyVisitor;

#[musli::visitor]
impl<'de, C: ?Sized + Context> Visitor<'de, C> for AnyVisitor {
    type Ok = Value;
    #[cfg(feature = "alloc")]
    type String = StringVisitor;
    #[cfg(feature = "alloc")]
    type Bytes = BytesVisitor;
    #[cfg(feature = "alloc")]
    type Number = ValueNumberVisitor;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be decoded into dynamic container")
    }

    #[inline]
    fn visit_unit(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(Value::Unit)
    }

    #[inline]
    fn visit_bool(self, _: &C, value: bool) -> Result<Self::Ok, C::Error> {
        Ok(Value::Bool(value))
    }

    #[inline]
    fn visit_char(self, _: &C, value: char) -> Result<Self::Ok, C::Error> {
        Ok(Value::Char(value))
    }

    #[inline]
    fn visit_u8(self, _: &C, value: u8) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U8(value)))
    }

    #[inline]
    fn visit_u16(self, _: &C, value: u16) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U16(value)))
    }

    #[inline]
    fn visit_u32(self, _: &C, value: u32) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U32(value)))
    }

    #[inline]
    fn visit_u64(self, _: &C, value: u64) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U64(value)))
    }

    #[inline]
    fn visit_u128(self, _: &C, value: u128) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U128(value)))
    }

    #[inline]
    fn visit_i8(self, _: &C, value: i8) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I8(value)))
    }

    #[inline]
    fn visit_i16(self, _: &C, value: i16) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I16(value)))
    }

    #[inline]
    fn visit_i32(self, _: &C, value: i32) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I32(value)))
    }

    #[inline]
    fn visit_i64(self, _: &C, value: i64) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I64(value)))
    }

    #[inline]
    fn visit_i128(self, _: &C, value: i128) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I128(value)))
    }

    #[inline]
    fn visit_usize(self, _: &C, value: usize) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::Usize(value)))
    }

    #[inline]
    fn visit_isize(self, _: &C, value: isize) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::Isize(value)))
    }

    #[inline]
    fn visit_f32(self, _: &C, value: f32) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::F32(value)))
    }

    #[inline]
    fn visit_f64(self, _: &C, value: f64) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::F64(value)))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_option<D>(self, _: &C, decoder: Option<D>) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, Cx = C, Error = C::Error>,
    {
        match decoder {
            Some(decoder) => Ok(Value::Option(Some(Box::new(decoder.decode::<Value>()?)))),
            None => Ok(Value::Option(None)),
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_sequence<D>(self, _: &C, seq: &mut D) -> Result<Self::Ok, C::Error>
    where
        D: SequenceDecoder<'de, Cx = C>,
    {
        let mut out = Vec::with_capacity(seq.size_hint().or_default());

        while let Some(item) = seq.next()? {
            out.push(item);
        }

        Ok(Value::Sequence(out))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_map<D>(self, _: &C, map: &mut D) -> Result<Self::Ok, C::Error>
    where
        D: MapDecoder<'de, Cx = C>,
    {
        let mut out = Vec::with_capacity(map.size_hint().or_default());

        while let Some(mut entry) = map.decode_entry()? {
            let first = entry.decode_map_key()?.decode()?;
            let second = entry.decode_map_value()?.decode()?;
            out.push((first, second));
        }

        Ok(Value::Map(out))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_bytes(self, _: &C, _: SizeHint) -> Result<Self::Bytes, C::Error> {
        Ok(BytesVisitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_string(self, _: &C, _: SizeHint) -> Result<Self::String, C::Error> {
        Ok(StringVisitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_number(self, _: &C) -> Result<Self::Number, C::Error> {
        Ok(ValueNumberVisitor)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_variant<D>(self, _: &C, variant: &mut D) -> Result<Self::Ok, C::Error>
    where
        D: VariantDecoder<'de, Cx = C>,
    {
        let first = variant.decode_tag()?.decode()?;
        let second = variant.decode_value()?.decode()?;
        Ok(Value::Variant(Box::new((first, second))))
    }
}

impl<'de, M> Decode<'de, M> for Value {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        decoder.decode_any(AnyVisitor)
    }
}

#[cfg(feature = "alloc")]
struct BytesVisitor;

#[cfg(feature = "alloc")]
impl<'de, C: ?Sized + Context> ValueVisitor<'de, C, [u8]> for BytesVisitor {
    type Ok = Value;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "bytes")
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn visit_owned(self, _: &C, bytes: Vec<u8>) -> Result<Self::Ok, C::Error> {
        Ok(Value::Bytes(bytes))
    }

    #[inline]
    fn visit_ref(self, _: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        Ok(Value::Bytes(bytes.to_vec()))
    }
}

#[cfg(feature = "alloc")]
struct StringVisitor;

#[cfg(feature = "alloc")]
impl<'de, C: ?Sized + Context> ValueVisitor<'de, C, str> for StringVisitor {
    type Ok = Value;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_owned(self, _: &C, string: String) -> Result<Self::Ok, C::Error> {
        Ok(Value::String(string))
    }

    #[inline]
    fn visit_ref(self, _: &C, string: &str) -> Result<Self::Ok, C::Error> {
        Ok(Value::String(string.to_owned()))
    }
}

struct ValueNumberVisitor;

impl<'de, C: ?Sized + Context> NumberVisitor<'de, C> for ValueNumberVisitor {
    type Ok = Value;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "any supported number")
    }

    #[inline]
    fn visit_u8(self, _: &C, value: u8) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U8(value)))
    }

    #[inline]
    fn visit_u16(self, _: &C, value: u16) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U16(value)))
    }

    #[inline]
    fn visit_u32(self, _: &C, value: u32) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U32(value)))
    }

    #[inline]
    fn visit_u64(self, _: &C, value: u64) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U64(value)))
    }

    #[inline]
    fn visit_u128(self, _: &C, value: u128) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::U128(value)))
    }

    #[inline]
    fn visit_i8(self, _: &C, value: i8) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I8(value)))
    }

    #[inline]
    fn visit_i16(self, _: &C, value: i16) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I16(value)))
    }

    #[inline]
    fn visit_i32(self, _: &C, value: i32) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I32(value)))
    }

    #[inline]
    fn visit_i64(self, _: &C, value: i64) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I64(value)))
    }

    #[inline]
    fn visit_i128(self, _: &C, value: i128) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::I128(value)))
    }

    #[inline]
    fn visit_f32(self, _: &C, value: f32) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::F32(value)))
    }

    #[inline]
    fn visit_f64(self, _: &C, value: f64) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::F64(value)))
    }

    #[inline]
    fn visit_usize(self, _: &C, value: usize) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::Usize(value)))
    }

    #[inline]
    fn visit_isize(self, _: &C, value: isize) -> Result<Self::Ok, C::Error> {
        Ok(Value::Number(Number::Isize(value)))
    }
}

impl<M> Encode<M> for Value {
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        match self {
            Value::Unit => encoder.encode_unit(),
            Value::Bool(b) => encoder.encode_bool(*b),
            Value::Char(c) => encoder.encode_char(*c),
            Value::Number(n) => encoder.encode(n),
            #[cfg(feature = "alloc")]
            Value::Bytes(bytes) => encoder.encode_bytes(bytes),
            #[cfg(feature = "alloc")]
            Value::String(string) => encoder.encode_string(string),
            #[cfg(feature = "alloc")]
            Value::Sequence(values) => {
                use musli::hint::SequenceHint;

                let hint = SequenceHint::with_size(values.len());

                encoder.encode_sequence_fn(&hint, |sequence| {
                    for value in values {
                        sequence.encode_element()?.encode(value)?;
                    }

                    Ok(())
                })
            }
            #[cfg(feature = "alloc")]
            Value::Map(values) => {
                use musli::hint::MapHint;

                let hint = MapHint::with_size(values.len());

                encoder.encode_map_fn(&hint, |map| {
                    for (first, second) in values {
                        map.insert_entry(first, second)?;
                    }

                    Ok(())
                })
            }
            #[cfg(feature = "alloc")]
            Value::Variant(variant) => {
                let (tag, variant) = &**variant;
                let encoder = encoder.encode_variant()?;
                encoder.insert_variant(tag, variant)
            }
            #[cfg(feature = "alloc")]
            Value::Option(option) => match option {
                Some(value) => encoder.encode_some()?.encode(&**value),
                None => encoder.encode_none(),
            },
        }
    }
}

/// Value's [AsDecoder] implementation.
pub struct AsValueDecoder<'a, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    value: Value,
}

impl<'a, const OPT: Options, C: ?Sized> AsValueDecoder<'a, OPT, C> {
    /// Construct a new buffered value decoder.
    #[inline]
    pub fn new(cx: &'a C, value: Value) -> Self {
        Self { cx, value }
    }
}

impl<'a, const OPT: Options, C: ?Sized + Context> AsDecoder for AsValueDecoder<'a, OPT, C> {
    type Cx = C;
    type Decoder<'this> = ValueDecoder<'a, 'this, OPT, C> where Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, C::Error> {
        Ok(self.value.decoder(self.cx))
    }
}
