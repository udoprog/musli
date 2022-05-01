use core::fmt;
use core::marker;

use crate::de::ValueVisitor;
use crate::error::Error;

/// Construct an anonymous bytes visitor from a function.
pub fn bytes_visitor_fn<'de, T, O, E>(
    function: T,
) -> impl ValueVisitor<'de, Target = [u8], Ok = O, Error = E>
where
    T: FnOnce(&[u8]) -> Result<O, E>,
    E: Error,
{
    FromFn {
        function,
        _marker: marker::PhantomData,
    }
}

struct FromFn<T, O, E> {
    function: T,
    _marker: marker::PhantomData<(O, E)>,
}

impl<'de, T, O, E> ValueVisitor<'de> for FromFn<T, O, E>
where
    T: FnOnce(&[u8]) -> Result<O, E>,
    E: Error,
{
    type Target = [u8];
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
