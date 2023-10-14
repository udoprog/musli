use crate::buf::Buf;
use crate::error::Error;
use crate::pointer::{Ref, Size, Slice, Unsized};
use crate::traits::{UnsizedZeroCopy, ZeroCopy};

mod sealed {
    use crate::pointer::{Ref, Size, Slice, Unsized};
    use crate::traits::{UnsizedZeroCopy, ZeroCopy};

    pub trait Sealed {}

    impl<T, O: Size> Sealed for Ref<T, O> where T: ZeroCopy {}
    impl<T, O: Size> Sealed for Slice<T, O> where T: ZeroCopy {}
    impl<T: ?Sized, O: Size> Sealed for Unsized<T, O> where T: UnsizedZeroCopy {}
    impl<T: ?Sized> Sealed for &T where T: Sealed {}
    impl<T: ?Sized> Sealed for &mut T where T: Sealed {}
}

/// Trait used for loading any kind of reference.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
pub unsafe trait Load {
    /// The target being read.
    type Target: ?Sized;

    /// Validate the value.
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error>;
}

/// Trait used for loading any kind of reference.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
pub unsafe trait LoadMut: Load {
    /// Validate the value.
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error>;
}

// SAFETY: Blanket implementation is fine over known sound implementations.
unsafe impl<T: ?Sized> Load for &T
where
    T: Load,
{
    type Target = T::Target;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        T::load(self, buf)
    }
}

// SAFETY: Blanket implementation is fine over known sound implementations.
unsafe impl<T: ?Sized> Load for &mut T
where
    T: Load,
{
    type Target = T::Target;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        T::load(self, buf)
    }
}

// SAFETY: Blanket implementation is fine over known sound implementations.
unsafe impl<T: ?Sized> LoadMut for &mut T
where
    T: LoadMut,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        T::load_mut(self, buf)
    }
}

unsafe impl<T: ?Sized, O: Size> Load for Unsized<T, O>
where
    T: UnsizedZeroCopy,
{
    type Target = T;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

unsafe impl<T, O: Size> Load for Ref<T, O>
where
    T: ZeroCopy,
{
    type Target = T;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_sized(*self)
    }
}

unsafe impl<T, O: Size> Load for Slice<T, O>
where
    T: ZeroCopy,
{
    type Target = [T];

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_slice(*self)
    }
}

unsafe impl<T: ?Sized, O: Size> LoadMut for Unsized<T, O>
where
    T: UnsizedZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}

unsafe impl<T, O: Size> LoadMut for Ref<T, O>
where
    T: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_sized_mut(*self)
    }
}

unsafe impl<T, O: Size> LoadMut for Slice<T, O>
where
    T: ZeroCopy,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_slice_mut(*self)
    }
}
