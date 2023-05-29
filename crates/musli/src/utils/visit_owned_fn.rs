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
///     fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
///     where
///         C: Context<'buf, Input = D::Error>,
///         D: Decoder<'de>,
///     {
///         decoder.decode_string(cx, musli::utils::visit_owned_fn("a string variant for Enum", |cx: &mut C, variant: &str| {
///             match variant {
///                 "A" => Ok(Enum::A),
///                 "B" => Ok(Enum::A),
///                 other => Err(cx.message("expected either 'A' or 'B'")),
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
pub fn visit_owned_fn<'de, 'buf, E, U, C, T, O>(
    expected: E,
    function: U,
) -> impl ValueVisitor<'de, 'buf, C, T, Ok = O>
where
    E: fmt::Display,
    U: FnOnce(&mut C, &T) -> Result<O, C::Error>,
    C: Context<'buf>,
    T: ?Sized + ToOwned,
{
    FromFn { expected, function }
}

struct FromFn<E, U> {
    expected: E,
    function: U,
}

impl<'de, 'buf, W, U, C, T, O> ValueVisitor<'de, 'buf, C, T> for FromFn<W, U>
where
    W: fmt::Display,
    U: FnOnce(&mut C, &T) -> Result<O, C::Error>,
    T: ?Sized + ToOwned,
    C: Context<'buf>,
{
    type Ok = O;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expected)
    }

    #[inline]
    fn visit_ref(self, cx: &mut C, string: &T) -> Result<Self::Ok, C::Error> {
        (self.function)(cx, string)
    }
}
