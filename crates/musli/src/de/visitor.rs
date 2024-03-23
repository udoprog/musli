use core::fmt;
use core::marker::PhantomData;

use crate::de::{
    Decoder, MapDecoder, NumberHint, NumberVisitor, SequenceDecoder, SizeHint, TypeHint,
    VariantDecoder,
};
use crate::expecting::{self, Expecting};
use crate::Context;

use super::ValueVisitor;

/// Visitor capable of decoding any type into a value [`Visitor::Ok`].
///
/// Each callback on this visitor indicates the type that should be decoded from
/// the passed in decoder. A typical implementation would simply call the
/// corresponding decoder function for the type being visited.
pub trait Visitor<'de, C: ?Sized + Context>: Sized {
    /// The value produced by the visitor.
    type Ok;
    /// String decoder to use.
    type String: ValueVisitor<'de, C, str, Ok = Self::Ok>;
    /// Bytes decoder to use.
    type Bytes: ValueVisitor<'de, C, [u8], Ok = Self::Ok>;
    /// Number decoder to use.
    type Number: NumberVisitor<'de, C, Ok = Self::Ok>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::visitor]`][crate::visitor] attribute
    /// macro when implementing [`Visitor`].
    #[doc(hidden)]
    type __UseMusliVisitorAttributeMacro;

    /// Format the human-readable message that should occur if the decoder was
    /// expecting to decode some specific kind of value.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Fallback used when the type is either not implemented for this visitor
    /// or the underlying format doesn't know which type to decode.
    fn visit_any<D>(self, cx: &C, _: D, hint: TypeHint) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, Cx = C>,
    {
        Err(cx.message(expecting::unsupported_type(
            &hint,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `unit`.
    #[inline]
    fn visit_unit(self, cx: &C) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unit,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `bool`.
    #[inline]
    fn visit_bool(self, cx: &C, _: bool) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Bool,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `char`.
    #[inline]
    fn visit_char(self, cx: &C, _: char) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Char,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u8`.
    #[inline]
    fn visit_u8(self, cx: &C, _: u8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u16`.
    #[inline]
    fn visit_u16(self, cx: &C, _: u16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u32`.
    #[inline]
    fn visit_u32(self, cx: &C, _: u32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u64`.
    #[inline]
    fn visit_u64(self, cx: &C, _: u64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `u128`.
    #[inline]
    fn visit_u128(self, cx: &C, _: u128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i8`.
    #[inline]
    fn visit_i8(self, cx: &C, _: i8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i16`.
    #[inline]
    fn visit_i16(self, cx: &C, _: i16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i32`.
    #[inline]
    fn visit_i32(self, cx: &C, _: i32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i64`.
    #[inline]
    fn visit_i64(self, cx: &C, _: i64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `i128`.
    #[inline]
    fn visit_i128(self, cx: &C, _: i128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `usize`.
    #[inline]
    fn visit_usize(self, cx: &C, _: usize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Usize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `isize`.
    #[inline]
    fn visit_isize(self, cx: &C, _: isize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Isize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `f32`.
    #[inline]
    fn visit_f32(self, cx: &C, _: f32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Float32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a `f64`.
    #[inline]
    fn visit_f64(self, cx: &C, _: f64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Float64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is an optional type.
    #[inline]
    fn visit_option<D>(self, cx: &C, _: Option<D>) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, Cx = C>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Option,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a sequence.
    #[inline]
    fn visit_sequence<D>(self, cx: &C, decoder: D) -> Result<Self::Ok, C::Error>
    where
        D: SequenceDecoder<'de, Cx = C>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::SequenceWith(decoder.size_hint()),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a map.
    #[inline]
    fn visit_map<D>(self, cx: &C, decoder: D) -> Result<Self::Ok, <D::Cx as Context>::Error>
    where
        D: MapDecoder<'de, Cx = C>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::MapWith(decoder.size_hint()),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is `string`.
    #[inline]
    fn visit_string(self, cx: &C, hint: SizeHint) -> Result<Self::String, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::StringWith(hint),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is `bytes`.
    #[inline]
    fn visit_bytes(self, cx: &C, hint: SizeHint) -> Result<Self::Bytes, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::BytesWith(hint),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a number.
    #[inline]
    fn visit_number(self, cx: &C, hint: NumberHint) -> Result<Self::Number, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::NumberWith(hint),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Indicates that the visited type is a variant.
    #[inline]
    fn visit_variant<D>(self, cx: &C, _: D) -> Result<Self::Ok, C::Error>
    where
        D: VariantDecoder<'de, Cx = C>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Variant,
            ExpectingWrapper::new(&self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<'a, T, C: ?Sized> {
    inner: T,
    _marker: PhantomData<&'a C>,
}

impl<'a, T, C: ?Sized> ExpectingWrapper<'a, T, C> {
    fn new(inner: &T) -> &Self {
        // SAFETY: `ExpectingWrapper` is repr(transparent) over `T`.
        unsafe { &*(inner as *const T as *const Self) }
    }
}

impl<'a, 'de, T, C> Expecting for ExpectingWrapper<'a, T, C>
where
    C: ?Sized + Context,
    T: Visitor<'de, C>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
