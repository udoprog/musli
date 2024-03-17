use musli::Context;

use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned, UnsignedOps};
use crate::options::Options;
use crate::reader::Reader;
use crate::writer::Writer;

/// Governs how unsigned integers are encoded into a [Writer].
#[inline]
pub fn encode_unsigned<C, W, T, const F: Options>(
    cx: &C,
    writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: Unsigned + UnsignedOps,
{
    match crate::options::integer::<F>() {
        crate::options::Integer::Variable => c::encode(cx, writer, value),
        _ => {
            let bo = crate::options::byteorder::<F>();
            value.write_bytes(cx, writer, bo)
        }
    }
}

/// Decode an unsigned value from the specified reader using the configuration
/// passed in through `F`.
#[inline]
pub fn decode_unsigned<'de, C, R, T: UnsignedOps, const F: Options>(
    cx: &C,
    reader: R,
) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: Unsigned,
{
    match crate::options::integer::<F>() {
        crate::options::Integer::Variable => c::decode(cx, reader),
        _ => {
            let bo = crate::options::byteorder::<F>();
            T::read_bytes(cx, reader, bo)
        }
    }
}

/// Governs how signed integers are encoded into a [Writer].
#[inline]
pub fn encode_signed<C, W, T, const F: Options>(cx: &C, writer: W, value: T) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    match crate::options::integer::<F>() {
        crate::options::Integer::Variable => c::encode(cx, writer, zig::encode(value)),
        _ => {
            let bo = crate::options::byteorder::<F>();
            value.unsigned().write_bytes(cx, writer, bo)
        }
    }
}

/// Governs how signed integers are decoded from a [Reader].
#[inline]
pub fn decode_signed<'de, C, R, T, const F: Options>(cx: &C, reader: R) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    match crate::options::integer::<F>() {
        crate::options::Integer::Variable => {
            let value: T::Unsigned = c::decode(cx, reader)?;
            Ok(zig::decode(value))
        }
        _ => {
            let bo = crate::options::byteorder::<F>();
            Ok(T::Unsigned::read_bytes(cx, reader, bo)?.signed())
        }
    }
}

/// Governs how usize lengths are encoded into a [Writer].
#[inline]
pub fn encode_usize<C, W, const F: Options>(cx: &C, writer: W, value: usize) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    match crate::options::length::<F>() {
        crate::options::Integer::Variable => c::encode(cx, writer, value),
        _ => {
            let bo = crate::options::byteorder::<F>();
            macro_rules! fixed {
                ($ty:ty) => {{
                    let Ok(value) = <$ty>::try_from(value) else {
                        return Err(cx.message("Size type out of bounds for value type"));
                    };

                    <$ty as UnsignedOps>::write_bytes(value, cx, writer, bo)
                }};
            }

            crate::width_arm!(crate::options::length_width::<F>(), fixed)
        }
    }
}

/// Governs how usize lengths are decoded from a [Reader].
#[inline]
pub fn decode_usize<'de, C, R, const F: Options>(cx: &C, reader: R) -> Result<usize, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    match crate::options::length::<F>() {
        crate::options::Integer::Variable => c::decode(cx, reader),
        _ => {
            let bo = crate::options::byteorder::<F>();

            macro_rules! fixed {
                ($ty:ty) => {{
                    let Ok(value) =
                        usize::try_from(<$ty as UnsignedOps>::read_bytes(cx, reader, bo)?)
                    else {
                        return Err(cx.message("Value type out of bounds for usize"));
                    };

                    Ok(value)
                }};
            }

            crate::width_arm!(crate::options::length_width::<F>(), fixed)
        }
    }
}
