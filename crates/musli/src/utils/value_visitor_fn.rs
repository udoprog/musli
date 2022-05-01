use core::fmt;
use core::marker;

use crate::de::ValueVisitor;
use crate::error::Error;
use crate::no_std::ToOwned;

/// Construct an anonymous value visitor from a function.
pub fn value_visitor_fn<'de, T, O, E, U>(
    function: T,
) -> impl ValueVisitor<'de, Target = U, Ok = O, Error = E>
where
    T: FnOnce(&U) -> Result<O, E>,
    E: Error,
    U: ToOwned,
{
    FromFn {
        function,
        _marker: marker::PhantomData,
    }
}

struct FromFn<T, O, E, U> {
    function: T,
    _marker: marker::PhantomData<(O, E, U)>,
}

impl<'de, T, O, E, U> ValueVisitor<'de> for FromFn<T, O, E, U>
where
    T: FnOnce(&U) -> Result<O, E>,
    E: Error,
    U: ToOwned,
{
    type Target = U;
    type Ok = O;
    type Error = E;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "string")
    }

    #[inline]
    fn visit_any(self, string: &Self::Target) -> Result<Self::Ok, Self::Error> {
        (self.function)(string)
    }
}
