use crate::buf::Buf;
use crate::endian::ByteOrder;
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

impl<P, O: Size, E: ByteOrder> Load for Ref<P, O, E>
where
    P: ZeroCopy,
{
    type Target = P;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_sized(*self)
    }
}

impl<T, O: Size, E: ByteOrder> Load for Ref<[T], O, E>
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

impl<O: Size, E: ByteOrder> Load for Ref<str, O, E> {
    type Target = str;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

impl<P, O: Size, E: ByteOrder> LoadMut for Ref<P, O, E>
where
    P: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_sized_mut(*self)
    }
}

impl<T, O: Size, E: ByteOrder> LoadMut for Ref<[T], O, E>
where
    [T]: Pointee<O, Packed = O, Metadata = usize>,
    T: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}

impl<O: Size, E: ByteOrder> LoadMut for Ref<str, O, E> {
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}
