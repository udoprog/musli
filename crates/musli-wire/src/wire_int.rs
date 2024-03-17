use musli::Context;

use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned, UnsignedOps};
use crate::options::Options;
use crate::reader::Reader;
use crate::tag::{Kind, Tag, DATA_MASK};
use crate::writer::Writer;

/// Governs how usize lengths are encoded into a [Writer].
#[inline]
pub(crate) fn encode_length<C, W, const F: Options>(
    cx: &C,
    mut writer: W,
    value: usize,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    match crate::options::length::<F>() {
        crate::options::Integer::Variable => {
            if value.is_smaller_than(DATA_MASK) {
                writer.write_byte(cx, Tag::new(Kind::Continuation, value.as_byte()).byte())
            } else {
                writer.write_byte(cx, Tag::empty(Kind::Continuation).byte())?;
                c::encode(cx, writer, value)
            }
        }
        _ => {
            let bo = crate::options::byteorder::<F>();
            let width = crate::options::length_width::<F>();
            let bytes = 1u8 << width as u8;
            writer.write_byte(cx, Tag::new(Kind::Prefix, bytes).byte())?;

            macro_rules! fixed {
                ($ty:ty) => {{
                    let Ok(value) = <$ty>::try_from(value) else {
                        return Err(cx.message("Numerical value out of bounds for usize"));
                    };

                    value.write_bytes(cx, writer, bo)
                }};
            }

            crate::width_arm!(width, fixed)
        }
    }
}

/// Governs how usize lengths are decoded from a [Reader].
#[inline]
pub(crate) fn decode_length<'de, C, R, const F: Options>(
    cx: &C,
    mut reader: R,
) -> Result<usize, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    match crate::options::length::<F>() {
        crate::options::Integer::Variable => {
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
        _ => {
            let bo = crate::options::byteorder::<F>();
            let width = crate::options::length_width::<F>();

            let bytes = 1u8 << width as u8;
            let tag = Tag::from_byte(reader.read_byte(cx)?);

            if tag != Tag::new(Kind::Prefix, bytes) {
                return Err(cx.message(format_args!(
                    "Expected fixed {} bytes prefix tag, but got {tag:?}",
                    bytes
                )));
            }

            macro_rules! fixed {
                ($ty:ty) => {{
                    let Ok(value) = usize::try_from(<$ty>::read_bytes(cx, reader, bo)?) else {
                        return Err(cx.message("Value type out of bounds for usize"));
                    };

                    Ok(value)
                }};
            }

            crate::width_arm!(width, fixed)
        }
    }
}

/// Governs how unsigned integers are encoded into a [Writer].
#[inline]
pub(crate) fn encode_unsigned<C, W, T, const F: Options>(
    cx: &C,
    mut writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: UnsignedOps,
{
    match crate::options::integer::<F>() {
        crate::options::Integer::Variable => {
            if value.is_smaller_than(DATA_MASK) {
                writer.write_byte(cx, Tag::new(Kind::Continuation, value.as_byte()).byte())
            } else {
                writer.write_byte(cx, Tag::empty(Kind::Continuation).byte())?;
                c::encode(cx, writer, value)
            }
        }
        _ => {
            let bo = crate::options::byteorder::<F>();
            writer.write_byte(cx, Tag::new(Kind::Prefix, T::BYTES).byte())?;
            value.write_bytes(cx, writer, bo)
        }
    }
}

/// Governs how unsigned integers are decoded from a [Reader].
#[inline]
pub(crate) fn decode_unsigned<'de, C, R, T, const F: Options>(
    cx: &C,
    mut reader: R,
) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: UnsignedOps,
{
    match crate::options::integer::<F>() {
        crate::options::Integer::Variable => {
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
        _ => {
            let bo = crate::options::byteorder::<F>();

            if Tag::from_byte(reader.read_byte(cx)?) != Tag::new(Kind::Prefix, T::BYTES) {
                return Err(cx.message("Expected fixed integer"));
            }

            T::read_bytes(cx, reader, bo)
        }
    }
}

/// Governs how signed integers are encoded into a [Writer].
#[inline]
pub(crate) fn encode_signed<C, W, T, const F: Options>(
    cx: &C,
    writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    let value = zig::encode(value);
    encode_unsigned::<C, W, T::Unsigned, F>(cx, writer, value)
}

/// Governs how signed integers are decoded from a [Reader].
#[inline]
pub(crate) fn decode_signed<'de, C, R, T, const F: Options>(
    cx: &C,
    reader: R,
) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    let value = decode_unsigned::<C, R, T::Unsigned, F>(cx, reader)?;
    Ok(zig::decode(value))
}
