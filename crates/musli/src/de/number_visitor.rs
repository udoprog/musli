use core::fmt;
use core::marker::PhantomData;

use crate::de::{Decoder, TypeHint};
use crate::expecting::{self, Expecting};
use crate::Context;

/// A visitor capable of processing arbitrary number values.
pub trait NumberVisitor<'de, C: ?Sized + Context>: Sized {
    /// The output of the visitor.
    type Ok;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit `u8`.
    #[inline]
    fn visit_u8(self, cx: &C, _: u8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `u16`.
    #[inline]
    fn visit_u16(self, cx: &C, _: u16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `u32`.
    #[inline]
    fn visit_u32(self, cx: &C, _: u32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `u64`.
    #[inline]
    fn visit_u64(self, cx: &C, _: u64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `u128`.
    #[inline]
    fn visit_u128(self, cx: &C, _: u128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Unsigned128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `i8`.
    #[inline]
    fn visit_i8(self, cx: &C, _: i8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `i16`.
    #[inline]
    fn visit_i16(self, cx: &C, _: i16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `i32`.
    #[inline]
    fn visit_i32(self, cx: &C, _: i32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `i64`.
    #[inline]
    fn visit_i64(self, cx: &C, _: i64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `i128`.
    #[inline]
    fn visit_i128(self, cx: &C, _: i128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Signed128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `f32`.
    #[inline]
    fn visit_f32(self, cx: &C, _: f32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Float32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `f64`.
    #[inline]
    fn visit_f64(self, cx: &C, _: f64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Float64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `usize`.
    #[inline]
    fn visit_usize(self, cx: &C, _: usize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Usize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit `isize`.
    #[inline]
    fn visit_isize(self, cx: &C, _: isize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Isize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Visit bytes constituting a raw number.
    #[inline]
    fn visit_bytes(self, cx: &C, _: &'de [u8]) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::Number,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Fallback used when the type is either not implemented for this visitor
    /// or the underlying format doesn't know which type to decode.
    #[inline]
    fn visit_any<D>(self, cx: &C, _: D, hint: TypeHint) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, C>,
    {
        Err(cx.message(expecting::unsupported_type(
            &hint,
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
    #[inline]
    fn new(value: &T) -> &Self {
        // SAFETY: `ExpectingWrapper` is repr(transparent) over `T`.
        unsafe { &*(value as *const T as *const Self) }
    }
}

impl<'a, 'de, T, C> Expecting for ExpectingWrapper<'a, T, C>
where
    C: ?Sized + Context,
    T: NumberVisitor<'de, C>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
