use crate::buf::Buf;
use crate::endian::ByteOrder;
use crate::error::Error;
use crate::pointer::{Ref, Size};
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

impl<T, E: ByteOrder, O: Size> Load for Ref<T, E, O>
where
    T: ZeroCopy,
{
    type Target = T;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_sized::<T>(self.offset())
    }
}

impl<T, E: ByteOrder, O: Size> Load for Ref<[T], E, O>
where
    T: ZeroCopy,
{
    type Target = [T];

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

impl<E: ByteOrder, O: Size> Load for Ref<str, E, O> {
    type Target = str;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

impl<T, E: ByteOrder, O: Size> LoadMut for Ref<T, E, O>
where
    T: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_sized_mut::<T>(self.offset())
    }
}

impl<T, E: ByteOrder, O: Size> LoadMut for Ref<[T], E, O>
where
    T: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}

impl<E: ByteOrder, O: Size> LoadMut for Ref<str, E, O> {
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}
