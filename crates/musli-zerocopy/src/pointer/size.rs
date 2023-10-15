use core::fmt;

use crate::ZeroCopy;

/// The default [`Size`] in use by this crate.
pub type DefaultSize = u32;

#[cfg(not(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128"
)))]
compile_error!("musli-zerocopy is only supported on 32, 64, or 128-bit platforms");

mod sealed {
    pub trait Sealed {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for usize {}
}

/// Trait which defines the size of a pointer.
///
/// This trait is sealed, and its internals hidden. It's only public use must be
/// as a marker trait.
pub trait Size: self::sealed::Sealed + 'static + Sized + ZeroCopy + Copy + fmt::Display {
    /// The default zero pointer.
    #[doc(hidden)]
    const ZERO: Self;

    /// The max size of a pointer.
    #[doc(hidden)]
    const MAX: Self;

    /// Convert a usize to a pointer.
    #[doc(hidden)]
    fn from_usize(offset: usize) -> Option<Self>;

    /// Convert the pointer to a usize.
    #[doc(hidden)]
    fn as_usize(self) -> usize;

    /// Test if the value is zero.
    #[doc(hidden)]
    fn is_zero(self) -> bool;
}

impl Size for u16 {
    const ZERO: Self = 0;
    const MAX: Self = u16::MAX;

    fn from_usize(offset: usize) -> Option<Self> {
        if offset > u16::MAX as usize {
            None
        } else {
            Some(offset as u16)
        }
    }

    fn as_usize(self) -> usize {
        self as usize
    }

    fn is_zero(self) -> bool {
        self == 0
    }
}

impl Size for u32 {
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

impl Size for usize {
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
