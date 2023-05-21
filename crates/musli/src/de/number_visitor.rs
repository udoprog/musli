use core::fmt;
use core::marker;

use crate::de::{Decoder, TypeHint};
use crate::expecting::{self, Expecting};
use crate::Context;

/// A visitor capable of processing arbitrary number values.
pub trait NumberVisitor<'de, 'buf, C>: Sized
where
    C: Context<'buf>,
{
    /// The output of the visitor.
    type Ok;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit `u8`.
    #[inline]
    fn visit_u8(self, cx: &mut C, _: u8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `u16`.
    #[inline]
    fn visit_u16(self, cx: &mut C, _: u16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `u32`.
    #[inline]
    fn visit_u32(self, cx: &mut C, _: u32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `u64`.
    #[inline]
    fn visit_u64(self, cx: &mut C, _: u64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `u128`.
    #[inline]
    fn visit_u128(self, cx: &mut C, _: u128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `i8`.
    #[inline]
    fn visit_i8(self, cx: &mut C, _: i8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `i16`.
    #[inline]
    fn visit_i16(self, cx: &mut C, _: i16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `i32`.
    #[inline]
    fn visit_i32(self, cx: &mut C, _: i32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `i64`.
    #[inline]
    fn visit_i64(self, cx: &mut C, _: i64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `i128`.
    #[inline]
    fn visit_i128(self, cx: &mut C, _: i128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `f32`.
    #[inline]
    fn visit_f32(self, cx: &mut C, _: f32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Float32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `f64`.
    #[inline]
    fn visit_f64(self, cx: &mut C, _: f64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Float64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `usize`.
    #[inline]
    fn visit_usize(self, cx: &mut C, _: usize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Usize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit `isize`.
    #[inline]
    fn visit_isize(self, cx: &mut C, _: isize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Isize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Visit bytes constituting a raw number.
    #[inline]
    fn visit_bytes(self, cx: &mut C, _: &'de [u8]) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Number,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Fallback used when the type is either not implemented for this visitor
    /// or the underlying format doesn't know which type to decode.
    #[inline]
    fn visit_any<D>(self, cx: &mut C, _: D, hint: TypeHint) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, Error = C::Input>,
    {
        Err(cx.message(expecting::invalid_type(&hint, &ExpectingWrapper::new(self))))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T, C>(T, marker::PhantomData<C>);

impl<T, C> ExpectingWrapper<T, C> {
    #[inline]
    fn new(value: T) -> Self {
        Self(value, marker::PhantomData)
    }
}

impl<'de, 'buf, T, C> Expecting for ExpectingWrapper<T, C>
where
    C: Context<'buf>,
    T: NumberVisitor<'de, 'buf, C>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
