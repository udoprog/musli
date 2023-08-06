use core::fmt;
use core::mem;

use crate::buf::Buf;
use crate::error::Error;
use crate::owned_buf::OwnedBuf;
use crate::traits::{Read, Size, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Ptr(usize);

impl Ptr {
    #[inline]
    pub(crate) fn new(at: usize) -> Self {
        Self(at)
    }

    #[inline]
    pub(crate) fn wrapping_add(self, rhs: usize) -> Self {
        Self(self.0.wrapping_add(rhs))
    }

    #[inline]
    pub(crate) fn as_usize(self) -> usize {
        self.0
    }
}

impl Write for Ptr {
    #[inline]
    fn write(&self, buf: &mut OwnedBuf) {
        self.0.write(buf);
    }
}

impl Read<'_> for Ptr {
    fn read(buf: &'_ Buf, ptr: Ptr) -> Result<Self, Error> {
        Ok(Ptr(usize::read(buf, ptr)?))
    }
}

impl Size for Ptr {
    #[inline]
    fn size() -> usize {
        mem::size_of::<usize>()
    }
}

impl fmt::Display for Ptr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}
