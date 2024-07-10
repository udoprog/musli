use crate::buf::{Buf, Load};
use crate::endian::ByteOrder;
use crate::error::Error;
use crate::pointer::{Pointee, Ref, Size};

/// Trait used for accessing the value behind a reference when interacting with
/// higher level containers such as [`phf`] or [`swiss`].
///
/// This is a high level trait which can be implemented safely, typically it's
/// used to build facade types for when you want some type to behave like a
/// different type, but have a different layout.
///
/// See the [module level documentation][crate::buf#extension-traits] for an
/// example.
///
/// [`phf`]: crate::phf
/// [`swiss`]: crate::swiss
pub trait Visit {
    /// The target type being visited.
    type Target: ?Sized;

    /// Validate the value.
    fn visit<V, O>(&self, buf: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O;
}

impl<T: ?Sized> Visit for &T {
    type Target = T;

    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(*self))
    }
}

impl<T, E, O> Visit for Ref<T, E, O>
where
    T: ?Sized + Pointee,
    Self: Load,
    E: ByteOrder,
    O: Size,
{
    type Target = <Ref<T, E, O> as Load>::Target;

    #[inline]
    fn visit<V, U>(&self, buf: &Buf, visitor: V) -> Result<U, Error>
    where
        V: FnOnce(&Self::Target) -> U,
    {
        let value = buf.load(*self)?;
        Ok(visitor(value))
    }
}

impl Visit for str {
    type Target = str;

    #[inline]
    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(self))
    }
}

impl<T> Visit for [T] {
    type Target = [T];

    #[inline]
    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(self))
    }
}
