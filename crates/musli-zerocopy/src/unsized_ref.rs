use core::marker::PhantomData;

use crate::buf::{Buf, BufMut};
use crate::error::Error;
use crate::ptr::Ptr;
use crate::zero_copy::ZeroCopy;

/// An unsized reference.
#[derive(Debug)]
#[repr(C)]
pub struct UnsizedRef<T: ?Sized> {
    ptr: Ptr,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T: ?Sized> UnsizedRef<T> {
    pub(crate) fn new(ptr: Ptr, len: usize) -> Self {
        Self {
            ptr,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn ptr(&self) -> Ptr {
        self.ptr
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

unsafe impl<T: ?Sized> ZeroCopy for UnsizedRef<T> {
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.write(&self.ptr)?;
        buf.write(&self.len)?;
        Ok(())
    }

    fn read_from(buf: &Buf) -> Result<&Self, Error> {
        let mut v = buf.validate::<Self>()?;
        v.field::<Ptr>()?;
        v.field::<usize>()?;
        v.finalize()?;
        Ok(unsafe { buf.cast() })
    }

    unsafe fn validate_aligned(buf: &Buf) -> Result<(), Error> {
        let mut v = buf.validate_aligned()?;
        v.field::<Ptr>()?;
        v.field::<usize>()?;
        v.finalize()
    }
}

impl<T: ?Sized> Clone for UnsizedRef<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Copy for UnsizedRef<T> {}
