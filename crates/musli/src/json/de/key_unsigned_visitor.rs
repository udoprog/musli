use core::fmt;
use core::marker;

use crate::Context;
use crate::de::UnsizedVisitor;
use crate::json::parser::SliceParser;
use crate::json::parser::integer::Unsigned;

use super::parse_unsigned;

pub(crate) struct KeyUnsignedVisitor<T> {
    _marker: marker::PhantomData<T>,
}

impl<T> KeyUnsignedVisitor<T> {
    pub(super) const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

#[crate::trait_defaults(crate)]
impl<C, T> UnsizedVisitor<'_, C, [u8]> for KeyUnsignedVisitor<T>
where
    C: Context,
    T: Unsigned,
{
    type Ok = T;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        parse_unsigned(cx, &mut SliceParser::new(bytes))
    }
}
