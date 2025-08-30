use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

use crate::alloc::ToOwned;
use crate::expecting::{self, Expecting};
use crate::{Allocator, Context};

/// A visitor for data where we might need to borrow without copying from the
/// underlying [`Decoder`].
///
/// When implementing this trait you must use the `#[musli::trait_defaults]`
/// attribute macro.
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
/// scenarios, even if one involves erroring. A type like [`Cow`] is an example
/// of a type which can comfortably handle both.
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::UnsizedVisitor;
///
/// struct Visitor;
///
/// #[musli::trait_defaults]
/// impl<'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor
/// where
///     C: Context,
/// {
///     type Ok = ();
///
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(
///             f,
///             "a reference of bytes"
///         )
///     }
/// }
/// ```
///
/// [`Cow`]: std::borrow::Cow
/// [`Decoder::decode_bytes`]: crate::de::Decoder::decode_bytes
/// [`Decoder::decode_string`]: crate::de::Decoder::decode_string
/// [`Decoder::decode_unsized`]: crate::de::Decoder::decode_unsized
/// [`Decoder`]: crate::de::Decoder
#[allow(unused_variables)]
pub trait UnsizedVisitor<'de, C, T>: Sized
where
    C: Context<Error = Self::Error, Allocator = Self::Allocator>,
    T: ?Sized + ToOwned,
{
    /// The value produced by the visitor.
    type Ok;
    /// The error produced by the visitor.
    type Error;
    /// The allocator associated with the visitor.
    type Allocator: Allocator;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the `#[musli::trait_defaults]`.
    #[doc(hidden)]
    type __UseMusliUnsizedVisitorAttributeMacro;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit an owned value.
    #[inline]
    fn visit_owned(self, cx: C, value: T::Owned<Self::Allocator>) -> Result<Self::Ok, Self::Error> {
        self.visit_ref(cx, value.borrow())
    }

    /// Visit a string that is borrowed directly from the source data.
    #[inline]
    fn visit_borrowed(self, cx: C, value: &'de T) -> Result<Self::Ok, Self::Error> {
        self.visit_ref(cx, value)
    }

    /// Visit a value reference that is provided from the decoder in any manner
    /// possible. Which might require additional decoding work.
    #[inline]
    fn visit_ref(self, cx: C, value: &T) -> Result<Self::Ok, Self::Error> {
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
    #[inline]
    fn new(value: &T) -> &Self {
        // SAFETY: `ExpectingWrapper` is repr(transparent) over `T`.
        unsafe { &*(value as *const T as *const Self) }
    }
}

impl<'de, T, C, U> Expecting for ExpectingWrapper<'_, T, C, U>
where
    T: UnsizedVisitor<'de, C, U, Error = C::Error, Allocator = C::Allocator>,
    C: Context,
    U: ?Sized + ToOwned,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
