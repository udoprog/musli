use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::marker;

use crate::traits::Typed;
use crate::types::TypeTag;
use musli::error::Error;
use musli_binary_common::int::continuation as c;
use musli_binary_common::int::zigzag as zig;
use musli_binary_common::int::{ByteOrder, ByteOrderIo, NetworkEndian, Signed, Unsigned};
use musli_binary_common::reader::Reader;
use musli_binary_common::writer::Writer;

mod private {
    pub trait Sealed {}
    impl<B> Sealed for super::Fixed<B> {}
    impl Sealed for super::Variable {}
}

/// Trait which governs how integers are encoded in a binary format.
///
/// The two common implementations of this is [Variable] and [Fixed].
pub trait IntegerEncoding:
    Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd + private::Sealed
{
    /// Governs how unsigned integers are encoded into a [Writer].
    fn encode_unsigned<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ByteOrderIo + Typed;

    /// Governs how unsigned integers are decoded from a [Reader].
    fn decode_unsigned<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo + Typed;

    /// Governs how signed integers are encoded into a [Writer].
    fn encode_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo + Typed;

    /// Governs how signed integers are decoded from a [Reader].
    fn decode_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T> + Typed;
}

/// Encoding formats which ensure that variably sized types (like `usize`,
/// `isize`) are encoded in a format which is platform-neutral.
pub trait UsizeEncoding {
    /// Governs how usize lengths are encoded into a [Writer].
    fn encode_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer;

    /// Governs how usize lengths are encoded into a [Writer].
    fn encode_typed_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer;

    /// Governs how usize lengths are decoded from a [Reader].
    fn decode_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>;

    /// Governs how usize lengths are decoded from a [Reader].
    fn decode_typed_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>;
}

/// Type that indicates that the given numerical type should use variable-length
/// encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Variable {}

impl IntegerEncoding for Variable {
    #[inline]
    fn encode_unsigned<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Unsigned,
    {
        writer.write_byte(TypeTag::Continuation as u8)?;
        c::encode(writer, value)
    }

    #[inline]
    fn decode_unsigned<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Unsigned,
    {
        if reader.read_byte()? != TypeTag::Continuation as u8 {
            return Err(R::Error::custom("expected Continuation"));
        }

        c::decode(reader)
    }

    #[inline]
    fn encode_signed<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
    {
        writer.write_byte(TypeTag::Continuation as u8)?;
        c::encode(writer, zig::encode(value))
    }

    #[inline]
    fn decode_signed<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: Unsigned<Signed = T>,
    {
        if reader.read_byte()? != TypeTag::Continuation as u8 {
            return Err(R::Error::custom("expected Continuation"));
        }

        let value: T::Unsigned = c::decode(reader)?;
        Ok(zig::decode(value))
    }
}

impl UsizeEncoding for Variable {
    #[inline]
    fn encode_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer,
    {
        c::encode(writer, value)
    }

    #[inline]
    fn encode_typed_usize<W>(mut writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer,
    {
        writer.write_byte(TypeTag::Continuation as u8)?;
        c::encode(writer, value)
    }

    #[inline]
    fn decode_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>,
    {
        c::decode(reader)
    }

    #[inline]
    fn decode_typed_usize<'de, R>(mut reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>,
    {
        if reader.read_byte()? != TypeTag::Continuation as u8 {
            return Err(R::Error::custom("expected Continuation"));
        }

        c::decode(reader)
    }
}

/// A fixed-length integer encoding which encodes something to a little-endian
/// encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct Fixed<B = NetworkEndian> {
    _marker: marker::PhantomData<B>,
}

impl<B> IntegerEncoding for Fixed<B>
where
    B: ByteOrder,
{
    #[inline]
    fn encode_unsigned<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ByteOrderIo + Typed,
    {
        writer.write_byte(T::TYPE_FLAG as u8)?;
        value.write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_unsigned<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo + Typed,
    {
        if reader.read_byte()? != T::TYPE_FLAG as u8 {
            return Err(R::Error::custom("expected fixed integer"));
        }

        T::read_bytes::<_, B>(reader)
    }

    #[inline]
    fn encode_signed<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo + Typed,
    {
        writer.write_byte(T::Unsigned::TYPE_FLAG as u8)?;
        value.unsigned().write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_signed<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T> + Typed,
    {
        if reader.read_byte()? != T::Unsigned::TYPE_FLAG as u8 {
            return Err(R::Error::custom("expected fixed integer"));
        }

        Ok(T::Unsigned::read_bytes::<_, B>(reader)?.signed())
    }
}

/// A fixed-length encoding which encodes numbers to the width of `L` and the
/// endianness of `B`.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct FixedLength<L = u32, B = NetworkEndian>
where
    L: Unsigned,
    B: ByteOrder,
{
    _marker: marker::PhantomData<(L, B)>,
}

impl<L, B> UsizeEncoding for FixedLength<L, B>
where
    B: ByteOrder,
    usize: TryFrom<L>,
    L: ByteOrderIo + Typed + TryFrom<usize>,
    L::Error: 'static + Debug + Display + Send + Sync,
    <usize as TryFrom<L>>::Error: 'static + Debug + Display + Send + Sync,
{
    #[inline]
    fn encode_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer,
    {
        let value: L = value.try_into().map_err(W::Error::custom)?;
        value.write_bytes::<_, B>(writer)
    }

    #[inline]
    fn encode_typed_usize<W>(mut writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer,
    {
        writer.write_byte(L::TYPE_FLAG as u8)?;
        let value: L = value.try_into().map_err(W::Error::custom)?;
        value.write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>,
    {
        usize::try_from(L::read_bytes::<_, B>(reader)?).map_err(R::Error::custom)
    }

    #[inline]
    fn decode_typed_usize<'de, R>(mut reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>,
    {
        if reader.read_byte()? != L::TYPE_FLAG as u8 {
            return Err(R::Error::custom("expected fixed integer"));
        }

        usize::try_from(L::read_bytes::<_, B>(reader)?).map_err(R::Error::custom)
    }
}
