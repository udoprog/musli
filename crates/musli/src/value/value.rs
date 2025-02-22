use core::cmp::Ordering;
use core::fmt;
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use crate::alloc::{AllocError, System};
use crate::alloc::{Box, String, Vec};
use crate::de::{AsDecoder, Decode, Decoder, Visitor};
use crate::de::{
    EntryDecoder, MapDecoder, SequenceDecoder, SizeHint, UnsizedVisitor, VariantDecoder,
};
use crate::en::{Encode, Encoder};
use crate::en::{MapEncoder, SequenceEncoder, VariantEncoder};
use crate::{Allocator, Context, Options};

use super::de::ValueDecoder;
use super::type_hint::{NumberHint, TypeHint};

/// A dynamic value capable of representing any [Müsli] type whether it be
/// complex or simple.
///
/// [Müsli]: https://github.com/udoprog/musli
#[non_exhaustive]
pub enum Value<A>
where
    A: Allocator,
{
    /// The default unit value.
    Unit,
    /// A boolean value.
    Bool(bool),
    /// A character.
    Char(char),
    /// A number.
    Number(Number),
    /// An array.
    Bytes(Vec<u8, A>),
    /// A string in a value.
    String(String<A>),
    /// A unit value.
    Sequence(Vec<Value<A>, A>),
    /// A pair stored in the value.
    Map(Vec<(Value<A>, Value<A>), A>),
    /// A variant pair. The first value identifies the variant, the second value
    /// contains the value of the variant.
    Variant(Box<(Value<A>, Value<A>), A>),
    /// An optional value.
    Option(Option<Box<Value<A>, A>>),
}

impl<A> Value<A>
where
    A: Allocator,
{
    /// Construct a [`IntoValueDecoder`] implementation out of the current
    /// value.
    #[inline]
    pub fn into_decoder<const OPT: Options, C, M>(self, cx: C) -> IntoValueDecoder<OPT, C, A, M>
    where
        C: Context,
    {
        IntoValueDecoder::new(cx, self)
    }

    /// Construct a [`AsValueDecoder`] implementation out of the current value.
    #[inline]
    pub fn as_decoder<const OPT: Options, C, M>(&self, cx: C) -> AsValueDecoder<'_, OPT, C, A, M>
    where
        C: Context,
    {
        AsValueDecoder::new(cx, self)
    }

    /// Get a decoder associated with a value.
    #[inline]
    pub(crate) fn decoder<const OPT: Options, C, M>(&self, cx: C) -> ValueDecoder<'_, OPT, C, A, M>
    where
        C: Context,
    {
        ValueDecoder::new(cx, self)
    }

    /// Get the type hint corresponding to the value.
    pub(crate) fn type_hint(&self) -> TypeHint {
        match self {
            Value::Unit => TypeHint::Unit,
            Value::Bool(..) => TypeHint::Bool,
            Value::Char(..) => TypeHint::Char,
            Value::Number(number) => TypeHint::Number(number.type_hint()),
            Value::Bytes(bytes) => TypeHint::Bytes(SizeHint::exact(bytes.len())),
            Value::String(string) => TypeHint::String(SizeHint::exact(string.len())),
            Value::Sequence(sequence) => TypeHint::Sequence(SizeHint::exact(sequence.len())),
            Value::Map(map) => TypeHint::Map(SizeHint::exact(map.len())),
            Value::Variant(..) => TypeHint::Variant,
            Value::Option(..) => TypeHint::Option,
        }
    }
}

impl<A> fmt::Debug for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "Unit"),
            Self::Bool(value) => f.debug_tuple("Bool").field(value).finish(),
            Self::Char(value) => f.debug_tuple("Char").field(value).finish(),
            Self::Number(value) => f.debug_tuple("Number").field(value).finish(),
            Self::Bytes(value) => f.debug_tuple("Bytes").field(value).finish(),
            Self::String(value) => f.debug_tuple("String").field(value).finish(),
            Self::Sequence(value) => f.debug_tuple("Sequence").field(value).finish(),
            Self::Map(value) => f.debug_tuple("Map").field(value).finish(),
            Self::Variant(value) => f.debug_tuple("Variant").field(value).finish(),
            Self::Option(value) => f.debug_tuple("Option").field(value).finish(),
        }
    }
}

impl<A> PartialEq for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(lhs), Self::Bool(rhs)) => lhs == rhs,
            (Self::Char(lhs), Self::Char(rhs)) => lhs == rhs,
            (Self::Number(lhs), Self::Number(rhs)) => lhs == rhs,
            (Self::Bytes(lhs), Self::Bytes(rhs)) => lhs == rhs,
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::Sequence(lhs), Self::Sequence(rhs)) => lhs == rhs,
            (Self::Map(lhs), Self::Map(rhs)) => lhs == rhs,
            (Self::Variant(lhs), Self::Variant(rhs)) => lhs == rhs,
            (Self::Option(lhs), Self::Option(rhs)) => lhs == rhs,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<A> PartialOrd for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Unit, Self::Unit) => Some(Ordering::Equal),
            (Self::Bool(lhs), Self::Bool(rhs)) => lhs.partial_cmp(rhs),
            (Self::Char(lhs), Self::Char(rhs)) => lhs.partial_cmp(rhs),
            (Self::Number(lhs), Self::Number(rhs)) => lhs.partial_cmp(rhs),
            (Self::Bytes(lhs), Self::Bytes(rhs)) => lhs.partial_cmp(rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs.partial_cmp(rhs),
            (Self::Sequence(lhs), Self::Sequence(rhs)) => lhs.partial_cmp(rhs),
            (Self::Map(lhs), Self::Map(rhs)) => lhs.partial_cmp(rhs),
            (Self::Variant(lhs), Self::Variant(rhs)) => lhs.partial_cmp(rhs),
            (Self::Option(lhs), Self::Option(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
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
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
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

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
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

#[crate::de::visitor(crate)]
impl<'de, C> Visitor<'de, C> for AnyVisitor
where
    C: Context,
{
    type Ok = Value<C::Allocator>;
    type String = StringVisitor;
    type Bytes = BytesVisitor;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be decoded into dynamic container")
    }

    #[inline]
    fn visit_empty(self, _: C) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Unit)
    }

    #[inline]
    fn visit_bool(self, _: C, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(value))
    }

    #[inline]
    fn visit_char(self, _: C, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Char(value))
    }

    #[inline]
    fn visit_u8(self, _: C, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::U8(value)))
    }

    #[inline]
    fn visit_u16(self, _: C, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::U16(value)))
    }

    #[inline]
    fn visit_u32(self, _: C, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::U32(value)))
    }

    #[inline]
    fn visit_u64(self, _: C, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::U64(value)))
    }

    #[inline]
    fn visit_u128(self, _: C, value: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::U128(value)))
    }

    #[inline]
    fn visit_i8(self, _: C, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::I8(value)))
    }

    #[inline]
    fn visit_i16(self, _: C, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::I16(value)))
    }

    #[inline]
    fn visit_i32(self, _: C, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::I32(value)))
    }

    #[inline]
    fn visit_i64(self, _: C, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::I64(value)))
    }

    #[inline]
    fn visit_i128(self, _: C, value: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::I128(value)))
    }

    #[inline]
    fn visit_usize(self, _: C, value: usize) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::Usize(value)))
    }

    #[inline]
    fn visit_isize(self, _: C, value: isize) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::Isize(value)))
    }

    #[inline]
    fn visit_f32(self, _: C, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::F32(value)))
    }

    #[inline]
    fn visit_f64(self, _: C, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::F64(value)))
    }

    #[inline]
    fn visit_none(self, _: C) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Option(None))
    }

    #[inline]
    fn visit_some<D>(self, decoder: D) -> Result<Self::Ok, Self::Error>
    where
        D: Decoder<'de, Cx = C, Error = C::Error, Allocator = C::Allocator>,
    {
        let cx = decoder.cx();
        let value = decoder.decode::<Value<C::Allocator>>()?;
        let value = Box::new_in(value, cx.alloc()).map_err(cx.map())?;
        Ok(Value::Option(Some(value)))
    }

    #[inline]
    fn visit_sequence<D>(self, seq: &mut D) -> Result<Self::Ok, Self::Error>
    where
        D: ?Sized + SequenceDecoder<'de, Cx = C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = seq.cx();

        let mut out =
            Vec::with_capacity_in(seq.size_hint().or_default(), cx.alloc()).map_err(cx.map())?;

        while let Some(item) = seq.try_next()? {
            out.push(item).map_err(cx.map())?;
        }

        Ok(Value::Sequence(out))
    }

    #[inline]
    fn visit_map<D>(self, map: &mut D) -> Result<Self::Ok, D::Error>
    where
        D: ?Sized + MapDecoder<'de, Cx = C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = map.cx();

        let mut out =
            Vec::with_capacity_in(map.size_hint().or_default(), cx.alloc()).map_err(cx.map())?;

        while let Some(mut entry) = map.decode_entry()? {
            let first = entry.decode_key()?.decode()?;
            let second = entry.decode_value()?.decode()?;
            out.push((first, second)).map_err(cx.map())?;
        }

        Ok(Value::Map(out))
    }

    #[inline]
    fn visit_bytes(self, _: C, _: SizeHint) -> Result<Self::Bytes, Self::Error> {
        Ok(BytesVisitor)
    }

    #[inline]
    fn visit_string(self, _: C, _: SizeHint) -> Result<Self::String, Self::Error> {
        Ok(StringVisitor)
    }

    #[inline]
    fn visit_variant<D>(self, variant: &mut D) -> Result<Self::Ok, Self::Error>
    where
        D: ?Sized + VariantDecoder<'de, Cx = C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let first = variant.decode_tag()?.decode()?;
        let second = variant.decode_value()?.decode()?;
        let value =
            Box::new_in((first, second), variant.cx().alloc()).map_err(variant.cx().map())?;
        Ok(Value::Variant(value))
    }
}

impl<'de, M, A> Decode<'de, M, A> for Value<A>
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        decoder.decode_any(AnyVisitor)
    }
}

struct BytesVisitor;

#[crate::de::unsized_visitor(crate)]
impl<C> UnsizedVisitor<'_, C, [u8]> for BytesVisitor
where
    C: Context,
{
    type Ok = Value<C::Allocator>;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: C, b: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut bytes = Vec::with_capacity_in(b.len(), cx.alloc()).map_err(cx.map())?;
        bytes.extend_from_slice(b).map_err(cx.map())?;
        Ok(Value::Bytes(bytes))
    }
}

struct StringVisitor;

#[crate::de::unsized_visitor(crate)]
impl<C> UnsizedVisitor<'_, C, str> for StringVisitor
where
    C: Context,
{
    type Ok = Value<C::Allocator>;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: C, s: &str) -> Result<Self::Ok, Self::Error> {
        let mut string = String::new_in(cx.alloc());
        string.push_str(s).map_err(cx.map())?;
        Ok(Value::String(string))
    }
}

impl<M, C> Encode<M> for Value<C>
where
    C: Allocator,
{
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        match self {
            Value::Unit => encoder.encode_empty(),
            Value::Bool(b) => encoder.encode_bool(*b),
            Value::Char(c) => encoder.encode_char(*c),
            Value::Number(n) => encoder.encode(n),
            Value::Bytes(bytes) => encoder.encode_bytes(bytes),
            Value::String(string) => encoder.encode_string(string),
            Value::Sequence(values) => {
                use crate::hint::SequenceHint;

                let hint = SequenceHint::with_size(values.len());

                encoder.encode_sequence_fn(&hint, |sequence| {
                    for value in values.iter() {
                        sequence.encode_next()?.encode(value)?;
                    }

                    Ok(())
                })
            }
            Value::Map(values) => {
                use crate::hint::MapHint;

                let hint = MapHint::with_size(values.len());

                encoder.encode_map_fn(&hint, |map| {
                    for (first, second) in values.iter() {
                        map.insert_entry(first, second)?;
                    }

                    Ok(())
                })
            }
            Value::Variant(variant) => {
                let (tag, variant) = &**variant;
                let encoder = encoder.encode_variant()?;
                encoder.insert_variant(tag, variant)
            }
            Value::Option(option) => match option {
                Some(value) => encoder.encode_some()?.encode(&**value),
                None => encoder.encode_none(),
            },
        }
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

/// Value's [`AsDecoder`] implementation.
pub struct IntoValueDecoder<const OPT: Options, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    cx: C,
    value: Value<A>,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, C, A, M> IntoValueDecoder<OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    /// Construct a new buffered value decoder.
    #[inline]
    pub fn new(cx: C, value: Value<A>) -> Self {
        Self {
            cx,
            value,
            _marker: PhantomData,
        }
    }
}

impl<const OPT: Options, C, A, M> AsDecoder for IntoValueDecoder<OPT, C, A, M>
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
        Ok(self.value.decoder(self.cx))
    }
}

/// Value's [`AsDecoder`] implementation.
pub struct AsValueDecoder<'de, const OPT: Options, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    cx: C,
    value: &'de Value<A>,
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, C, A, M> AsValueDecoder<'de, OPT, C, A, M>
where
    C: Context,
    A: Allocator,
    M: 'static,
{
    /// Construct a new buffered value decoder.
    #[inline]
    pub fn new(cx: C, value: &'de Value<A>) -> Self {
        Self {
            cx,
            value,
            _marker: PhantomData,
        }
    }
}

impl<const OPT: Options, C, A, M> AsDecoder for AsValueDecoder<'_, OPT, C, A, M>
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
        Ok(self.value.decoder(self.cx))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl TryFrom<&str> for Value<System> {
    type Error = AllocError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut string = String::new_in(System::new());
        string.push_str(value)?;
        Ok(Value::String(string))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl From<rust_alloc::string::String> for Value<System> {
    #[inline]
    fn from(value: rust_alloc::string::String) -> Self {
        Value::String(String::from(value))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl From<rust_alloc::vec::Vec<Value<System>>> for Value<System> {
    #[inline]
    fn from(value: rust_alloc::vec::Vec<Value<System>>) -> Self {
        Value::Sequence(Vec::from(value))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl TryFrom<&[u8]> for Value<System> {
    type Error = AllocError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut string = Vec::new_in(System::new());
        string.extend_from_slice(value)?;
        Ok(Value::Bytes(string))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<const N: usize> TryFrom<&[u8; N]> for Value<System> {
    type Error = AllocError;

    #[inline]
    fn try_from(value: &[u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<const N: usize> TryFrom<[u8; N]> for Value<System> {
    type Error = AllocError;

    #[inline]
    fn try_from(value: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}
