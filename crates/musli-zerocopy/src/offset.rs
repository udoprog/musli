use core::fmt;

use crate::ZeroCopy;

/// The default [`TargetSize`] to use.
pub(crate) type DefaultTargetSize = u32;

#[cfg(not(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128"
)))]
compile_error!("musli-zerocopy is only supported on 32, 64, or 128-bit platforms");

/// Trait which defines the size of a pointer.
pub trait TargetSize: 'static + Sized + ZeroCopy + fmt::Display + Copy {
    /// The default zero pointer.
    const ZERO: Self;
    /// The max size of a pointer.
    const MAX: Self;

    /// Convert a usize to a pointer.
    fn from_usize(offset: usize) -> Option<Self>;

    /// Convert the pointer to a usize.
    fn as_usize(self) -> usize;

    /// Test if the value is zero.
    fn is_zero(self) -> bool;
}

impl TargetSize for u32 {
    const ZERO: Self = 0;
    const MAX: Self = u32::MAX;

    fn from_usize(offset: usize) -> Option<Self> {
        if offset > u32::MAX as usize {
            None
        } else {
            Some(offset as u32)
        }
    }

    fn as_usize(self) -> usize {
        self as usize
    }

    fn is_zero(self) -> bool {
        self == 0
    }
}

impl TargetSize for usize {
    const ZERO: Self = 0;
    const MAX: Self = usize::MAX;

    fn from_usize(offset: usize) -> Option<Self> {
        Some(offset)
    }

    fn as_usize(self) -> usize {
        self
    }

    fn is_zero(self) -> bool {
        self == 0
    }
}

/// An absolute pointer to a location in a [`Buf`].
///
/// [`Buf`]: crate::buf::Buf
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(transparent)]
#[zero_copy(crate)]
pub struct Offset<O: TargetSize = DefaultTargetSize> {
    offset: O,
}

impl<O: TargetSize> Offset<O> {
    /// A pointer pointing to the start of a buffer.
    pub const ZERO: Self = Self { offset: O::ZERO };

    /// Construct a new offset with a custom pointer size.
    #[inline]
    pub fn new(offset: usize) -> Self {
        let Some(offset) = O::from_usize(offset) else {
            panic!("Offset {offset} not in the legal range of 0-{}", O::MAX);
        };

        Self { offset }
    }

    #[inline]
    pub(crate) fn offset(&self) -> usize {
        self.offset.as_usize()
    }
}

impl<O: TargetSize> fmt::Debug for Offset<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Offset(usize);

        impl fmt::Debug for Offset {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:x}", self.0)
            }
        }

        f.debug_tuple("Offset")
            .field(&Offset(self.offset.as_usize()))
            .finish()
    }
}
