use core::fmt::{Debug, Display};
use core::hash::Hash;

use musli::Context;

use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{ByteOrder, ByteOrderIo, Fixed, FixedUsize, Signed, Unsigned, Variable};
use crate::reader::Reader;
use crate::writer::Writer;

mod private {
    use crate::int::{ByteOrder, Unsigned};

    pub trait Sealed {}
    impl<B> Sealed for crate::int::Fixed<B> where B: ByteOrder {}
    impl Sealed for crate::int::Variable {}
    impl<L, B> Sealed for crate::int::FixedUsize<L, B>
    where
        L: Unsigned,
        B: ByteOrder,
    {
    }
}

/// Trait which governs how integers are encoded in a binary format.
///
/// The two common implementations of this is [Variable] and [Fixed].
pub trait IntegerEncoding:
    Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd + private::Sealed
{
    /// Governs how unsigned integers are encoded into a [Writer].
    fn encode_unsigned<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: ByteOrderIo;

    /// Governs how unsigned integers are decoded from a [Reader].
    fn decode_unsigned<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: ByteOrderIo;

    /// Governs how signed integers are encoded into a [Writer].
    fn encode_signed<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo;

    /// Governs how signed integers are decoded from a [Reader].
    fn decode_signed<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>;
}

/// Encoding formats which ensure that variably sized types (like `usize`,
/// `isize`) are encoded in a format which is platform-neutral.
pub trait UsizeEncoding: private::Sealed {
    /// Governs how usize lengths are encoded into a [Writer].
    fn encode_usize<'buf, C, W>(cx: &mut C, writer: W, value: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer;

    /// Governs how usize lengths are decoded from a [Reader].
    fn decode_usize<'de, 'buf, C, R>(cx: &mut C, reader: R) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>;
}

/// [IntegerEncoding] and [UsizeEncoding] implementation which encodes integers
/// using zigzag variable length encoding.
impl IntegerEncoding for Variable {
    #[inline]
    fn encode_unsigned<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Unsigned,
    {
        c::encode(cx, writer, value)
    }

    #[inline]
    fn decode_unsigned<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Unsigned,
    {
        c::decode(cx, reader)
    }

    #[inline]
    fn encode_signed<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Signed,
    {
        c::encode(cx, writer, zig::encode(value))
    }

    #[inline]
    fn decode_signed<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: Unsigned<Signed = T>,
    {
        let value: T::Unsigned = c::decode(cx, reader)?;
        Ok(zig::decode(value))
    }
}

impl UsizeEncoding for Variable {
    #[inline]
    fn encode_usize<'buf, C, W>(cx: &mut C, writer: W, value: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
    {
        c::encode(cx, writer, value)
    }

    #[inline]
    fn decode_usize<'de, 'buf, C, R>(cx: &mut C, reader: R) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
    {
        c::decode(cx, reader)
    }
}

impl<B> IntegerEncoding for Fixed<B>
where
    B: ByteOrder,
{
    #[inline]
    fn encode_unsigned<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: ByteOrderIo,
    {
        value.write_bytes_unsigned::<_, _, B>(cx, writer)
    }

    #[inline]
    fn decode_unsigned<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: ByteOrderIo,
    {
        T::read_bytes_unsigned::<_, _, B>(cx, reader)
    }

    #[inline]
    fn encode_signed<'buf, C, W, T>(cx: &mut C, writer: W, value: T) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
        T: Signed,
        T::Unsigned: ByteOrderIo,
    {
        value.unsigned().write_bytes_unsigned::<_, _, B>(cx, writer)
    }

    #[inline]
    fn decode_signed<'de, 'buf, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
        T: Signed,
        T::Unsigned: ByteOrderIo<Signed = T>,
    {
        Ok(T::Unsigned::read_bytes_unsigned::<_, _, B>(cx, reader)?.signed())
    }
}

impl<L, B> UsizeEncoding for FixedUsize<L, B>
where
    B: ByteOrder,
    usize: TryFrom<L>,
    L: ByteOrderIo,
    L: TryFrom<usize>,
    L::Error: 'static + Debug + Display + Send + Sync,
    <usize as TryFrom<L>>::Error: 'static + Debug + Display + Send + Sync,
{
    #[inline]
    fn encode_usize<'buf, C, W>(cx: &mut C, writer: W, value: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = W::Error>,
        W: Writer,
    {
        let value: L = value.try_into().map_err(|err| cx.custom(err))?;
        value.write_bytes_unsigned::<_, _, B>(cx, writer)
    }

    #[inline]
    fn decode_usize<'de, 'buf, C, R>(cx: &mut C, reader: R) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
        R: Reader<'de>,
    {
        usize::try_from(L::read_bytes_unsigned::<_, _, B>(cx, reader)?)
            .map_err(|error| cx.custom(error))
    }
}
