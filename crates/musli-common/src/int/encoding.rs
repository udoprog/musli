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
    cx: &mut C,
    writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: Context<Input = W::Error>,
    W: Writer,
    T: Unsigned + UnsignedOps,
{
    match (
        crate::options::integer::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => c::encode(cx, writer, value),
        (_, bo) => value.write_bytes(cx, writer, bo),
    }
}

/// Decode an unsigned value from the specified reader using the configuration
/// passed in through `F`.
#[inline]
pub fn decode_unsigned<'de, C, R, T: UnsignedOps, const F: Options>(
    cx: &mut C,
    reader: R,
) -> Result<T, C::Error>
where
    C: Context<Input = R::Error>,
    R: Reader<'de>,
    T: Unsigned,
{
    match (
        crate::options::integer::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => c::decode(cx, reader),
        (_, bo) => T::read_bytes(cx, reader, bo),
    }
}

/// Governs how signed integers are encoded into a [Writer].
#[inline]
pub fn encode_signed<C, W, T, const F: Options>(
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
    match (
        crate::options::integer::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => c::encode(cx, writer, zig::encode(value)),
        (_, bo) => value.unsigned().write_bytes(cx, writer, bo),
    }
}

/// Governs how signed integers are decoded from a [Reader].
#[inline]
pub fn decode_signed<'de, C, R, T, const F: Options>(cx: &mut C, reader: R) -> Result<T, C::Error>
where
    C: Context<Input = R::Error>,
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    match (
        crate::options::integer::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => {
            let value: T::Unsigned = c::decode(cx, reader)?;
            Ok(zig::decode(value))
        }
        (_, bo) => Ok(T::Unsigned::read_bytes(cx, reader, bo)?.signed()),
    }
}

macro_rules! fixed_arm {
    ($f:ty, $macro:path, $bo:expr) => {
        match crate::options::length_width::<$f>() {
            crate::options::Width::U8 => {
                $macro!(u8, $bo)
            }
            crate::options::Width::U16 => {
                $macro!(u16, $bo)
            }
            crate::options::Width::U32 => {
                $macro!(u32, $bo)
            }
            crate::options::Width::U64 => {
                $macro!(u64, $bo)
            }
        }
    };
}

/// Governs how usize lengths are encoded into a [Writer].
#[inline]
pub fn encode_usize<C, W, const F: Options>(
    cx: &mut C,
    writer: W,
    value: usize,
) -> Result<(), C::Error>
where
    C: Context<Input = W::Error>,
    W: Writer,
{
    macro_rules! fixed {
        ($ty:ty, $bo:expr) => {{
            let Ok(value) = <$ty>::try_from(value) else {
                return Err(cx.message("Size type out of bounds for value type"));
            };

            <$ty as UnsignedOps>::write_bytes(value, cx, writer, $bo)
        }};
    }

    match (
        crate::options::length::<F>(),
        crate::options::byteorder::<F>(),
    ) {
        (crate::options::Integer::Variable, _) => c::encode(cx, writer, value),
        (_, bo) => {
            fixed_arm!(F, fixed, bo)
        }
    }
}

/// Governs how usize lengths are decoded from a [Reader].
#[inline]
pub fn decode_usize<'de, C, R, const F: Options>(cx: &mut C, reader: R) -> Result<usize, C::Error>
where
    C: Context<Input = R::Error>,
    R: Reader<'de>,
{
    macro_rules! fixed {
        ($ty:ty, $bo:expr) => {{
            let Ok(value) = usize::try_from(<$ty as UnsignedOps>::read_bytes(cx, reader, $bo)?)
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
        (crate::options::Integer::Variable, _) => c::decode(cx, reader),
        (_, bo) => {
            fixed_arm!(F, fixed, bo)
        }
    }
}
