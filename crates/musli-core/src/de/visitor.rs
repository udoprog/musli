use core::fmt;
use core::marker::PhantomData;

use crate::expecting::{self, Expecting};
use crate::Context;

use super::{Decoder, MapDecoder, SequenceDecoder, SizeHint, UnsizedVisitor, VariantDecoder};

/// Visitor capable of decoding any type into a value [`Visitor::Ok`].
///
/// Each callback on this visitor indicates the type that should be decoded from
/// the passed in decoder. A typical implementation would simply call the
/// corresponding decoder function for the type being visited.
pub trait Visitor<'de, C>
where
    Self: Sized,
    C: Context,
{
    /// The value produced by the visitor.
    type Ok;
    /// String decoder to use.
    type String: UnsizedVisitor<'de, C, str, Ok = Self::Ok>;
    /// Bytes decoder to use.
    type Bytes: UnsizedVisitor<'de, C, [u8], Ok = Self::Ok>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::visitor]`][musli::visitor] attribute
    /// macro when implementing [`Visitor`].
    #[doc(hidden)]
    type __UseMusliVisitorAttributeMacro;

    /// Format the human-readable message that should occur if the decoder was
    /// expecting to decode some specific kind of value.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Indicates that the visited type is empty.
    #[inline]
    fn visit_empty(self, cx: C) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Empty,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `bool`.
    #[inline]
    fn visit_bool(self, cx: C, _: bool) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Bool,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `char`.
    #[inline]
    fn visit_char(self, cx: C, _: char) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Char,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u8`.
    #[inline]
    fn visit_u8(self, cx: C, _: u8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u16`.
    #[inline]
    fn visit_u16(self, cx: C, _: u16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u32`.
    #[inline]
    fn visit_u32(self, cx: C, _: u32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u64`.
    #[inline]
    fn visit_u64(self, cx: C, _: u64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u128`.
    #[inline]
    fn visit_u128(self, cx: C, _: u128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i8`.
    #[inline]
    fn visit_i8(self, cx: C, _: i8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i16`.
    #[inline]
    fn visit_i16(self, cx: C, _: i16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i32`.
    #[inline]
    fn visit_i32(self, cx: C, _: i32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i64`.
    #[inline]
    fn visit_i64(self, cx: C, _: i64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i128`.
    #[inline]
    fn visit_i128(self, cx: C, _: i128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `usize`.
    #[inline]
    fn visit_usize(self, cx: C, _: usize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Usize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `isize`.
    #[inline]
    fn visit_isize(self, cx: C, _: isize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Isize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `f32`.
    #[inline]
    fn visit_f32(self, cx: C, _: f32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Float32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `f64`.
    #[inline]
    fn visit_f64(self, cx: C, _: f64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Float64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is an optional type.
    #[inline]
    fn visit_option<D>(self, cx: C, _: Option<D>) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, Cx = C, Error = C::Error, Mode = C::Mode>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Option,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a sequence.
    #[inline]
    fn visit_sequence<D>(self, decoder: &mut D) -> Result<Self::Ok, C::Error>
    where
        D: ?Sized + SequenceDecoder<'de, Cx = C>,
    {
        Err(decoder.cx().message(expecting::unsupported_type(
            &expecting::SequenceWith(decoder.size_hint()),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a map.
    #[inline]
    fn visit_map<D>(self, decoder: &mut D) -> Result<Self::Ok, <D::Cx as Context>::Error>
    where
        D: ?Sized + MapDecoder<'de, Cx = C>,
    {
        Err(decoder.cx().message(expecting::unsupported_type(
            &expecting::MapWith(decoder.size_hint()),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is `string`.
    #[inline]
    fn visit_string(self, cx: C, hint: SizeHint) -> Result<Self::String, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::StringWith(hint),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is `bytes`.
    #[inline]
    fn visit_bytes(self, cx: C, hint: SizeHint) -> Result<Self::Bytes, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::BytesWith(hint),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a variant.
    #[inline]
    fn visit_variant<D>(self, decoder: &mut D) -> Result<Self::Ok, C::Error>
    where
        D: VariantDecoder<'de, Cx = C>,
    {
        Err(decoder.cx().message(expecting::unsupported_type(
            &expecting::Variant,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the encoding does not support dynamic types.
    #[inline]
    fn visit_unknown<D>(self, decoder: D) -> Result<Self::Ok, D::Error>
    where
        D: Decoder<'de, Cx = C, Error = C::Error, Mode = C::Mode>,
    {
        Err(decoder.cx().message(expecting::unsupported_type(
            &expecting::Any,
            ExpectingWrapper::new(&self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T, C> {
    inner: T,
    _marker: PhantomData<C>,
}

impl<T, C> ExpectingWrapper<T, C> {
    fn new(inner: &T) -> &Self {
        // SAFETY: `ExpectingWrapper` is repr(transparent) over `T`.
        unsafe { &*(inner as *const T as *const Self) }
    }
}

impl<'de, T, C> Expecting for ExpectingWrapper<T, C>
where
    C: Context,
    T: Visitor<'de, C>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
