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
/// use musli::{Context, Decode, Decoder, Mode};
///
/// #[derive(Debug, PartialEq)]
/// enum Enum {
///     A,
///     B,
/// }
///
/// impl<'de, M> Decode<'de, M> for Enum
/// where
///     M: Mode,
/// {
///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
///     where
///         C: Context<Input = D::Error>,
///         D: Decoder<'de>,
///     {
///         decoder.decode_string(cx, musli::utils::visit_owned_fn("A string variant for Enum", |cx: &mut C, variant: &str| {
///             match variant {
///                 "A" => Ok(Enum::A),
///                 "B" => Ok(Enum::A),
///                 other => Err(cx.message("Expected either 'A' or 'B'")),
///             }
///         }))
///     }
/// }
///
/// let value = musli_value::Value::String("A".to_string());
///
/// assert_eq!(musli_value::decode::<Enum>(&value)?, Enum::A);
/// Ok::<_, musli_value::Error>(())
/// ```
pub fn visit_owned_fn<'de, E, U, C, T, O>(
    expected: E,
    function: U,
) -> impl ValueVisitor<'de, C, T, Ok = O>
where
    E: fmt::Display,
    U: FnOnce(&mut C, &T) -> Result<O, C::Error>,
    C: Context,
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
    U: FnOnce(&mut C, &T) -> Result<O, C::Error>,
    T: ?Sized + ToOwned,
    C: Context,
{
    type Ok = O;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expected.fmt(f)
    }

    #[inline]
    fn visit_ref(self, cx: &mut C, string: &T) -> Result<Self::Ok, C::Error> {
        (self.function)(cx, string)
    }
}
