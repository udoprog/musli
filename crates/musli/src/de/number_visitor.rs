use core::fmt;

use crate::de::{Decoder, TypeHint};
use crate::error::Error;
use crate::expecting::{self, Expecting};
use crate::Context;

/// A visitor capable of processing arbitrary number values.
pub trait NumberVisitor<'de>: Sized {
    /// The output of the visitor.
    type Ok;
    /// An error type.
    type Error: Error;
    /// The context associated with the value visitor.
    type Context: Context<Input = Self::Error>;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit `u8`.
    #[inline]
    fn visit_u8(
        self,
        cx: &mut Self::Context,
        _: u8,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned8,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `u16`.
    #[inline]
    fn visit_u16(
        self,
        cx: &mut Self::Context,
        _: u16,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned16,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `u32`.
    #[inline]
    fn visit_u32(
        self,
        cx: &mut Self::Context,
        _: u32,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned32,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `u64`.
    #[inline]
    fn visit_u64(
        self,
        cx: &mut Self::Context,
        _: u64,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned64,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `u128`.
    #[inline]
    fn visit_u128(
        self,
        cx: &mut Self::Context,
        _: u128,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned128,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `i8`.
    #[inline]
    fn visit_i8(
        self,
        cx: &mut Self::Context,
        _: i8,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed8,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `i16`.
    #[inline]
    fn visit_i16(
        self,
        cx: &mut Self::Context,
        _: i16,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed16,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `i32`.
    #[inline]
    fn visit_i32(
        self,
        cx: &mut Self::Context,
        _: i32,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed32,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `i64`.
    #[inline]
    fn visit_i64(
        self,
        cx: &mut Self::Context,
        _: i64,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed64,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `i128`.
    #[inline]
    fn visit_i128(
        self,
        cx: &mut Self::Context,
        _: i128,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed128,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `f32`.
    #[inline]
    fn visit_f32(
        self,
        cx: &mut Self::Context,
        _: f32,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Float32,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `f64`.
    #[inline]
    fn visit_f64(
        self,
        cx: &mut Self::Context,
        _: f64,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Float64,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `usize`.
    #[inline]
    fn visit_usize(
        self,
        cx: &mut Self::Context,
        _: usize,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Usize,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit `isize`.
    #[inline]
    fn visit_isize(
        self,
        cx: &mut Self::Context,
        _: isize,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Isize,
            &ExpectingWrapper(self),
        )))
    }

    /// Visit bytes constituting a raw number.
    #[inline]
    fn visit_bytes(
        self,
        cx: &mut Self::Context,
        _: &'de [u8],
    ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Number,
            &ExpectingWrapper(self),
        )))
    }

    /// Fallback used when the type is either not implemented for this visitor
    /// or the underlying format doesn't know which type to decode.
    #[inline]
    fn visit_any<D>(
        self,
        cx: &mut Self::Context,
        _: D,
        hint: TypeHint,
    ) -> Result<Self::Ok, <Self::Context as Context>::Error>
    where
        D: Decoder<'de, Error = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(&hint, &ExpectingWrapper(self))))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T>(T);

impl<'de, T> Expecting for ExpectingWrapper<T>
where
    T: NumberVisitor<'de>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
