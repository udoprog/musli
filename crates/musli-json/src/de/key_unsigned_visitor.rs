use core::fmt;
use core::marker;

use musli::de::ValueVisitor;
use musli::Context;

use crate::error::Error;
use crate::reader::integer::Unsigned;
use crate::reader::SliceParser;

use super::parse_unsigned;

pub(crate) struct KeyUnsignedVisitor<C, T> {
    _marker: marker::PhantomData<(C, T)>,
}

impl<C, T> KeyUnsignedVisitor<C, T> {
    pub(super) const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, C, T> ValueVisitor<'de, C, [u8]> for KeyUnsignedVisitor<C, T>
where
    C: Context<Input = Error>,
    T: Unsigned,
{
    type Ok = T;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes")
    }

    #[inline]
    fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        parse_unsigned(cx, &mut SliceParser::new(bytes))
    }
}
