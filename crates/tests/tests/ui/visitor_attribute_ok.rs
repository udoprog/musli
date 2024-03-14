use core::fmt;
use core::convert::Infallible;

use musli::de::Visitor;

struct AnyVisitor;

#[musli::visitor]
impl<'de> Visitor<'de> for AnyVisitor {
    type Ok = ();
    type Error = Infallible;

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
