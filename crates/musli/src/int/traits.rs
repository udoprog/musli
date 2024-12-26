use core::ops::{BitAnd, BitXor, Neg, Shl, Shr};

use crate::options::ByteOrder;
use crate::{Context, Reader, Writer};

/// Trait that encodes common behaviors of unsigned numbers.
pub(crate) trait Unsigned:
    Copy
    + Shr<u32, Output = Self>
    + Shl<u32, Output = Self>
    + BitXor<Output = Self>
    + BitAnd<Output = Self>
    + Ord
{
    /// The number `1` as represented by the current unsigned number.
    const ONE: Self;

    /// Number of bytes.
    #[cfg(feature = "wire")]
    const BYTES: u8;

    /// Number of bits.
    const BITS: u32;

    /// The signed representation of this unsigned number.
    type Signed: Signed<Unsigned = Self>;

    /// Coerce this number bitwise into its signed representation.
    fn signed(self) -> Self::Signed;

    /// Construct from the first byte.
    fn from_byte(byte: u8) -> Self;

    /// Coerce into the lowest 8-bits as a byte.
    fn as_byte(self) -> u8;

    /// Test if this value is smaller than the specified byte.
    #[cfg(feature = "wire")]
    fn is_smaller_than(self, byte: u8) -> bool;

    /// Test if value is zero.
    fn is_zero(self) -> bool;

    /// Perform a wrapping shift-left operation.
    fn wrapping_shl(self, value: u32) -> Self;

    /// Perform a wrapping addition.
    fn wrapping_add(self, value: Self) -> Self;
}

/// Helper trait for performing I/O over [Unsigned] types.
pub(crate) trait UnsignedOps: Unsigned {
    /// Write the current byte array to the given writer in little-endian
    /// encoding.
    fn write_bytes<C, W>(self, cx: &C, writer: W, byte_order: ByteOrder) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
        W: Writer;

    /// Read the current value from the reader in little-endian encoding.
    fn read_bytes<'de, C, R>(cx: &C, reader: R, byte_order: ByteOrder) -> Result<Self, C::Error>
    where
        C: ?Sized + Context,
        R: Reader<'de>;
}

/// Trait that encodes common behaviors of signed numbers.
pub(crate) trait Signed:
    Copy
    + Neg<Output = Self>
    + Shr<u32, Output = Self>
    + Shl<u32, Output = Self>
    + BitXor<Output = Self>
{
    /// The number of bits in this signed number.
    const BITS: u32;

    /// The unsigned representation of this number.
    type Unsigned: Unsigned<Signed = Self>;

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
            #[cfg(feature = "wire")]
            const BYTES: u8 = (<$unsigned>::BITS / 8) as u8;
            const BITS: u32 = <$signed>::BITS;

            type Signed = $signed;

            #[inline(always)]
            fn signed(self) -> Self::Signed {
                self as $signed
            }

            #[inline(always)]
            fn from_byte(byte: u8) -> Self {
                byte as $unsigned
            }

            #[inline(always)]
            fn as_byte(self) -> u8 {
                self as u8
            }

            #[inline(always)]
            #[cfg(feature = "wire")]
            fn is_smaller_than(self, b: u8) -> bool {
                self < b as $unsigned
            }

            #[inline(always)]
            fn is_zero(self) -> bool {
                self == 0
            }

            #[inline(always)]
            fn wrapping_shl(self, value: u32) -> Self {
                <$unsigned>::wrapping_shl(self, value)
            }

            #[inline(always)]
            fn wrapping_add(self, value: Self) -> Self {
                <$unsigned>::wrapping_add(self, value)
            }
        }
    };
}

macro_rules! implement_ops {
    ($signed:ty, $unsigned:ty) => {
        implement!($signed, $unsigned);

        impl UnsignedOps for $unsigned {
            #[inline(always)]
            fn write_bytes<C, W>(
                self,
                cx: &C,
                mut writer: W,
                byte_order: ByteOrder,
            ) -> Result<(), C::Error>
            where
                C: ?Sized + Context,
                W: Writer,
            {
                let bytes = match byte_order {
                    ByteOrder::NATIVE => self,
                    _ => <$unsigned>::swap_bytes(self),
                };

                let bytes = <$unsigned>::to_ne_bytes(bytes);
                writer.write_bytes(cx, &bytes)
            }

            #[inline(always)]
            fn read_bytes<'de, C, R>(
                cx: &C,
                mut reader: R,
                byte_order: ByteOrder,
            ) -> Result<Self, C::Error>
            where
                C: ?Sized + Context,
                R: Reader<'de>,
            {
                let bytes = reader.read_array(cx)?;
                let bytes = <$unsigned>::from_ne_bytes(bytes);

                let bytes = match byte_order {
                    ByteOrder::NATIVE => bytes,
                    _ => <$unsigned>::swap_bytes(bytes),
                };

                Ok(bytes)
            }
        }
    };
}

implement_ops!(i8, u8);
implement_ops!(i16, u16);
implement_ops!(i32, u32);
implement_ops!(i64, u64);
implement_ops!(i128, u128);
implement!(isize, usize);
