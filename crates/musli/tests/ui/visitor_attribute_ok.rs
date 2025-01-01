use core::fmt;

use musli::de::Visitor;
use musli::Context;

struct AnyVisitor;

#[musli::visitor]
impl<'de, C> Visitor<'de, C> for AnyVisitor where C: Context {
    type Ok = ();

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
