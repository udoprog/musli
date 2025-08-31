use core::fmt;

use crate::endian::ByteOrder;
use crate::error::{CoerceError, CoerceErrorKind};
use crate::traits::ZeroCopy;

/// The default [`Size`] to use.
pub type DefaultSize = u32;

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64",)))]
compile_error!("musli-zerocopy is only supported on 32, 64-bit platforms");

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    #[cfg(target_pointer_width = "64")]
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
    + self::sealed::Sealed
{
    /// The default zero pointer.
    #[doc(hidden)]
    const ZERO: Self;

    /// The max size of a pointer.
    #[doc(hidden)]
    const MAX: Self;

    #[doc(hidden)]
    const ONE: Self;

    #[doc(hidden)]
    const N2: Self;

    #[doc(hidden)]
    const N4: Self;

    #[doc(hidden)]
    const N8: Self;

    #[doc(hidden)]
    const N16: Self;

    #[doc(hidden)]
    /// Perform wrapping multiplication over the type.
    fn wrapping_mul(self, other: Self) -> Self;

    #[doc(hidden)]
    /// Perform checked multiplication over the type.
    fn checked_mul(self, other: Self) -> Option<Self>;

    /// Try to construct this value from usize.
    fn try_from_usize(value: usize) -> Result<Self, CoerceError>;

    /// Convert the pointer to a usize.
    #[doc(hidden)]
    fn as_usize<E>(self) -> usize
    where
        E: ByteOrder;

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
            const ONE: Self = 1;
            const N2: Self = 2;
            const N4: Self = 4;
            const N8: Self = 8;
            const N16: Self = 16;

            #[inline(always)]
            fn wrapping_mul(self, other: Self) -> Self {
                self.wrapping_mul(other)
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                self.checked_mul(other)
            }

            #[inline]
            fn try_from_usize(value: usize) -> Result<Self, CoerceError> {
                if value > <$ty>::MAX as usize {
                    Err(CoerceError::new(CoerceErrorKind::LengthOverflow {
                        len: value,
                        size: <$ty>::MAX as usize,
                    }))
                } else {
                    Ok(value as $ty)
                }
            }

            #[inline]
            fn as_usize<E>(self) -> usize
            where
                E: ByteOrder,
            {
                debug_assert!(
                    usize::try_from($swap(self)).is_ok(),
                    "Value {self} cannot be represented on this platform"
                );
                $swap(self) as usize
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
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_size!(u32, E::swap_u32);
#[cfg(target_pointer_width = "64")]
impl_size!(u64, E::swap_u64);
impl_size!(usize, E::swap_usize);
