use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::marker;

use crate::types::{Kind, Tag, DATA_MASK};
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
        T: ByteOrderIo;

    /// Governs how unsigned integers are decoded from a [Reader].
    fn decode_unsigned<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo;

    /// Governs how signed integers are encoded into a [Writer].
    fn encode_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo;

    /// Governs how signed integers are decoded from a [Reader].
    fn decode_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>;
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
        if value.is_smaller_than(DATA_MASK) {
            writer.write_byte(Tag::new(Kind::Continuation, value.as_byte()).byte())
        } else {
            writer.write_byte(Tag::empty(Kind::Continuation).byte())?;
            c::encode(writer, value)
        }
    }

    #[inline]
    fn decode_unsigned<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Unsigned,
    {
        let tag = Tag::from_byte(reader.read_byte()?);

        if tag.kind() != Kind::Continuation {
            return Err(R::Error::custom("Expected Continuation"));
        }

        if let Some(data) = tag.data() {
            Ok(T::from_byte(data))
        } else {
            c::decode(reader)
        }
    }

    #[inline]
    fn encode_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        Self::encode_unsigned(writer, zig::encode(value))
    }

    #[inline]
    fn decode_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: Unsigned<Signed = T> + ByteOrderIo,
    {
        let value: T::Unsigned = Self::decode_unsigned(reader)?;
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
        if value.is_smaller_than(DATA_MASK) {
            writer.write_byte(Tag::new(Kind::Continuation, value.as_byte()).byte())
        } else {
            writer.write_byte(Tag::empty(Kind::Continuation).byte())?;
            c::encode(writer, value)
        }
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
        let tag = Tag::from_byte(reader.read_byte()?);

        if tag.kind() != Kind::Continuation {
            return Err(R::Error::custom("Expected Continuation"));
        }

        if let Some(data) = tag.data() {
            Ok(usize::from_byte(data))
        } else {
            c::decode(reader)
        }
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
        T: ByteOrderIo,
    {
        writer.write_byte(Tag::new(Kind::Prefix, T::BYTES).byte())?;
        value.write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_unsigned<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo,
    {
        if Tag::from_byte(reader.read_byte()?) != Tag::new(Kind::Prefix, T::BYTES) {
            return Err(R::Error::custom("expected fixed integer"));
        }

        T::read_bytes::<_, B>(reader)
    }

    #[inline]
    fn encode_signed<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        writer.write_byte(Tag::new(Kind::Prefix, T::Unsigned::BYTES).byte())?;
        value.unsigned().write_bytes::<_, B>(writer)
    }

    #[inline]
    fn decode_signed<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>,
    {
        if Tag::from_byte(reader.read_byte()?) != Tag::new(Kind::Prefix, T::Unsigned::BYTES) {
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
    L: ByteOrderIo + TryFrom<usize>,
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
        writer.write_byte(Tag::new(Kind::Prefix, L::BYTES).byte())?;
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
        if Tag::from_byte(reader.read_byte()?) != Tag::new(Kind::Prefix, L::BYTES) {
            return Err(R::Error::custom("expected fixed integer"));
        }

        usize::try_from(L::read_bytes::<_, B>(reader)?).map_err(R::Error::custom)
    }
}
