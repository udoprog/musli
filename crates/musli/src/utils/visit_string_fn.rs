use core::fmt;
use core::marker;

use crate::de::ValueVisitor;
use crate::error::Error;
use crate::Context;

/// Construct an anonymous string visitor from a function.
pub fn visit_string_fn<'de, T, C, O, E>(
    function: T,
) -> impl ValueVisitor<'de, Context = C, Target = str, Ok = O, Error = E>
where
    T: FnOnce(&mut C, &str) -> Result<O, C::Error>,
    C: Context<E>,
    E: Error,
{
    FromFn {
        function,
        _marker: marker::PhantomData,
    }
}

struct FromFn<T, C, O, E> {
    function: T,
    _marker: marker::PhantomData<(C, O, E)>,
}

impl<'de, T, C, O, E> ValueVisitor<'de> for FromFn<T, C, O, E>
where
    T: FnOnce(&mut C, &str) -> Result<O, C::Error>,
    C: Context<E>,
    E: Error,
{
    type Target = str;
    type Ok = O;
    type Error = E;
    type Context = C;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "string")
    }

    #[inline]
    fn visit_ref(
        self,
        cx: &mut C,
        string: &Self::Target,
    ) -> Result<Self::Ok, <C as Context<Self::Error>>::Error>
    where
        C: Context<Self::Error>,
    {
        (self.function)(cx, string)
    }
}
