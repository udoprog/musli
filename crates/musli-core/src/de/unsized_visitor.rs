use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

use crate::alloc::ToOwned;
use crate::expecting::{self, Expecting};
use crate::Context;

/// A visitor for data where we might need to borrow without copying from the
/// underlying [`Decoder`].
///
/// A visitor is needed with [`Decoder::decode_bytes`] and
/// [`Decoder::decode_string`] because the caller doesn't know if the encoding
/// format is capable of producing references to the underlying data directly or
/// if it needs to be processed first.
///
/// If all you want is to decode a value by reference, use the
/// [`Decoder::decode_unsized`] method.
///
/// By requiring a visitor we ensure that the caller has to handle both
/// scenarios, even if one involves erroring. A type like [Cow] is an example of
/// a type which can comfortably handle both.
///
/// [Cow]: std::borrow::Cow
/// [`Decoder`]: crate::de::Decoder
/// [`Decoder::decode_bytes`]: crate::de::Decoder::decode_bytes
/// [`Decoder::decode_string`]: crate::de::Decoder::decode_string
/// [`Decoder::decode_unsized`]: crate::de::Decoder::decode_unsized
pub trait UnsizedVisitor<'de, C, T>
where
    Self: Sized,
    C: Context<Error = Self::Error>,
    T: ?Sized + ToOwned<C::Allocator>,
{
    /// The value produced by the visitor.
    type Ok;
    /// The error produced by the visitor.
    type Error;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit an owned value.
    #[inline(always)]
    fn visit_owned(self, cx: C, value: T::Owned) -> Result<Self::Ok, Self::Error> {
        self.visit_ref(cx, value.borrow())
    }

    /// Visit a string that is borrowed directly from the source data.
    #[inline(always)]
    fn visit_borrowed(self, cx: C, value: &'de T) -> Result<Self::Ok, Self::Error> {
        self.visit_ref(cx, value)
    }

    /// Visit a value reference that is provided from the decoder in any manner
    /// possible. Which might require additional decoding work.
    #[inline(always)]
    fn visit_ref(self, cx: C, _: &T) -> Result<Self::Ok, Self::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::AnyValue,
            ExpectingWrapper::new(&self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<'a, T, C, I>
where
    I: ?Sized,
{
    inner: T,
    _marker: PhantomData<(C, &'a I)>,
}

impl<T, C, U> ExpectingWrapper<'_, T, C, U>
where
    U: ?Sized,
{
    #[inline(always)]
    fn new(value: &T) -> &Self {
        // SAFETY: `ExpectingWrapper` is repr(transparent) over `T`.
        unsafe { &*(value as *const T as *const Self) }
    }
}

impl<'de, T, C, U> Expecting for ExpectingWrapper<'_, T, C, U>
where
    T: UnsizedVisitor<'de, C, U, Error = C::Error>,
    C: Context,
    U: ?Sized + ToOwned<C::Allocator>,
{
    #[inline(always)]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
