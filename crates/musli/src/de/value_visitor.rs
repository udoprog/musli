use core::borrow::Borrow;
use core::fmt;

use crate::error::Error;
use crate::expecting::{self, Expecting};
use crate::no_std::ToOwned;

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
pub trait ValueVisitor<'de>: Sized {
    /// The value being visited.
    type Target: ?Sized + ToOwned;
    /// The value produced.
    type Ok;
    /// The error produced.
    type Error: Error;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit an owned value.
    #[inline]
    fn visit_owned(self, value: <Self::Target as ToOwned>::Owned) -> Result<Self::Ok, Self::Error> {
        self.visit_any(value.borrow())
    }

    /// Visit a string that is borrowed directly from the source data.
    #[inline]
    fn visit_borrowed(self, value: &'de Self::Target) -> Result<Self::Ok, Self::Error> {
        self.visit_any(value)
    }

    /// Visit a string that is provided from the decoder in any manner possible.
    /// Which might require additional decoding work.
    #[inline]
    fn visit_any(self, _: &Self::Target) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::bad_visitor_type(
            &expecting::AnyValue,
            &ValueVisitorExpecting(self),
        )))
    }
}

#[repr(transparent)]
struct ValueVisitorExpecting<T>(T);

impl<'de, T> Expecting for ValueVisitorExpecting<T>
where
    T: ValueVisitor<'de>,
{
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
