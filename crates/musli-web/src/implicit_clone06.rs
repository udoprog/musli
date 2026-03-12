use implicit_clone06::ImplicitClone;

use crate::Framework;
use crate::web::{Handle, WebImpl};

impl<H, F> ImplicitClone for Handle<H, F>
where
    H: WebImpl<F>,
    F: Framework,
{
    #[inline]
    fn implicit_clone(&self) -> Self {
        self.clone()
    }
}
