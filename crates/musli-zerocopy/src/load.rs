use crate::buf::Buf;
use crate::error::Error;
use crate::offset::TargetSize;
use crate::r#ref::Ref;
use crate::r#unsized::Unsized;
use crate::slice::Slice;
use crate::zero_copy::{UnsizedZeroCopy, ZeroCopy};

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

unsafe impl<T: ?Sized, O: TargetSize> Load for Unsized<T, O>
where
    T: UnsizedZeroCopy,
{
    type Target = T;

    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

unsafe impl<T, O: TargetSize> Load for Ref<T, O>
where
    T: ZeroCopy,
{
    type Target = T;

    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_sized(*self)
    }
}

unsafe impl<T, O: TargetSize> Load for Slice<T, O>
where
    T: ZeroCopy,
{
    type Target = [T];

    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_slice(*self)
    }
}

unsafe impl<T: ?Sized, O: TargetSize> LoadMut for Unsized<T, O>
where
    T: UnsizedZeroCopy,
{
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}

unsafe impl<T, O: TargetSize> LoadMut for Ref<T, O>
where
    T: ZeroCopy,
{
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_sized_mut(*self)
    }
}

unsafe impl<T, O: TargetSize> LoadMut for Slice<T, O>
where
    T: ZeroCopy,
{
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_slice_mut(*self)
    }
}
