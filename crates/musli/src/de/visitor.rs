use core::fmt;

use crate::de::{
    Decoder, NumberHint, NumberVisitor, PairsDecoder, SequenceDecoder, SizeHint, TypeHint,
    VariantDecoder,
};
use crate::error::Error;
use crate::expecting::{self, Expecting};
use crate::Context;

use super::ValueVisitor;

/// Visitor capable of decoding any type into a value [`Visitor::Ok`].
///
/// Each callback on this visitor indicates the type that should be decoded from
/// the passed in decoder. A typical implementation would simply call the
/// corresponding decoder function for the type being visited.
pub trait Visitor<'de>: Sized {
    /// The value produced by the visitor.
    type Ok;
    /// The error type produced.
    type Error: Error;
    /// String decoder to use.
    type String<'buf, C>: ValueVisitor<'de, 'buf, C, str, Ok = Self::Ok>
    where
        C: Context<'buf, Input = Self::Error>;
    /// Bytes decoder to use.
    type Bytes<'buf, C>: ValueVisitor<'de, 'buf, C, [u8], Ok = Self::Ok>
    where
        C: Context<'buf, Input = Self::Error>;
    /// Number decoder to use.
    type Number<'buf, C>: NumberVisitor<'de, 'buf, C, Ok = Self::Ok>
    where
        C: Context<'buf, Input = Self::Error>;

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
    fn visit_any<'buf, C, D>(self, cx: &mut C, _: D, hint: TypeHint) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        D: Decoder<'de, Error = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(&hint, &ExpectingWrapper(self))))
    }

    /// Indicates that the visited type is a `unit`.
    #[inline]
    fn visit_unit<'buf, C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unit,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `bool`.
    #[inline]
    fn visit_bool<'buf, C>(self, cx: &mut C, _: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Bool,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `char`.
    #[inline]
    fn visit_char<'buf, C>(self, cx: &mut C, _: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Char,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u8`.
    #[inline]
    fn visit_u8<'buf, C>(self, cx: &mut C, _: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned8,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u16`.
    #[inline]
    fn visit_u16<'buf, C>(self, cx: &mut C, _: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned16,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u32`.
    #[inline]
    fn visit_u32<'buf, C>(self, cx: &mut C, _: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned32,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u64`.
    #[inline]
    fn visit_u64<'buf, C>(self, cx: &mut C, _: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned64,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u128`.
    #[inline]
    fn visit_u128<'buf, C>(self, cx: &mut C, _: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned128,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i8`.
    #[inline]
    fn visit_i8<'buf, C>(self, cx: &mut C, _: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed8,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i16`.
    #[inline]
    fn visit_i16<'buf, C>(self, cx: &mut C, _: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed16,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i32`.
    #[inline]
    fn visit_i32<'buf, C>(self, cx: &mut C, _: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed32,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i64`.
    #[inline]
    fn visit_i64<'buf, C>(self, cx: &mut C, _: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed64,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i128`.
    #[inline]
    fn visit_i128<'buf, C>(self, cx: &mut C, _: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed128,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `usize`.
    #[inline]
    fn visit_usize<'buf, C>(self, cx: &mut C, _: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Usize,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `isize`.
    #[inline]
    fn visit_isize<'buf, C>(self, cx: &mut C, _: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Isize,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `f32`.
    #[inline]
    fn visit_f32<'buf, C>(self, cx: &mut C, _: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Float32,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `f64`.
    #[inline]
    fn visit_f64<'buf, C>(self, cx: &mut C, _: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Float64,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is an optional type.
    #[inline]
    fn visit_option<'buf, C, D>(self, cx: &mut C, _: Option<D>) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        D: Decoder<'de, Error = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Option,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a sequence.
    #[inline]
    fn visit_sequence<'buf, C, D>(self, cx: &mut C, decoder: D) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        D: SequenceDecoder<'de, Error = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::SequenceWith(decoder.size_hint()),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a map.
    #[inline]
    fn visit_map<'buf, C, D>(self, cx: &mut C, decoder: D) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        D: PairsDecoder<'de, Error = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::MapWith(decoder.size_hint()),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is `string`.
    #[inline]
    fn visit_string<'buf, C>(
        self,
        cx: &mut C,
        hint: SizeHint,
    ) -> Result<Self::String<'buf, C>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::StringWith(hint),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is `bytes`.
    #[inline]
    fn visit_bytes<'buf, C>(
        self,
        cx: &mut C,
        hint: SizeHint,
    ) -> Result<Self::Bytes<'buf, C>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::BytesWith(hint),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a number.
    #[inline]
    fn visit_number<'buf, C>(
        self,
        cx: &mut C,
        hint: NumberHint,
    ) -> Result<Self::Number<'buf, C>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::NumberWith(hint),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a variant.
    #[inline]
    fn visit_variant<'buf, C, D>(self, cx: &mut C, _: D) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        D: VariantDecoder<'de, Error = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Variant,
            &ExpectingWrapper(self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T>(T);

impl<'de, T> Expecting for ExpectingWrapper<T>
where
    T: Visitor<'de>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
