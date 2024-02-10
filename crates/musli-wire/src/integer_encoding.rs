use musli::Context;

use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{BigEndian, LittleEndian, Signed, Unsigned, UnsignedOps};
use crate::options::Options;
use crate::reader::Reader;
use crate::tag::{Kind, Tag, DATA_MASK};
use crate::writer::Writer;

macro_rules! fixed_arm {
    ($bo:ty, $what:ident::<$f:ty>, $macro:path) => {
        match crate::options::$what::<$f>() {
            crate::options::Width::U8 => {
                $macro!(u8, $bo)
            }
            crate::options::Width::U16 => {
                $macro!(u16, $bo)
            }
            crate::options::Width::U32 => {
                $macro!(u32, $bo)
            }
            _ => {
                $macro!(u64, $bo)
            }
        }
    };
}

/// Governs how usize lengths are encoded into a [Writer].
#[inline]
pub(crate) fn encode_typed_usize<C, W, const F: Options>(
    cx: &mut C,
    mut writer: W,
    value: usize,
) -> Result<(), C::Error>
where
    C: Context<Input = W::Error>,
    W: Writer,
{
    macro_rules! fixed {
        ($ty:ty, $bo:ty) => {{
            writer.write_byte(cx, Tag::new(Kind::Prefix, <$ty>::BYTES).byte())?;

            let Ok(value) = <$ty>::try_from(value) else {
                return Err(cx.message("Usize out of bounds for value type"));
            };

            value.write_bytes::<_, _, $bo>(cx, writer)
        }};
    }

    match (
        crate::options::length::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => {
            if value.is_smaller_than(DATA_MASK) {
                writer.write_byte(cx, Tag::new(Kind::Continuation, value.as_byte()).byte())
            } else {
                writer.write_byte(cx, Tag::empty(Kind::Continuation).byte())?;
                c::encode(cx, writer, value)
            }
        }
        (crate::options::Integer::Fixed, crate::options::ByteOrder::LittleEndian) => {
            fixed_arm!(LittleEndian, length_width::<F>, fixed)
        }
        _ => {
            fixed_arm!(BigEndian, length_width::<F>, fixed)
        }
    }
}

/// Governs how usize lengths are decoded from a [Reader].
#[inline]
pub(crate) fn decode_typed_usize<'de, C, R, const F: Options>(
    cx: &mut C,
    mut reader: R,
) -> Result<usize, C::Error>
where
    C: Context<Input = R::Error>,
    R: Reader<'de>,
{
    macro_rules! fixed {
        ($ty:ty, $bo:ty) => {{
            let tag = Tag::from_byte(reader.read_byte(cx)?);

            if tag != Tag::new(Kind::Prefix, <$ty>::BYTES) {
                return Err(cx.message(format_args!(
                    "Expected fixed {} bytes prefix tag, but got {tag:?}",
                    <$ty>::BYTES
                )));
            }

            let Ok(value) = usize::try_from(<$ty>::read_bytes_unsigned::<_, _, $bo>(cx, reader)?)
            else {
                return Err(cx.message("Value type out of bounds for usize"));
            };

            Ok(value)
        }};
    }

    match (
        crate::options::length::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => {
            let tag = Tag::from_byte(reader.read_byte(cx)?);

            if tag.kind() != Kind::Continuation {
                return Err(cx.message("Expected continuation"));
            }

            if let Some(data) = tag.data() {
                Ok(usize::from_byte(data))
            } else {
                c::decode(cx, reader)
            }
        }
        (crate::options::Integer::Fixed, crate::options::ByteOrder::LittleEndian) => {
            fixed_arm!(LittleEndian, length_width::<F>, fixed)
        }
        _ => {
            fixed_arm!(BigEndian, length_width::<F>, fixed)
        }
    }
}

/// Governs how unsigned integers are encoded into a [Writer].
#[inline]
pub(crate) fn encode_typed_unsigned<C, W, T, const F: Options>(
    cx: &mut C,
    mut writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: Context<Input = W::Error>,
    W: Writer,
    T: UnsignedOps,
{
    macro_rules! fixed {
        ($ty:ty, $bo:ty) => {{
            writer.write_byte(cx, Tag::new(Kind::Prefix, <$ty>::BYTES).byte())?;
            value.write_bytes::<_, _, $bo>(cx, writer)
        }};
    }

    match (
        crate::options::integer::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => {
            if value.is_smaller_than(DATA_MASK) {
                writer.write_byte(cx, Tag::new(Kind::Continuation, value.as_byte()).byte())
            } else {
                writer.write_byte(cx, Tag::empty(Kind::Continuation).byte())?;
                c::encode(cx, writer, value)
            }
        }
        (crate::options::Integer::Fixed, crate::options::ByteOrder::LittleEndian) => {
            fixed!(T, LittleEndian)
        }
        _ => {
            fixed!(T, BigEndian)
        }
    }
}

/// Governs how unsigned integers are decoded from a [Reader].
#[inline]
pub(crate) fn decode_typed_unsigned<'de, C, R, T, const F: Options>(
    cx: &mut C,
    mut reader: R,
) -> Result<T, C::Error>
where
    C: Context<Input = R::Error>,
    R: Reader<'de>,
    T: UnsignedOps,
{
    macro_rules! fixed {
        ($ty:ty, $bo:ty) => {{
            if Tag::from_byte(reader.read_byte(cx)?) != Tag::new(Kind::Prefix, <$ty>::BYTES) {
                return Err(cx.message("Expected fixed integer"));
            }

            <$ty as UnsignedOps>::read_bytes_unsigned::<_, _, $bo>(cx, reader)
        }};
    }

    match (
        crate::options::integer::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => {
            let tag = Tag::from_byte(reader.read_byte(cx)?);

            if tag.kind() != Kind::Continuation {
                return Err(cx.message("Expected continuation"));
            }

            if let Some(data) = tag.data() {
                Ok(T::from_byte(data))
            } else {
                c::decode(cx, reader)
            }
        }
        (crate::options::Integer::Fixed, crate::options::ByteOrder::LittleEndian) => {
            fixed!(T, LittleEndian)
        }
        _ => {
            fixed!(T, BigEndian)
        }
    }
}

/// Governs how signed integers are encoded into a [Writer].
#[inline]
pub(crate) fn encode_typed_signed<C, W, T, const F: Options>(
    cx: &mut C,
    writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: Context<Input = W::Error>,
    W: Writer,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    let value = zig::encode(value);
    encode_typed_unsigned::<_, _, _, F>(cx, writer, value)
}

/// Governs how signed integers are decoded from a [Reader].
#[inline]
pub(crate) fn decode_typed_signed<'de, C, R, T, const F: Options>(
    cx: &mut C,
    reader: R,
) -> Result<T, C::Error>
where
    C: Context<Input = R::Error>,
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    let value: T::Unsigned = decode_typed_unsigned::<_, _, _, F>(cx, reader)?;
    Ok(zig::decode(value))
}
