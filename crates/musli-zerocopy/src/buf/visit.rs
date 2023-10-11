use crate::buf::{Buf, Load};
use crate::error::Error;

pub(crate) mod sealed {
    use crate::buf::Load;

    pub trait Sealed {}

    impl<T: ?Sized> Sealed for T where T: Load {}
}

/// Trait used for handling any kind of zero copy value, be they references or
/// not.
pub trait Visit: self::sealed::Sealed {
    /// The target being read.
    type Target: ?Sized;

    /// Validate the value.
    fn visit<V, O>(&self, buf: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O;
}

impl<T: ?Sized> Visit for T
where
    T: Load,
{
    type Target = T::Target;

    fn visit<V, O>(&self, buf: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        let value = buf.load(self)?;
        Ok(visitor(value))
    }
}
