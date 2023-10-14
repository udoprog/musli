use crate::buf::{Buf, Load};
use crate::error::Error;

/// Trait used for handling any kind of zero copy value, be they references or
/// not.
pub trait Visit {
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

impl Visit for str {
    type Target = str;

    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(self))
    }
}

impl<T> Visit for [T] {
    type Target = [T];

    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(self))
    }
}
