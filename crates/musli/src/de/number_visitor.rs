use core::fmt;

use crate::error::Error;
use crate::expecting::{self, Expecting};

/// A visitor capable of processing arbitrary number values.
pub trait NumberVisitor: Sized {
    /// The output of the visitor.
    type Ok;
    /// An error type.
    type Error: Error;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit `u8`.
    #[inline]
    fn visit_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Unsigned8,
            &NumberExpecting(self),
        )))
    }

    /// Visit `u16`.
    #[inline]
    fn visit_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Unsigned16,
            &NumberExpecting(self),
        )))
    }

    /// Visit `u32`.
    #[inline]
    fn visit_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Unsigned32,
            &NumberExpecting(self),
        )))
    }

    /// Visit `u64`.
    #[inline]
    fn visit_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Unsigned64,
            &NumberExpecting(self),
        )))
    }

    /// Visit `u128`.
    #[inline]
    fn visit_u128(self, _: u128) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Unsigned128,
            &NumberExpecting(self),
        )))
    }

    /// Visit `i8`.
    #[inline]
    fn visit_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Signed8,
            &NumberExpecting(self),
        )))
    }

    /// Visit `i16`.
    #[inline]
    fn visit_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Signed16,
            &NumberExpecting(self),
        )))
    }

    /// Visit `i32`.
    #[inline]
    fn visit_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Signed32,
            &NumberExpecting(self),
        )))
    }

    /// Visit `i64`.
    #[inline]
    fn visit_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Signed64,
            &NumberExpecting(self),
        )))
    }

    /// Visit `i128`.
    #[inline]
    fn visit_i128(self, _: i128) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Signed128,
            &NumberExpecting(self),
        )))
    }

    /// Visit `f32`.
    #[inline]
    fn visit_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Float32,
            &NumberExpecting(self),
        )))
    }

    /// Visit `f64`.
    #[inline]
    fn visit_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Float64,
            &NumberExpecting(self),
        )))
    }

    /// Visit `usize`.
    #[inline]
    fn visit_usize(self, _: usize) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Usize,
            &NumberExpecting(self),
        )))
    }

    /// Visit `isize`.
    #[inline]
    fn visit_isize(self, _: isize) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::Isize,
            &NumberExpecting(self),
        )))
    }
}

#[repr(transparent)]
struct NumberExpecting<T>(T);

impl<T> Expecting for NumberExpecting<T>
where
    T: NumberVisitor,
{
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
