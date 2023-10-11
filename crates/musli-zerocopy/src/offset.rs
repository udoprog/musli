use core::fmt;

use crate::ZeroCopy;

/// The size of a pointer.
pub(crate) type OffsetSize = u32;

#[cfg(not(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128"
)))]
compile_error!("musli-zerocopy is only supported on 32, 64, or 128-bit platforms");

/// An absolute pointer to a location in a [`Buf`].
///
/// [`Buf`]: crate::buf::Buf
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(transparent)]
#[zero_copy(crate)]
pub struct Offset {
    offset: OffsetSize,
}

impl Offset {
    /// A pointer pointing to the start of a buffer.
    pub const ZERO: Self = Self { offset: 0 };

    /// Construct a new offset.
    #[inline]
    pub fn new(offset: usize) -> Self {
        let Ok(offset) = OffsetSize::try_from(offset) else {
            panic!(
                "Offset {offset} not in the legal range of 0-{}",
                OffsetSize::MAX
            );
        };

        Self { offset }
    }

    #[inline]
    pub(crate) fn offset(&self) -> usize {
        self.offset as usize
    }
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Offset(OffsetSize);

        impl fmt::Debug for Offset {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:x}", self.0)
            }
        }

        f.debug_tuple("Offset").field(&Offset(self.offset)).finish()
    }
}
