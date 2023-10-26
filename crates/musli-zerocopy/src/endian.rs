//! Marker types which define a [`ByteOrder`] to use.

/// Default [`ByteOrder`].
pub type DefaultEndian = NativeEndian;

/// Alias for the native endian [`ByteOrder`].
#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

/// Alias for the native endian [`ByteOrder`].
#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

/// Marker type indicating that the big endian [`ByteOrder`] is in use.
#[non_exhaustive]
pub struct BigEndian;

/// Marker type indicating that the little endian [`ByteOrder`] is in use.
#[non_exhaustive]
pub struct LittleEndian;

mod sealed {
    use super::{BigEndian, LittleEndian};

    pub trait Sealed {}

    impl Sealed for BigEndian {}
    impl Sealed for LittleEndian {}
}

/// Defines a byte order to use.
///
/// This trait is implemented by two marker types [`BigEndian`] and
/// [`LittleEndian`], and its internals are intentionally hidden. Do not attempt
/// to use them yourself.
pub trait ByteOrder: 'static + Sized + self::sealed::Sealed {
    /// Swap the bytes for a `usize` with the current byte order.
    #[doc(hidden)]
    fn swap_usize(value: usize) -> usize;

    /// Swap the bytes for a `isize` with the current byte order.
    #[doc(hidden)]
    fn swap_isize(value: isize) -> isize;

    /// Swap the bytes of a `u16` with the current byte order.
    #[doc(hidden)]
    fn swap_u16(value: u16) -> u16;

    /// Swap the bytes of a `i16` with the current byte order.
    #[doc(hidden)]
    fn swap_i16(value: i16) -> i16;

    /// Swap the bytes for a `u32` with the current byte order.
    #[doc(hidden)]
    fn swap_u32(value: u32) -> u32;

    /// Swap the bytes for a `i32` with the current byte order.
    #[doc(hidden)]
    fn swap_i32(value: i32) -> i32;

    /// Swap the bytes for a `u64` with the current byte order.
    #[doc(hidden)]
    fn swap_u64(value: u64) -> u64;

    /// Swap the bytes for a `i64` with the current byte order.
    #[doc(hidden)]
    fn swap_i64(value: i64) -> i64;

    /// Swap the bytes for a `u128` with the current byte order.
    #[doc(hidden)]
    fn swap_u128(value: u128) -> u128;

    /// Swap the bytes for a `i128` with the current byte order.
    #[doc(hidden)]
    fn swap_i128(value: i128) -> i128;

    /// Swap the bytes for a `f32` with the current byte order.
    #[doc(hidden)]
    fn swap_f32(value: f32) -> f32;

    /// Swap the bytes for a `f64` with the current byte order.
    #[doc(hidden)]
    fn swap_f64(value: f64) -> f64;
}

impl ByteOrder for LittleEndian {
    #[inline]
    fn swap_usize(value: usize) -> usize {
        usize::from_le(value)
    }

    #[inline]
    fn swap_isize(value: isize) -> isize {
        isize::from_le(value)
    }

    #[inline]
    fn swap_u16(value: u16) -> u16 {
        u16::to_le(value)
    }

    #[inline]
    fn swap_i16(value: i16) -> i16 {
        i16::to_le(value)
    }

    #[inline]
    fn swap_u32(value: u32) -> u32 {
        u32::from_le(value)
    }

    #[inline]
    fn swap_i32(value: i32) -> i32 {
        i32::from_le(value)
    }

    #[inline]
    fn swap_u64(value: u64) -> u64 {
        u64::from_le(value)
    }

    #[inline]
    fn swap_i64(value: i64) -> i64 {
        i64::from_le(value)
    }

    #[inline]
    fn swap_u128(value: u128) -> u128 {
        u128::from_le(value)
    }

    #[inline]
    fn swap_i128(value: i128) -> i128 {
        i128::from_le(value)
    }

    #[inline]
    fn swap_f32(value: f32) -> f32 {
        f32::from_bits(u32::from_le(value.to_bits()))
    }

    #[inline]
    fn swap_f64(value: f64) -> f64 {
        f64::from_bits(u64::from_le(value.to_bits()))
    }
}

impl ByteOrder for BigEndian {
    #[inline]
    fn swap_usize(value: usize) -> usize {
        usize::from_be(value)
    }

    #[inline]
    fn swap_isize(value: isize) -> isize {
        isize::from_be(value)
    }

    #[inline]
    fn swap_u16(value: u16) -> u16 {
        u16::to_be(value)
    }

    #[inline]
    fn swap_i16(value: i16) -> i16 {
        i16::to_be(value)
    }

    #[inline]
    fn swap_u32(value: u32) -> u32 {
        u32::from_be(value)
    }

    #[inline]
    fn swap_i32(value: i32) -> i32 {
        i32::from_be(value)
    }

    #[inline]
    fn swap_u64(value: u64) -> u64 {
        u64::from_be(value)
    }

    #[inline]
    fn swap_i64(value: i64) -> i64 {
        i64::from_be(value)
    }

    #[inline]
    fn swap_u128(value: u128) -> u128 {
        u128::from_be(value)
    }

    #[inline]
    fn swap_i128(value: i128) -> i128 {
        i128::from_be(value)
    }

    #[inline]
    fn swap_f32(value: f32) -> f32 {
        f32::from_bits(u32::from_be(value.to_bits()))
    }

    #[inline]
    fn swap_f64(value: f64) -> f64 {
        f64::from_bits(u64::from_be(value.to_bits()))
    }
}
