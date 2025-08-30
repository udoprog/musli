use core::cmp::Ordering;
use core::fmt::{self, Write};
use core::marker::PhantomData;
use core::str;

#[cfg(feature = "alloc")]
use crate::alloc::GlobalAllocator;
use crate::alloc::{AllocError, Box, String, Vec};
use crate::de::{AsDecoder, Decode, Decoder, Visitor};
use crate::de::{
    EntryDecoder, MapDecoder, SequenceDecoder, SizeHint, UnsizedVisitor, VariantDecoder,
};
use crate::en::{Encode, Encoder};
use crate::en::{MapEncoder, SequenceEncoder, VariantEncoder};
use crate::{Allocator, Context, Options};

use super::de::ValueDecoder;
use super::type_hint::{NumberHint, TypeHint};

/// The kind of a value.
pub(crate) enum ValueKind<A>
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

/// This is a type-erased value which can be deserialized from any [Müsli]
/// supported type.
///
/// It's primarily used to store or cache values for more complex processing,
/// but can also be used to store any value in-memory.
///
/// [Müsli]: https://github.com/udoprog/musli
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::value::{self, Value};
///
/// #[derive(Decode, Encode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let value = value::encode(person)?;
/// let person: Person = value::decode(&value)?;
///
/// assert_eq!(person.name, "Alice");
/// assert_eq!(person.age, 30);
/// # Ok::<_, value::Error>(())
/// ```
///
/// Building a value directly:
///
/// ```
/// use musli::alloc::Global;
/// use musli::value::Value;
///
/// let alloc = Global::new();
///
/// let value: Value<Global> = Value::new_map_in([(Value::from(1u32), Value::from(2u32))], alloc)?;
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
pub struct Value<A>
where
    A: Allocator,
{
    pub(super) kind: ValueKind<A>,
}

impl<A> Value<A>
where
    A: Allocator,
{
    /// Construct a new value around a kind.
    #[inline]
    pub(crate) const fn new(kind: ValueKind<A>) -> Self {
        Self { kind }
    }

    /// Construct a map out of entries in the given iterator.
    ///
    /// Note that "maps" in musli can contain duplicate keys, and unless two
    /// maps are constructed from exactly the same input data they are not
    /// guaranteed to be equal.
    ///
    /// ```
    /// use musli::alloc::Global;
    /// use musli::value::Value;
    ///
    /// let alloc = Global::new();
    ///
    /// let a: Value<Global> = Value::new_map_in([
    ///     (Value::from(1u32), Value::from(2u32)),
    ///     (Value::from(1u32), Value::from(2u32))
    /// ], alloc)?;
    ///
    /// let b: Value<Global> = Value::new_map_in([
    ///     (Value::from(2u32), Value::from(1u32)),
    ///     (Value::from(2u32), Value::from(1u32))
    /// ], alloc)?;
    ///
    /// assert_ne!(a, b);
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::Global;
    /// use musli::value::Value;
    ///
    /// let alloc = Global::new();
    ///
    /// let value: Value<Global> = Value::new_map_in([(Value::from(1u32), Value::from(2u32))], alloc)?;
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    pub fn new_map_in(
        iter: impl IntoIterator<Item = (Value<A>, Value<A>)>,
        alloc: A,
    ) -> Result<Self, AllocError> {
        let mut entries = Vec::new_in(alloc);

        let iter = iter.into_iter();
        let (low, _) = iter.size_hint();
        entries.reserve(low)?;

        for (key, value) in iter {
            entries.push((key, value))?;
        }

        Ok(Self::new(ValueKind::Map(entries)))
    }

    /// Construct a [`IntoValueDecoder`] implementation out of the current
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{context, value};
    /// use musli::mode::Binary;
    ///
    /// let cx = context::new();
    /// let value = value::encode(42u32)?;
    /// let decoder = value.into_decoder::<0, _, Binary>(&cx);
    /// # Ok::<_, value::Error>(())
    /// ```
    #[inline]
    pub fn into_decoder<const OPT: Options, C, M>(self, cx: C) -> IntoValueDecoder<OPT, C, A, M>
    where
        C: Context,
    {
        IntoValueDecoder::new(cx, self)
    }

    /// Construct a [`AsValueDecoder`] implementation out of the current value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{context, value};
    /// use musli::mode::Binary;
    ///
    /// let cx = context::new();
    /// let value = value::encode(42u32)?;
    /// let decoder = value.as_decoder::<0, _, Binary>(&cx);
    /// # Ok::<_, value::Error>(())
    /// ```
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
        match &self.kind {
            ValueKind::Unit => TypeHint::Unit,
            ValueKind::Bool(..) => TypeHint::Bool,
            ValueKind::Char(..) => TypeHint::Char,
            ValueKind::Number(number) => TypeHint::Number(number.type_hint()),
            ValueKind::Bytes(bytes) => TypeHint::Bytes(SizeHint::exact(bytes.len())),
            ValueKind::String(string) => TypeHint::String(SizeHint::exact(string.len())),
            ValueKind::Sequence(sequence) => TypeHint::Sequence(SizeHint::exact(sequence.len())),
            ValueKind::Map(map) => TypeHint::Map(SizeHint::exact(map.len())),
            ValueKind::Variant(..) => TypeHint::Variant,
            ValueKind::Option(..) => TypeHint::Option,
        }
    }
}

/// Debug implementation for a value.
///
/// # Example
///
/// ```
/// use musli::alloc::Global;
/// use musli::value::Value;
///
/// let value: Value<Global> = Value::try_from(b"hello world")?;
/// assert_eq!(format!("{value:?}"), "\"hello world\"");
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
impl<A> fmt::Debug for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValueKind::Unit => write!(f, "()"),
            ValueKind::Bool(value) => value.fmt(f),
            ValueKind::Char(value) => value.fmt(f),
            ValueKind::Number(value) => value.fmt(f),
            ValueKind::Bytes(value) => BStr::new(value).fmt(f),
            ValueKind::String(value) => value.fmt(f),
            ValueKind::Sequence(value) => value.fmt(f),
            ValueKind::Map(items) => {
                let mut map = f.debug_map();

                for (key, value) in items.iter() {
                    map.entry(key, value);
                }

                map.finish()
            }
            ValueKind::Variant(value) => {
                let (variant, data) = &**value;
                f.debug_struct("Variant")
                    .field("variant", variant)
                    .field("data", data)
                    .finish()
            }
            ValueKind::Option(value) => match value {
                Some(v) => f.debug_tuple("Some").field(v).finish(),
                None => f.debug_tuple("None").finish(),
            },
        }
    }
}

impl<A> PartialEq for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (ValueKind::Unit, ValueKind::Unit) => true,
            (ValueKind::Bool(lhs), ValueKind::Bool(rhs)) => lhs == rhs,
            (ValueKind::Char(lhs), ValueKind::Char(rhs)) => lhs == rhs,
            (ValueKind::Number(lhs), ValueKind::Number(rhs)) => lhs == rhs,
            (ValueKind::Bytes(lhs), ValueKind::Bytes(rhs)) => lhs == rhs,
            (ValueKind::String(lhs), ValueKind::String(rhs)) => lhs == rhs,
            (ValueKind::Sequence(lhs), ValueKind::Sequence(rhs)) => lhs == rhs,
            (ValueKind::Map(lhs), ValueKind::Map(rhs)) => lhs == rhs,
            (ValueKind::Variant(lhs), ValueKind::Variant(rhs)) => lhs == rhs,
            (ValueKind::Option(lhs), ValueKind::Option(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl<A> PartialOrd for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.kind, &other.kind) {
            (ValueKind::Unit, ValueKind::Unit) => Some(Ordering::Equal),
            (ValueKind::Bool(lhs), ValueKind::Bool(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::Char(lhs), ValueKind::Char(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::Number(lhs), ValueKind::Number(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::Bytes(lhs), ValueKind::Bytes(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::String(lhs), ValueKind::String(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::Sequence(lhs), ValueKind::Sequence(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::Map(lhs), ValueKind::Map(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::Variant(lhs), ValueKind::Variant(rhs)) => lhs.partial_cmp(rhs),
            (ValueKind::Option(lhs), ValueKind::Option(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}

/// A dynamic number value.
///
/// This can represent any of the primitive number types in Rust.
/// Used internally by the Value enum to store numeric data.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[non_exhaustive]
pub(crate) enum Number {
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

    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
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

#[crate::visitor(crate)]
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
        Ok(Value::new(ValueKind::Unit))
    }

    #[inline]
    fn visit_bool(self, _: C, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Bool(value)))
    }

    #[inline]
    fn visit_char(self, _: C, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Char(value)))
    }

    #[inline]
    fn visit_u8(self, _: C, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::U8(value))))
    }

    #[inline]
    fn visit_u16(self, _: C, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::U16(value))))
    }

    #[inline]
    fn visit_u32(self, _: C, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::U32(value))))
    }

    #[inline]
    fn visit_u64(self, _: C, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::U64(value))))
    }

    #[inline]
    fn visit_u128(self, _: C, value: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::U128(value))))
    }

    #[inline]
    fn visit_i8(self, _: C, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::I8(value))))
    }

    #[inline]
    fn visit_i16(self, _: C, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::I16(value))))
    }

    #[inline]
    fn visit_i32(self, _: C, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::I32(value))))
    }

    #[inline]
    fn visit_i64(self, _: C, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::I64(value))))
    }

    #[inline]
    fn visit_i128(self, _: C, value: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::I128(value))))
    }

    #[inline]
    fn visit_usize(self, _: C, value: usize) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::Usize(value))))
    }

    #[inline]
    fn visit_isize(self, _: C, value: isize) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::Isize(value))))
    }

    #[inline]
    fn visit_f32(self, _: C, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::F32(value))))
    }

    #[inline]
    fn visit_f64(self, _: C, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Number(Number::F64(value))))
    }

    #[inline]
    fn visit_none(self, _: C) -> Result<Self::Ok, Self::Error> {
        Ok(Value::new(ValueKind::Option(None)))
    }

    #[inline]
    fn visit_some<D>(self, decoder: D) -> Result<Self::Ok, Self::Error>
    where
        D: Decoder<'de, Cx = C, Error = C::Error, Allocator = C::Allocator>,
    {
        let cx = decoder.cx();
        let value = decoder.decode::<Value<C::Allocator>>()?;
        let value = Box::new_in(value, cx.alloc()).map_err(cx.map())?;
        Ok(Value::new(ValueKind::Option(Some(value))))
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

        Ok(Value::new(ValueKind::Sequence(out)))
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

        Ok(Value::new(ValueKind::Map(out)))
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
        Ok(Value::new(ValueKind::Variant(value)))
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

#[crate::unsized_visitor(crate)]
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
        Ok(Value::new(ValueKind::Bytes(bytes)))
    }
}

struct StringVisitor;

#[crate::unsized_visitor(crate)]
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
        Ok(Value::new(ValueKind::String(string)))
    }
}

impl<M, C> Encode<M> for Value<C>
where
    C: Allocator,
{
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        match &self.kind {
            ValueKind::Unit => encoder.encode_empty(),
            ValueKind::Bool(b) => encoder.encode_bool(*b),
            ValueKind::Char(c) => encoder.encode_char(*c),
            ValueKind::Number(n) => encoder.encode(n),
            ValueKind::Bytes(bytes) => encoder.encode_bytes(bytes),
            ValueKind::String(string) => encoder.encode_string(string),
            ValueKind::Sequence(values) => encoder.encode_sequence_fn(values.len(), |sequence| {
                for value in values.iter() {
                    sequence.encode_next()?.encode(value)?;
                }

                Ok(())
            }),
            ValueKind::Map(values) => {
                let hint = values.len();

                encoder.encode_map_fn(hint, |map| {
                    for (first, second) in values.iter() {
                        map.insert_entry(first, second)?;
                    }

                    Ok(())
                })
            }
            ValueKind::Variant(variant) => {
                let (tag, variant) = &**variant;
                let encoder = encoder.encode_variant()?;
                encoder.insert_variant(tag, variant)
            }
            ValueKind::Option(option) => match option {
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
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Allocator, Context};
    /// use musli::options::{self, Options};
    /// use musli::value::{Value, IntoValueDecoder};
    /// use musli::alloc::Global;
    ///
    /// const OPTIONS: Options = options::new().build();
    ///
    /// let alloc = Global::new();
    /// let cx = musli::context::new();
    /// let value: Value<Global> = Value::from(());
    /// let decoder = IntoValueDecoder::<OPTIONS, _, _, ()>::new(&cx, value);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Allocator, Context};
    /// use musli::options::{self, Options};
    /// use musli::value::{Value, AsValueDecoder};
    /// use musli::alloc::Global;
    ///
    /// const OPTIONS: Options = options::new().build();
    ///
    /// let alloc = Global::new();
    /// let cx = musli::context::new();
    /// let value: Value<Global> = Value::from(());
    /// let decoder = AsValueDecoder::<OPTIONS, _, _, ()>::new(&cx, &value);
    /// ```
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
impl<A> TryFrom<&str> for Value<A>
where
    A: GlobalAllocator,
{
    type Error = AllocError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut string = String::new_in(A::new());
        string.push_str(value)?;
        Ok(Value::new(ValueKind::String(string)))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<A> From<rust_alloc::string::String> for Value<A>
where
    A: GlobalAllocator,
{
    #[inline]
    fn from(value: rust_alloc::string::String) -> Self {
        Value::new(ValueKind::String(String::from(value)))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<A> From<rust_alloc::vec::Vec<Value<A>>> for Value<A>
where
    A: GlobalAllocator,
{
    #[inline]
    fn from(value: rust_alloc::vec::Vec<Value<A>>) -> Self {
        Value::new(ValueKind::Sequence(Vec::from(value)))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<A> TryFrom<&[u8]> for Value<A>
where
    A: GlobalAllocator,
{
    type Error = AllocError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut string = Vec::new_in(A::new());
        string.extend_from_slice(value)?;
        Ok(Value::new(ValueKind::Bytes(string)))
    }
}

/// Construct a value from a reference to a byte array using a global allocator.
///
/// # Examples
///
/// ```
/// use musli::alloc::Global;
/// use musli::value::Value;
///
/// let value: Value<Global> = Value::try_from(&[1, 2, 3, 4])?;
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<const N: usize, A> TryFrom<&[u8; N]> for Value<A>
where
    A: GlobalAllocator,
{
    type Error = AllocError;

    #[inline]
    fn try_from(value: &[u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

/// Construct a value from a byte array using a global allocator.
///
/// # Examples
///
/// ```
/// use musli::alloc::Global;
/// use musli::value::Value;
///
/// let value: Value<Global> = Value::try_from([1, 2, 3, 4])?;
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<const N: usize, A> TryFrom<[u8; N]> for Value<A>
where
    A: GlobalAllocator,
{
    type Error = AllocError;

    #[inline]
    fn try_from(value: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

/// Construct a value from a unit type.
///
/// # Examples
///
/// ```
/// use musli::value::Value;
/// use musli::alloc::Global;
///
/// let unit: Value<Global> = Value::from(());
/// let number: Value<Global> = Value::from(42u32);
///
/// assert_eq!(unit, unit);
/// assert_ne!(unit, number);
/// ```
impl<A> From<()> for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn from((): ()) -> Self {
        Value::new(ValueKind::Unit)
    }
}

macro_rules! number_from {
    ($($ty:ty => $variant:ident, $example:expr, $min:expr, $max:expr),* $(,)?) => {
        $(
            /// Convert from a primitive number.
            ///
            /// # Examples
            ///
            /// ```
            /// use musli::value::Value;
            /// use musli::alloc::Global;
            ///
            #[doc = concat!("let value: ", stringify!($ty), " = ", stringify!($example), ";")]
            /// let value: Value<Global> = Value::from(value);
            ///
            #[doc = concat!("let value: ", stringify!($ty), " = ", stringify!($min), ";")]
            /// let min: Value<Global> = Value::from(value);
            ///
            #[doc = concat!("let value: ", stringify!($ty), " = ", stringify!($max), ";")]
            /// let max: Value<Global> = Value::from(value);
            ///
            /// assert_eq!(value, value);
            /// assert_ne!(min, max);
            /// ```
            impl<A> From<$ty> for Value<A>
            where
                A: Allocator,
            {
                #[inline]
                fn from(value: $ty) -> Self {
                    Value::new(ValueKind::Number(Number::$variant(value)))
                }
            }
        )*
    };
}

number_from! {
    i8 => I8, 42, i8::MAX, i8::MIN,
    i16 => I16, 42, i16::MAX, i16::MIN,
    i32 => I32, 42, i32::MAX, i32::MIN,
    i64 => I64, 42, i64::MAX, i64::MIN,
    i128 => I128, 42, i128::MAX, i128::MIN,
    isize => Isize, 42, isize::MAX, isize::MIN,
    u8 => U8, 42, u8::MAX, u8::MIN,
    u16 => U16, 42, u16::MAX, u16::MIN,
    u32 => U32, 42, u32::MAX, u32::MIN,
    u64 => U64, 42, u64::MAX, u64::MIN,
    u128 => U128, 42, u128::MAX, u128::MIN,
    usize => Usize, 42, usize::MAX, usize::MIN,
    f32 => F32, 42.42, 0.42, 100000.42,
    f64 => F64, 42.42, 0.42, 100000.42,
}

#[repr(transparent)]
struct BStr([u8]);

impl BStr {
    fn new(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes as *const [u8] as *const Self) }
    }
}

impl fmt::Debug for BStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;

        for chunk in self.0.utf8_chunks() {
            f.write_str(chunk.valid())?;

            for b in chunk.invalid() {
                write!(f, "\\x{:02x}", b)?;
            }
        }

        f.write_char('"')?;
        Ok(())
    }
}
