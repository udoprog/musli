use crate::buf::Buf;
use crate::error::Error;
use crate::pointer::{Pointee, Ref, Size};
use crate::traits::ZeroCopy;

/// Trait used for loading any kind of reference through [`Buf::load`].
///
/// This is a high level trait which can be implemented safely, typically it's
/// used to build facade types for when you want some type to behave like a
/// different type, but have a different layout.
///
/// See the [module level documentation][crate::buf#extension-traits] for an
/// example.
pub trait Load {
    /// The target being read.
    type Target: ?Sized;

    /// Validate the value.
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error>;
}

/// Trait used for loading any kind of reference through [`Buf::load_mut`].
///
/// This is a high level trait which can be implemented safely, typically it's
/// used to build facade types for when you want some type to behave like a
/// different type, but have a different layout.
///
/// See the [module level documentation][crate::buf#extension-traits] for an
/// example.
pub trait LoadMut: Load {
    /// Validate the value.
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error>;
}

impl<T: ?Sized> Load for &T
where
    T: Load,
{
    type Target = T::Target;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        T::load(self, buf)
    }
}

impl<T: ?Sized> Load for &mut T
where
    T: Load,
{
    type Target = T::Target;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        T::load(self, buf)
    }
}

impl<T: ?Sized> LoadMut for &mut T
where
    T: LoadMut,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        T::load_mut(self, buf)
    }
}

impl<P, O: Size> Load for Ref<P, O>
where
    P: ZeroCopy,
{
    type Target = P;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_sized(*self)
    }
}

impl<T, O: Size> Load for Ref<[T], O>
where
    [T]: Pointee<O, Packed = O, Metadata = usize>,
    T: ZeroCopy,
{
    type Target = [T];

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

impl<O: Size> Load for Ref<str, O> {
    type Target = str;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

impl<P, O: Size> LoadMut for Ref<P, O>
where
    P: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_sized_mut(*self)
    }
}

impl<T, O: Size> LoadMut for Ref<[T], O>
where
    [T]: Pointee<O, Packed = O, Metadata = usize>,
    T: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}

impl<O: Size> LoadMut for Ref<str, O> {
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}
