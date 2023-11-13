use core::fmt;

use crate::endian::ByteOrder;
use crate::error::IntoRepr;
use crate::traits::ZeroCopy;

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
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
    impl Sealed for u64 {}
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
    'static
    + Sized
    + TryFrom<usize>
    + Copy
    + fmt::Display
    + fmt::Debug
    + ZeroCopy
    + IntoRepr
    + self::sealed::Sealed
{
    /// The default zero pointer.
    #[doc(hidden)]
    const ZERO: Self;

    /// The max size of a pointer.
    #[doc(hidden)]
    const MAX: Self;

    /// Try to construct this value from usize.
    fn try_from_usize(value: usize) -> Option<Self>;

    /// Convert the pointer to a usize.
    #[doc(hidden)]
    fn as_usize<E: ByteOrder>(self) -> usize;

    /// Test if the value is zero.
    #[doc(hidden)]
    fn is_zero(self) -> bool;
}

macro_rules! impl_size {
    ($ty:ty, $swap:path) => {
        #[doc = concat!("Size implementation for `", stringify!($ty), "`")]
        ///
        /// # Examples
        ///
        /// ```
        /// use musli_zerocopy::{endian, Size};
        ///
        #[doc = concat!("let max = ", stringify!($ty), "::MAX.as_usize::<endian::Big>();")]
        #[doc = concat!("let min = ", stringify!($ty), "::MIN.as_usize::<endian::Little>();")]
        /// ```
        impl Size for $ty {
            const ZERO: Self = 0;
            const MAX: Self = <$ty>::MAX;

            #[inline]
            fn try_from_usize(value: usize) -> Option<Self> {
                if value > <$ty>::MAX as usize {
                    None
                } else {
                    Some(value as $ty)
                }
            }

            #[inline]
            fn as_usize<E: ByteOrder>(self) -> usize {
                if self > usize::MAX as $ty {
                    usize::MAX
                } else {
                    $swap(self) as usize
                }
            }

            #[inline]
            fn is_zero(self) -> bool {
                self == 0
            }
        }
    };
}

impl_size!(u8, core::convert::identity);
impl_size!(u16, E::swap_u16);
impl_size!(u32, E::swap_u32);
#[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
impl_size!(u64, E::swap_u64);
impl_size!(usize, core::convert::identity);
