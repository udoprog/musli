use implicit_clone06::ImplicitClone;

use crate::web::{Handle, WebImpl};

impl<H> ImplicitClone for Handle<H>
where
    H: WebImpl,
{
    #[inline]
    fn implicit_clone(&self) -> Self {
        self.clone()
    }
}
