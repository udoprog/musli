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
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for usize {}
}

/// Trait which defines the size of a pointer.
///
/// Some of the available [`Size`] implementations are:
/// * `u8`, `u16`, and `u32` for sized pointers matching the width of the
///   specified type.
/// * `usize` for target-dependently sized pointers.
///
/// The default size is defined by the [`DefaultSize`] type alias.
///
/// This trait is sealed and its internals hidden. Publicly it's only used as a
/// marker trait.
pub trait Size:
    TryFrom<usize>
    + self::sealed::Sealed
    + 'static
    + Sized
    + ZeroCopy
    + Copy
    + fmt::Display
    + fmt::Debug
{
    /// The default zero pointer.
    #[doc(hidden)]
    const ZERO: Self;

    /// The max size of a pointer.
    #[doc(hidden)]
    const MAX: Self;

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

    #[inline]
    fn as_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn is_zero(self) -> bool {
        self == 0
    }
}

impl Size for u32 {
    const ZERO: Self = 0;
    const MAX: Self = u32::MAX;

    #[inline]
    fn as_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn is_zero(self) -> bool {
        self == 0
    }
}

impl Size for usize {
    const ZERO: Self = 0;
    const MAX: Self = usize::MAX;

    #[inline]
    fn as_usize(self) -> usize {
        self
    }

    #[inline]
    fn is_zero(self) -> bool {
        self == 0
    }
}
