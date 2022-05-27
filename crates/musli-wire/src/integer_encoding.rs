use core::fmt::{Debug, Display};

use crate::tag::{Kind, Tag, DATA_MASK};
use musli::error::Error;
use musli_common::int::continuation as c;
use musli_common::int::zigzag as zig;
use musli_common::int::{ByteOrder, ByteOrderIo, Fixed, FixedUsize, Signed, Unsigned, Variable};
use musli_common::reader::Reader;
use musli_common::writer::Writer;

mod private {
    pub trait Sealed {}
    impl<B> Sealed for musli_common::int::Fixed<B> {}
    impl<L, B> Sealed for musli_common::int::FixedUsize<L, B> {}
    impl Sealed for musli_common::int::Variable {}
}

/// Trait which governs how integers are encoded in a binary format.
///
/// The two common implementations of this is [Variable] and [Fixed].
pub trait WireIntegerEncoding: musli_common::int::IntegerEncoding + private::Sealed {
    /// Governs how unsigned integers are encoded into a [Writer].
    fn encode_typed_unsigned<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ByteOrderIo;

    /// Governs how unsigned integers are decoded from a [Reader].
    fn decode_typed_unsigned<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo;

    /// Governs how signed integers are encoded into a [Writer].
    fn encode_typed_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo;

    /// Governs how signed integers are decoded from a [Reader].
    fn decode_typed_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>;
}

/// Encoding formats which ensure that variably sized types (like `usize`,
/// `isize`) are encoded in a format which is platform-neutral.
pub trait WireUsizeEncoding: musli_common::int::UsizeEncoding + private::Sealed {
    /// Governs how usize lengths are encoded into a [Writer].
    fn encode_typed_usize<W>(writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer;

    /// Governs how usize lengths are decoded from a [Reader].
    fn decode_typed_usize<'de, R>(reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>;
}

impl WireIntegerEncoding for Variable {
    #[inline]
    fn encode_typed_unsigned<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
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
    fn decode_typed_unsigned<'de, R, T>(mut reader: R) -> Result<T, R::Error>
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
    fn encode_typed_signed<W, T>(writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        Self::encode_typed_unsigned(writer, zig::encode(value))
    }

    #[inline]
    fn decode_typed_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: Unsigned<Signed = T> + ByteOrderIo,
    {
        let value: T::Unsigned = Self::decode_typed_unsigned(reader)?;
        Ok(zig::decode(value))
    }
}

impl WireUsizeEncoding for Variable {
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

impl<B> WireIntegerEncoding for Fixed<B>
where
    B: ByteOrder,
{
    #[inline]
    fn encode_typed_unsigned<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ByteOrderIo,
    {
        writer.write_byte(Tag::new(Kind::Prefix, T::BYTES).byte())?;
        value.write_bytes_unsigned::<_, B>(writer)
    }

    #[inline]
    fn decode_typed_unsigned<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: ByteOrderIo,
    {
        if Tag::from_byte(reader.read_byte()?) != Tag::new(Kind::Prefix, T::BYTES) {
            return Err(R::Error::custom("expected fixed integer"));
        }

        T::read_bytes_unsigned::<_, B>(reader)
    }

    #[inline]
    fn encode_typed_signed<W, T>(mut writer: W, value: T) -> Result<(), W::Error>
    where
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        writer.write_byte(Tag::new(Kind::Prefix, T::Unsigned::BYTES).byte())?;
        value.unsigned().write_bytes_unsigned::<_, B>(writer)
    }

    #[inline]
    fn decode_typed_signed<'de, R, T>(mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>,
    {
        if Tag::from_byte(reader.read_byte()?) != Tag::new(Kind::Prefix, T::Unsigned::BYTES) {
            return Err(R::Error::custom("expected fixed integer"));
        }

        Ok(T::Unsigned::read_bytes_unsigned::<_, B>(reader)?.signed())
    }
}

impl<L, B> WireUsizeEncoding for FixedUsize<L, B>
where
    B: ByteOrder,
    usize: TryFrom<L>,
    L: ByteOrderIo + TryFrom<usize>,
    L::Error: 'static + Debug + Display + Send + Sync,
    <usize as TryFrom<L>>::Error: 'static + Debug + Display + Send + Sync,
{
    #[inline]
    fn encode_typed_usize<W>(mut writer: W, value: usize) -> Result<(), W::Error>
    where
        W: Writer,
    {
        writer.write_byte(Tag::new(Kind::Prefix, L::BYTES).byte())?;
        let value: L = value.try_into().map_err(W::Error::custom)?;
        value.write_bytes_unsigned::<_, B>(writer)
    }

    #[inline]
    fn decode_typed_usize<'de, R>(mut reader: R) -> Result<usize, R::Error>
    where
        R: Reader<'de>,
    {
        if Tag::from_byte(reader.read_byte()?) != Tag::new(Kind::Prefix, L::BYTES) {
            return Err(R::Error::custom("expected fixed integer"));
        }

        usize::try_from(L::read_bytes_unsigned::<_, B>(reader)?).map_err(R::Error::custom)
    }
}
