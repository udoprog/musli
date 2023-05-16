use core::fmt;
use core::marker;

use musli::de::Visitor;
use musli::error::Error;

struct AnyVisitor<E> {
    _marker: marker::PhantomData<E>,
}

#[musli::visitor]
impl<'de, E> Visitor<'de> for AnyVisitor<E>
where
    E: Error,
{
    type Ok = ();
    type Error = E;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "value that can be decoded into dynamic container"
        )
    }
}

fn main() {
}
