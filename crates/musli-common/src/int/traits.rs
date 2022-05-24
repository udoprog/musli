use core::ops::{BitAnd, BitXor, Neg, Shl, Shr};

use crate::int::ByteOrder;
use crate::reader::Reader;
use crate::writer::Writer;

/// Trait that encodes common behaviors of unsigned numbers.
pub trait Unsigned:
    Sized
    + Copy
    + Shr<u32, Output = Self>
    + Shl<u32, Output = Self>
    + BitXor<Self, Output = Self>
    + BitAnd<Self, Output = Self>
    + PartialOrd<Self>
    + Ord
{
    /// The number `1` as represented by the current unsigned number.
    const ONE: Self;

    /// Number of bytes.
    const BYTES: u8;

    /// The signed representation of this unsigned number.
    type Signed: Signed;

    /// Coerce this number bitwise into its signed representation.
    fn signed(self) -> Self::Signed;

    /// Construct from the first byte.
    fn from_byte(byte: u8) -> Self;

    /// Coerce into the lowest 8-bits as a byte.
    fn as_byte(self) -> u8;

    /// Test if this value is smaller than the specified byte.
    fn is_smaller_than(self, byte: u8) -> bool;

    /// Test if value is zero.
    fn is_zero(self) -> bool;

    /// Perform a shift-right operation.
    fn checked_shr(self, value: u32) -> Option<Self>;

    /// Perform a shift-left operation.
    fn checked_shl(self, value: u32) -> Option<Self>;

    /// Perform a checked addition.
    fn checked_add(self, value: Self) -> Option<Self>;
}

/// Helper trait for performing I/O over [Unsigned] types.
pub trait ByteOrderIo: Unsigned {
    /// Write the current byte array to the given writer in little-endian
    /// encoding.
    fn write_bytes<W, B>(self, writer: W) -> Result<(), W::Error>
    where
        W: Writer,
        B: ByteOrder;

    /// Read the current value from the reader in little-endian encoding.
    fn read_bytes<'de, R, B>(reader: R) -> Result<Self, R::Error>
    where
        R: Reader<'de>,
        B: ByteOrder;
}

/// Trait that encodes common behaviors of signed numbers.
pub trait Signed:
    Sized
    + Copy
    + Neg<Output = Self>
    + Shr<u32, Output = Self>
    + Shl<u32, Output = Self>
    + BitXor<Self, Output = Self>
{
    /// The number of bits in this signed number.
    const BITS: u32;

    /// The unsigned representation of this number.
    type Unsigned: Unsigned;

    /// Coerce this number bitwise into its unsigned representation.
    fn unsigned(self) -> Self::Unsigned;
}

macro_rules! implement {
    ($signed:ty, $unsigned:ty) => {
        impl Signed for $signed {
            const BITS: u32 = <$signed>::BITS;

            type Unsigned = $unsigned;

            fn unsigned(self) -> Self::Unsigned {
                self as $unsigned
            }
        }

        impl Unsigned for $unsigned {
            const ONE: Self = 1;
            const BYTES: u8 = (<$unsigned>::BITS / 8) as u8;

            type Signed = $signed;

            #[inline]
            fn signed(self) -> Self::Signed {
                self as $signed
            }

            #[inline]
            fn from_byte(byte: u8) -> Self {
                byte as $unsigned
            }

            #[inline]
            fn as_byte(self) -> u8 {
                self as u8
            }

            #[inline]
            fn is_smaller_than(self, b: u8) -> bool {
                self < b as $unsigned
            }

            #[inline]
            fn is_zero(self) -> bool {
                self == 0
            }

            #[inline]
            fn checked_shr(self, value: u32) -> Option<Self> {
                self.checked_shr(value)
            }

            #[inline]
            fn checked_shl(self, value: u32) -> Option<Self> {
                self.checked_shl(value)
            }

            #[inline]
            fn checked_add(self, value: Self) -> Option<Self> {
                self.checked_add(value)
            }
        }
    };
}

macro_rules! implement_io {
    ($signed:ty, $unsigned:ty, $read:ident, $write:ident) => {
        implement!($signed, $unsigned);

        impl ByteOrderIo for $unsigned {
            #[inline]
            fn write_bytes<W, B>(self, mut writer: W) -> Result<(), W::Error>
            where
                W: Writer,
                B: ByteOrder,
            {
                writer.write_array(B::$write(self))
            }

            #[inline]
            fn read_bytes<'de, R, B>(mut reader: R) -> Result<Self, R::Error>
            where
                R: Reader<'de>,
                B: ByteOrder,
            {
                Ok(B::$read(reader.read_array()?))
            }
        }
    };
}

implement_io!(i8, u8, read_u8, write_u8);
implement_io!(i16, u16, read_u16, write_u16);
implement_io!(i32, u32, read_u32, write_u32);
implement_io!(i64, u64, read_u64, write_u64);
implement_io!(i128, u128, read_u128, write_u128);
implement!(isize, usize);
