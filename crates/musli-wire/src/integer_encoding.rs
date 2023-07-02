use musli::Context;
use musli_common::int::continuation as c;
use musli_common::int::zigzag as zig;
use musli_common::int::{ByteOrder, ByteOrderIo, Fixed, FixedUsize, Signed, Unsigned, Variable};
use musli_common::reader::Reader;
use musli_common::writer::Writer;

use crate::tag::{Kind, Tag, DATA_MASK};

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
    fn encode_typed_unsigned<'buf, C, W, T>(
        cx: &mut C,
        writer: W,
        value: T,
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: ByteOrderIo;

    /// Governs how unsigned integers are decoded from a [Reader].
    fn decode_typed_unsigned<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: ByteOrderIo;

    /// Governs how signed integers are encoded into a [Writer].
    fn encode_typed_signed<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo;

    /// Governs how signed integers are decoded from a [Reader].
    fn decode_typed_signed<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>;
}

/// Encoding formats which ensure that variably sized types (like `usize`,
/// `isize`) are encoded in a format which is platform-neutral.
pub trait WireUsizeEncoding: musli_common::int::UsizeEncoding + private::Sealed {
    /// Governs how usize lengths are encoded into a [Writer].
    fn encode_typed_usize<'buf, C, W>(cx: &mut C, writer: W, value: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer;

    /// Governs how usize lengths are decoded from a [Reader].
    fn decode_typed_usize<'de, 'buf, C, R>(cx: &mut C, reader: R) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>;
}

impl WireIntegerEncoding for Variable {
    #[inline]
    fn encode_typed_unsigned<'buf, C, W, T>(
        cx: &mut C,
        mut writer: W,
        value: T,
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Unsigned,
    {
        if value.is_smaller_than(DATA_MASK) {
            writer.write_byte(cx, Tag::new(Kind::Continuation, value.as_byte()).byte())
        } else {
            writer.write_byte(cx, Tag::empty(Kind::Continuation).byte())?;
            c::encode(cx, writer, value)
        }
    }

    #[inline]
    fn decode_typed_unsigned<'de, 'buf, C, R, T>(cx: &mut C, mut reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Unsigned,
    {
        let tag = Tag::from_byte(reader.read_byte(cx)?);

        if tag.kind() != Kind::Continuation {
            return Err(cx.message("expected continuation"));
        }

        if let Some(data) = tag.data() {
            Ok(T::from_byte(data))
        } else {
            c::decode(cx, reader)
        }
    }

    #[inline]
    fn encode_typed_signed<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        Self::encode_typed_unsigned(cx, writer, zig::encode(value))
    }

    #[inline]
    fn decode_typed_signed<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: Unsigned<Signed = T> + ByteOrderIo,
    {
        let value: T::Unsigned = Self::decode_typed_unsigned(cx, reader)?;
        Ok(zig::decode(value))
    }
}

impl WireUsizeEncoding for Variable {
    #[inline]
    fn encode_typed_usize<'buf, C, W>(
        cx: &mut C,
        mut writer: W,
        value: usize,
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
    {
        if value.is_smaller_than(DATA_MASK) {
            writer.write_byte(cx, Tag::new(Kind::Continuation, value.as_byte()).byte())
        } else {
            writer.write_byte(cx, Tag::empty(Kind::Continuation).byte())?;
            c::encode(cx, writer, value)
        }
    }

    #[inline]
    fn decode_typed_usize<'de, 'buf, C, R>(cx: &mut C, mut reader: R) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
    {
        let tag = Tag::from_byte(reader.read_byte(cx)?);

        if tag.kind() != Kind::Continuation {
            return Err(cx.message("expected continuation"));
        }

        if let Some(data) = tag.data() {
            Ok(usize::from_byte(data))
        } else {
            c::decode(cx, reader)
        }
    }
}

impl<B> WireIntegerEncoding for Fixed<B>
where
    B: ByteOrder,
{
    #[inline]
    fn encode_typed_unsigned<'buf, C, W, T>(
        cx: &mut C,
        mut writer: W,
        value: T,
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: ByteOrderIo,
    {
        writer.write_byte(cx, Tag::new(Kind::Prefix, T::BYTES).byte())?;
        value.write_bytes_unsigned::<_, _, B>(cx, writer)
    }

    #[inline]
    fn decode_typed_unsigned<'de, 'buf, C, R, T>(cx: &mut C, mut reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: ByteOrderIo,
    {
        if Tag::from_byte(reader.read_byte(cx)?) != Tag::new(Kind::Prefix, T::BYTES) {
            return Err(cx.message("expected fixed integer"));
        }

        T::read_bytes_unsigned::<_, _, B>(cx, reader)
    }

    #[inline]
    fn encode_typed_signed<'buf, C, W, T>(
        cx: &mut C,
        mut writer: W,
        value: T,
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        writer.write_byte(cx, Tag::new(Kind::Prefix, T::Unsigned::BYTES).byte())?;
        value.unsigned().write_bytes_unsigned::<_, _, B>(cx, writer)
    }

    #[inline]
    fn decode_typed_signed<'de, 'buf, C, R, T>(cx: &mut C, mut reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>,
    {
        if Tag::from_byte(reader.read_byte(cx)?) != Tag::new(Kind::Prefix, T::Unsigned::BYTES) {
            return Err(cx.message("expected fixed integer"));
        }

        Ok(T::Unsigned::read_bytes_unsigned::<_, _, B>(cx, reader)?.signed())
    }
}

impl<L, B> WireUsizeEncoding for FixedUsize<L, B>
where
    B: ByteOrder,
    usize: TryFrom<L>,
    L: ByteOrderIo + TryFrom<usize>,
{
    #[inline]
    fn encode_typed_usize<'buf, C, W>(
        cx: &mut C,
        mut writer: W,
        value: usize,
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
    {
        writer.write_byte(cx, Tag::new(Kind::Prefix, L::BYTES).byte())?;

        let Ok(value) = L::try_from(value) else {
            return Err(cx.message("usize out of bounds for value type"));
        };

        value.write_bytes_unsigned::<_, _, B>(cx, writer)
    }

    #[inline]
    fn decode_typed_usize<'de, 'buf, C, R>(cx: &mut C, mut reader: R) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
    {
        let tag = Tag::from_byte(reader.read_byte(cx)?);

        if tag != Tag::new(Kind::Prefix, L::BYTES) {
            return Err(cx.message(format_args!(
                "expected fixed {} bytes prefix tag, but got {tag:?}",
                L::BYTES
            )));
        }

        let Ok(value) = usize::try_from(L::read_bytes_unsigned::<_, _, B>(cx, reader)?) else {
            return Err(cx.message("value type out of bounds for usize"));
        };

        Ok(value)
    }
}
