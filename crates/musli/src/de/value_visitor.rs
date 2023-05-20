use core::borrow::Borrow;
use core::fmt;
use core::marker;

use crate::de::{Decoder, TypeHint};
use crate::expecting::{self, Expecting};
use crate::no_std::ToOwned;
use crate::Context;

/// A visitor for data where it might be possible to borrow it without copying
/// from the underlying [Decoder].
///
/// A visitor is required with [Decoder::decode_bytes] and
/// [Decoder::decode_string] because the caller doesn't know if the encoding
/// format is capable of producing references to the underlying data directly or
/// if it needs to be processed.
///
/// By requiring a visitor we ensure that the caller has to handle both
/// scenarios, even if one involves erroring. A type like
/// [Cow][std::borrow::Cow] is an example of a type which can comfortably handle
/// both.
///
/// [Decoder]: crate::de::Decoder
/// [Decoder::decode_bytes]: crate::de::Decoder::decode_bytes
/// [Decoder::decode_string]: crate::de::Decoder::decode_string
pub trait ValueVisitor<'de, C, T>: Sized
where
    C: Context,
    T: ?Sized + ToOwned,
{
    /// The value produced.
    type Ok;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit an owned value.
    #[inline]
    fn visit_owned(self, cx: &mut C, value: T::Owned) -> Result<Self::Ok, C::Error> {
        self.visit_ref(cx, value.borrow())
    }

    /// Visit a string that is borrowed directly from the source data.
    #[inline]
    fn visit_borrowed(self, cx: &mut C, value: &'de T) -> Result<Self::Ok, C::Error> {
        self.visit_ref(cx, value)
    }

    /// Visit a value reference that is provided from the decoder in any manner
    /// possible. Which might require additional decoding work.
    #[inline]
    fn visit_ref(self, cx: &mut C, _: &T) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::bad_visitor_type(
            &expecting::AnyValue,
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
struct ExpectingWrapper<U, C, T>(U, marker::PhantomData<(C, T)>)
where
    T: ?Sized;

impl<U, C, T> ExpectingWrapper<U, C, T>
where
    T: ?Sized,
{
    #[inline]
    fn new(value: U) -> Self {
        Self(value, marker::PhantomData)
    }
}

impl<'de, U, C, T> Expecting for ExpectingWrapper<U, C, T>
where
    U: ValueVisitor<'de, C, T>,
    C: Context,
    T: ?Sized + ToOwned,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
