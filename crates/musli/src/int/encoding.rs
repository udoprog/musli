use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned, UnsignedOps};
use crate::{Context, Options, Reader, Writer};

/// Governs how unsigned integers are encoded into a [`Writer`].
#[inline]
pub fn encode_unsigned<C, W, T, const OPT: Options>(
    cx: &C,
    writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: Unsigned + UnsignedOps,
{
    match crate::options::integer::<OPT>() {
        crate::options::Integer::Variable => c::encode(cx, writer, value),
        _ => {
            let bo = crate::options::byteorder::<OPT>();
            value.write_bytes(cx, writer, bo)
        }
    }
}

/// Decode an unsigned value from the specified reader using the configuration
/// passed in through `F`.
#[inline]
pub fn decode_unsigned<'de, C, R, T, const OPT: Options>(cx: &C, reader: R) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: UnsignedOps,
{
    match crate::options::integer::<OPT>() {
        crate::options::Integer::Variable => c::decode(cx, reader),
        _ => {
            let bo = crate::options::byteorder::<OPT>();
            T::read_bytes(cx, reader, bo)
        }
    }
}

/// Governs how signed integers are encoded into a [`Writer`].
#[inline]
pub fn encode_signed<C, W, T, const OPT: Options>(
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
    match crate::options::integer::<OPT>() {
        crate::options::Integer::Variable => c::encode(cx, writer, zig::encode(value)),
        _ => {
            let bo = crate::options::byteorder::<OPT>();
            value.unsigned().write_bytes(cx, writer, bo)
        }
    }
}

/// Governs how signed integers are decoded from a [`Reader`].
#[inline]
pub fn decode_signed<'de, C, R, T, const OPT: Options>(cx: &C, reader: R) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    match crate::options::integer::<OPT>() {
        crate::options::Integer::Variable => {
            let value: T::Unsigned = c::decode(cx, reader)?;
            Ok(zig::decode(value))
        }
        _ => {
            let bo = crate::options::byteorder::<OPT>();
            Ok(T::Unsigned::read_bytes(cx, reader, bo)?.signed())
        }
    }
}

/// Governs how usize lengths are encoded into a [`Writer`].
#[inline]
pub fn encode_usize<C, W, const OPT: Options>(
    cx: &C,
    writer: W,
    value: usize,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    match crate::options::length::<OPT>() {
        crate::options::Integer::Variable => c::encode(cx, writer, value),
        _ => {
            let bo = crate::options::byteorder::<OPT>();

            macro_rules! fixed {
                ($ty:ty) => {{
                    let Ok(value) = <$ty>::try_from(value) else {
                        return Err(cx.message("Size type out of bounds for value type"));
                    };

                    <$ty as UnsignedOps>::write_bytes(value, cx, writer, bo)
                }};
            }

            crate::options::width_arm!(crate::options::length_width::<OPT>(), fixed)
        }
    }
}

/// Governs how usize lengths are decoded from a [`Reader`].
#[inline]
pub fn decode_usize<'de, C, R, const OPT: Options>(cx: &C, reader: R) -> Result<usize, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    match crate::options::length::<OPT>() {
        crate::options::Integer::Variable => c::decode(cx, reader),
        _ => {
            let bo = crate::options::byteorder::<OPT>();

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

            crate::options::width_arm!(crate::options::length_width::<OPT>(), fixed)
        }
    }
}
