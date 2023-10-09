use core::marker::PhantomData;

use crate::buf::{Buf, BufMut};
use crate::error::Error;
use crate::ptr::Ptr;
use crate::zero_copy::ZeroCopy;

/// A sized reference.
#[repr(C)]
pub struct Ref<T> {
    ptr: Ptr,
    _marker: PhantomData<T>,
}

impl<T> Ref<T>
where
    T: ZeroCopy,
{
    /// Construct a reference wrapping the given type at the specified address.
    pub fn new(ptr: Ptr) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn ptr(&self) -> Ptr {
        self.ptr
    }
}

unsafe impl<T> ZeroCopy for Ref<T> {
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.write(&self.ptr)?;
        Ok(())
    }

    fn read_from(buf: &Buf) -> Result<&Self, Error> {
        let mut v = buf.validate::<Self>()?;
        v.field::<Ptr>()?;
        v.finalize()?;
        Ok(unsafe { buf.cast() })
    }

    unsafe fn validate_aligned(buf: &Buf) -> Result<(), Error> {
        let mut v = buf.validate_aligned()?;
        v.field::<Ptr>()?;
        v.finalize()
    }
}

impl<T> Clone for Ref<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Ref<T> {}
