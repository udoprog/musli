use core::fmt;
use core::marker;

use crate::de::UnsizedVisitor;
use crate::json::parser::integer::Signed;
use crate::json::parser::SliceParser;
use crate::Context;

use super::parse_signed;

pub(crate) struct KeySignedVisitor<T> {
    _marker: marker::PhantomData<T>,
}

impl<T> KeySignedVisitor<T> {
    #[inline]
    pub(super) const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<C: ?Sized + Context, T> UnsizedVisitor<'_, C, [u8]> for KeySignedVisitor<T>
where
    T: Signed,
{
    type Ok = T;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        parse_signed(cx, &mut SliceParser::new(bytes))
    }
}
