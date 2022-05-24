use core::fmt::Debug;
use core::hash::Hash;

mod private {
    pub trait Sealed {}
    impl Sealed for super::LittleEndian {}
    impl Sealed for super::BigEndian {}
}

/// Trait governing byte orders.
pub trait ByteOrder:
    Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd + private::Sealed
{
    /// Read a single byte.
    fn read_u8(bytes: [u8; 1]) -> u8;

    /// Read a 16-bit unsigned integer.
    fn read_u16(bytes: [u8; 2]) -> u16;

    /// Read a 32-bit unsigned integer.
    fn read_u32(bytes: [u8; 4]) -> u32;

    /// Read a 64-bit unsigned integer.
    fn read_u64(bytes: [u8; 8]) -> u64;

    /// Read a 128-bit unsigned integer.
    fn read_u128(bytes: [u8; 16]) -> u128;

    /// Read a 16-bit signed integer.
    fn read_i16(bytes: [u8; 2]) -> i16;

    /// Read a 32-bit signed integer.
    fn read_i32(bytes: [u8; 4]) -> i32;

    /// Read a 64-bit signed integer.
    fn read_i64(bytes: [u8; 8]) -> i64;

    /// Read a 128-bit signed integer.
    fn read_i128(bytes: [u8; 16]) -> i128;

    /// Write a single byte.
    fn write_u8(value: u8) -> [u8; 1];

    /// Write a 16-bit unsigned integer.
    fn write_u16(value: u16) -> [u8; 2];

    /// Write a 32-bit unsigned integer.
    fn write_u32(value: u32) -> [u8; 4];

    /// Write a 64-bit unsigned integer.
    fn write_u64(value: u64) -> [u8; 8];

    /// Write a 128-bit unsigned integer.
    fn write_u128(value: u128) -> [u8; 16];

    /// Write a 16-bit signed integer.
    fn write_i16(value: i16) -> [u8; 2];

    /// Write a 32-bit signed integer.
    fn write_i32(value: i32) -> [u8; 4];

    /// Write a 64-bit signed integer.
    fn write_i64(value: i64) -> [u8; 8];

    /// Write a 128-bit signed integer.
    fn write_i128(value: i128) -> [u8; 16];
}

/// Defines little-endian serialization.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LittleEndian {}

impl ByteOrder for LittleEndian {
    #[inline]
    fn read_u8(bytes: [u8; 1]) -> u8 {
        bytes[0]
    }

    #[inline]
    fn read_u16(bytes: [u8; 2]) -> u16 {
        u16::from_le_bytes(bytes)
    }

    #[inline]
    fn read_u32(bytes: [u8; 4]) -> u32 {
        u32::from_le_bytes(bytes)
    }

    #[inline]
    fn read_u64(bytes: [u8; 8]) -> u64 {
        u64::from_le_bytes(bytes)
    }

    #[inline]
    fn read_u128(bytes: [u8; 16]) -> u128 {
        u128::from_le_bytes(bytes)
    }

    #[inline]
    fn read_i16(bytes: [u8; 2]) -> i16 {
        i16::from_le_bytes(bytes)
    }

    #[inline]
    fn read_i32(bytes: [u8; 4]) -> i32 {
        i32::from_le_bytes(bytes)
    }

    #[inline]
    fn read_i64(bytes: [u8; 8]) -> i64 {
        i64::from_le_bytes(bytes)
    }

    #[inline]
    fn read_i128(bytes: [u8; 16]) -> i128 {
        i128::from_le_bytes(bytes)
    }

    #[inline]
    fn write_u8(value: u8) -> [u8; 1] {
        [value]
    }

    #[inline]
    fn write_u16(value: u16) -> [u8; 2] {
        u16::to_le_bytes(value)
    }

    #[inline]
    fn write_u32(value: u32) -> [u8; 4] {
        u32::to_le_bytes(value)
    }

    #[inline]
    fn write_u64(value: u64) -> [u8; 8] {
        u64::to_le_bytes(value)
    }

    #[inline]
    fn write_u128(value: u128) -> [u8; 16] {
        u128::to_le_bytes(value)
    }

    #[inline]
    fn write_i16(value: i16) -> [u8; 2] {
        i16::to_le_bytes(value)
    }

    #[inline]
    fn write_i32(value: i32) -> [u8; 4] {
        i32::to_le_bytes(value)
    }

    #[inline]
    fn write_i64(value: i64) -> [u8; 8] {
        i64::to_le_bytes(value)
    }

    #[inline]
    fn write_i128(value: i128) -> [u8; 16] {
        i128::to_le_bytes(value)
    }
}

/// Defines big-endian serialization.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BigEndian {}

impl ByteOrder for BigEndian {
    #[inline]
    fn read_u8(bytes: [u8; 1]) -> u8 {
        bytes[0]
    }

    #[inline]
    fn read_u16(bytes: [u8; 2]) -> u16 {
        u16::from_be_bytes(bytes)
    }

    #[inline]
    fn read_u32(bytes: [u8; 4]) -> u32 {
        u32::from_be_bytes(bytes)
    }

    #[inline]
    fn read_u64(bytes: [u8; 8]) -> u64 {
        u64::from_be_bytes(bytes)
    }

    #[inline]
    fn read_u128(bytes: [u8; 16]) -> u128 {
        u128::from_be_bytes(bytes)
    }

    #[inline]
    fn read_i16(bytes: [u8; 2]) -> i16 {
        i16::from_be_bytes(bytes)
    }

    #[inline]
    fn read_i32(bytes: [u8; 4]) -> i32 {
        i32::from_be_bytes(bytes)
    }

    #[inline]
    fn read_i64(bytes: [u8; 8]) -> i64 {
        i64::from_be_bytes(bytes)
    }

    #[inline]
    fn read_i128(bytes: [u8; 16]) -> i128 {
        i128::from_be_bytes(bytes)
    }

    #[inline]
    fn write_u8(value: u8) -> [u8; 1] {
        [value]
    }

    #[inline]
    fn write_u16(value: u16) -> [u8; 2] {
        u16::to_be_bytes(value)
    }

    #[inline]
    fn write_u32(value: u32) -> [u8; 4] {
        u32::to_be_bytes(value)
    }

    #[inline]
    fn write_u64(value: u64) -> [u8; 8] {
        u64::to_be_bytes(value)
    }

    #[inline]
    fn write_u128(value: u128) -> [u8; 16] {
        u128::to_be_bytes(value)
    }

    #[inline]
    fn write_i16(value: i16) -> [u8; 2] {
        i16::to_be_bytes(value)
    }

    #[inline]
    fn write_i32(value: i32) -> [u8; 4] {
        i32::to_be_bytes(value)
    }

    #[inline]
    fn write_i64(value: i64) -> [u8; 8] {
        i64::to_be_bytes(value)
    }

    #[inline]
    fn write_i128(value: i128) -> [u8; 16] {
        i128::to_be_bytes(value)
    }
}

/// Defines the network byte order, which is the same as [BigEndian].
pub type NetworkEndian = BigEndian;

/// Defines system native-endian byte order.
#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

/// Defines system native-endian byte order.
#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;
