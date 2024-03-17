use core::fmt;

use crate::de::ValueVisitor;
use crate::no_std::ToOwned;
use crate::Context;

/// Construct a visitor from a closure.
///
/// The first parameter `expected` is what will be printed as part of an error
/// in case a type we can't handle is being visited.
///
/// # Examples
///
/// ```
/// use musli::{Context, Decode, Decoder};
///
/// #[derive(Debug, PartialEq)]
/// enum Enum {
///     A,
///     B,
/// }
///
/// impl<'de, M> Decode<'de, M> for Enum {
///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
///     where
///         C: ?Sized + Context,
///         D: Decoder<'de, C>,
///     {
///         decoder.decode_string(cx, musli::utils::visit_owned_fn("A string variant for Enum", |cx: &C, variant: &str| {
///             match variant {
///                 "A" => Ok(Enum::A),
///                 "B" => Ok(Enum::A),
///                 other => Err(cx.message(format_args!("Expected either 'A' or 'B' but got {other}"))),
///             }
///         }))
///     }
/// }
/// ```
pub fn visit_owned_fn<'de, E, U, C, T, O>(
    expected: E,
    function: U,
) -> impl ValueVisitor<'de, C, T, Ok = O>
where
    E: fmt::Display,
    U: FnOnce(&C, &T) -> Result<O, C::Error>,
    C: ?Sized + Context,
    T: ?Sized + ToOwned,
{
    FromFn { expected, function }
}

struct FromFn<E, U> {
    expected: E,
    function: U,
}

impl<'de, W, U, C, T, O> ValueVisitor<'de, C, T> for FromFn<W, U>
where
    W: fmt::Display,
    U: FnOnce(&C, &T) -> Result<O, C::Error>,
    C: ?Sized + Context,
    T: ?Sized + ToOwned,
{
    type Ok = O;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expected.fmt(f)
    }

    #[inline]
    fn visit_ref(self, cx: &C, string: &T) -> Result<Self::Ok, C::Error> {
        (self.function)(cx, string)
    }
}
