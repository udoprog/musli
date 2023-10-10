use core::fmt;

use crate::ZeroCopy;

/// An absolute pointer to a location in a [`Buf`].
///
/// [`Buf`]: crate::buf::Buf
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(transparent)]
#[zero_copy(crate)]
pub struct Ptr {
    offset: usize,
}

impl Ptr {
    /// A pointer pointing to the start of a buffer.
    pub const ZERO: Self = Self { offset: 0 };

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
                write!(f, "{:x}", self.0)
            }
        }

        f.debug_tuple("Ptr").field(&Pointer(self.offset)).finish()
    }
}
