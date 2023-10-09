use core::fmt;

use crate::buf::Buf;
use crate::error::Error;
use crate::owned_buf::OwnedBuf;
use crate::to_buf::ZeroCopy;

/// A pointer to a location in a buffer.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Ptr {
    offset: usize,
}

impl Ptr {
    #[inline]
    pub(crate) fn new(offset: usize) -> Self {
        Self { offset }
    }

    #[inline]
    pub(crate) fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Debug for Ptr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Pointer(usize);

        impl fmt::Debug for Pointer {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Pointer::fmt(&(self.0 as *const ()), f)
            }
        }

        f.debug_tuple("Ptr").field(&Pointer(self.offset)).finish()
    }
}

unsafe impl ZeroCopy for Ptr {
    fn write_to(&self, buf: &mut OwnedBuf) -> Result<(), Error> {
        buf.write(&self.offset)
    }

    fn validate(buf: &Buf) -> Result<&Self, Error> {
        // SAFETY: Ptr is repr transparent over usize.
        unsafe { Ok(&*(usize::validate(buf)? as *const usize).cast()) }
    }
}
