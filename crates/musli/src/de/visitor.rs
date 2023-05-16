use core::fmt;

use crate::de::{
    Decoder, NumberHint, NumberVisitor, PairsDecoder, SequenceDecoder, SizeHint, TypeHint,
    VariantDecoder,
};
use crate::error::Error;
use crate::expecting::{self, Expecting};

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
    type String: ValueVisitor<'de, Target = str, Ok = Self::Ok, Error = Self::Error>;
    /// Bytes decoder to use.
    type Bytes: ValueVisitor<'de, Target = [u8], Ok = Self::Ok, Error = Self::Error>;
    /// Number decoder to use.
    type Number: NumberVisitor<'de, Ok = Self::Ok, Error = Self::Error>;

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
    fn visit_any<D>(self, _: D, hint: TypeHint) -> Result<Self::Ok, Self::Error>
    where
        D: Decoder<'de, Error = Self::Error>,
    {
        Err(Self::Error::message(expecting::invalid_type(
            &hint,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `unit`.
    #[inline]
    fn visit_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unit,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `bool`.
    #[inline]
    fn visit_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Bool,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `char`.
    #[inline]
    fn visit_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Char,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u8`.
    #[inline]
    fn visit_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned8,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u16`.
    #[inline]
    fn visit_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned16,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u32`.
    #[inline]
    fn visit_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned32,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u64`.
    #[inline]
    fn visit_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned64,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `u128`.
    #[inline]
    fn visit_u128(self, _: u128) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned128,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i8`.
    #[inline]
    fn visit_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed8,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i16`.
    #[inline]
    fn visit_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed16,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i32`.
    #[inline]
    fn visit_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed32,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i64`.
    #[inline]
    fn visit_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed64,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `i128`.
    #[inline]
    fn visit_i128(self, _: i128) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed128,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `usize`.
    #[inline]
    fn visit_usize(self, _: usize) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Usize,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `isize`.
    #[inline]
    fn visit_isize(self, _: isize) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Isize,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `f32`.
    #[inline]
    fn visit_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Float32,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a `f64`.
    #[inline]
    fn visit_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Float64,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is an optional type.
    #[inline]
    fn visit_option<D>(self, _: Option<D>) -> Result<Self::Ok, Self::Error>
    where
        D: Decoder<'de, Error = Self::Error>,
    {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Option,
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a sequence.
    #[inline]
    fn visit_sequence<D>(self, decoder: D) -> Result<Self::Ok, Self::Error>
    where
        D: SequenceDecoder<'de, Error = Self::Error>,
    {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::SequenceWith(decoder.size_hint()),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a map.
    #[inline]
    fn visit_map<D>(self, decoder: D) -> Result<Self::Ok, Self::Error>
    where
        D: PairsDecoder<'de, Error = Self::Error>,
    {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::MapWith(decoder.size_hint()),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is `string`.
    #[inline]
    fn visit_string(self, hint: SizeHint) -> Result<Self::String, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::StringWith(hint),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is `bytes`.
    #[inline]
    fn visit_bytes(self, hint: SizeHint) -> Result<Self::Bytes, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::BytesWith(hint),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a number.
    #[inline]
    fn visit_number(self, hint: NumberHint) -> Result<Self::Number, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::NumberWith(hint),
            &ExpectingWrapper(self),
        )))
    }

    /// Indicates that the visited type is a variant.
    #[inline]
    fn visit_variant<D>(self, _: D) -> Result<Self::Ok, Self::Error>
    where
        D: VariantDecoder<'de, Error = Self::Error>,
    {
        Err(Self::Error::message(expecting::invalid_type(
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
