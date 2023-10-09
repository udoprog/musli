use core::marker::PhantomData;

use crate::buf::Buf;
use crate::error::Error;
use crate::owned_buf::OwnedBuf;
use crate::ptr::Ptr;
use crate::ZeroCopy;

/// A slice reference.
#[repr(C)]
pub struct SliceRef<T> {
    ptr: Ptr,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> SliceRef<T> {
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

unsafe impl<T> ZeroCopy for SliceRef<T> {
    fn write_to(&self, buf: &mut OwnedBuf) -> Result<(), Error> {
        buf.write(&self.ptr)?;
        buf.write(&self.len)?;
        Ok(())
    }

    fn validate(buf: &Buf) -> Result<&Self, Error> {
        let mut validator = buf.validator::<Self>()?;
        validator.validate::<Ptr>()?;
        validator.validate::<usize>()?;
        validator.finalize()?;
        Ok(unsafe { buf.cast() })
    }
}

impl<T> Clone for SliceRef<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for SliceRef<T> {}
