use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned, UnsignedOps};
use crate::{Context, Options, Reader, Writer};

/// Governs how unsigned integers are encoded into a [`Writer`].
#[inline]
pub(crate) fn encode_unsigned<C, W, T, const OPT: Options>(
    cx: C,
    writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
    T: Unsigned + UnsignedOps,
{
    match crate::options::integer::<OPT>() {
        crate::options::Integer::Variable => c::encode(cx, writer, value),
        crate::options::Integer::Fixed => {
            let bo = crate::options::byteorder::<OPT>();
            value.write_bytes(cx, writer, bo)
        }
    }
}

/// Decode an unsigned value from the specified reader using the configuration
/// passed in through `F`.
#[inline]
pub(crate) fn decode_unsigned<'de, C, R, T, const OPT: Options>(
    cx: C,
    reader: R,
) -> Result<T, C::Error>
where
    C: Context,
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
pub(crate) fn encode_signed<C, W, T, const OPT: Options>(
    cx: C,
    writer: W,
    value: T,
) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    match crate::options::integer::<OPT>() {
        crate::options::Integer::Variable => c::encode(cx, writer, zig::encode(value)),
        crate::options::Integer::Fixed => {
            let bo = crate::options::byteorder::<OPT>();
            value.unsigned().write_bytes(cx, writer, bo)
        }
    }
}

/// Governs how signed integers are decoded from a [`Reader`].
#[inline]
pub(crate) fn decode_signed<'de, C, R, T, const OPT: Options>(
    cx: C,
    reader: R,
) -> Result<T, C::Error>
where
    C: Context,
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: UnsignedOps,
{
    match crate::options::integer::<OPT>() {
        crate::options::Integer::Variable => {
            let value: T::Unsigned = c::decode(cx, reader)?;
            Ok(zig::decode(value))
        }
        crate::options::Integer::Fixed => {
            let bo = crate::options::byteorder::<OPT>();
            Ok(T::Unsigned::read_bytes(cx, reader, bo)?.signed())
        }
    }
}

/// Governs how usize lengths are encoded into a [`Writer`].
#[inline]
pub(crate) fn encode_usize<C, W, const OPT: Options>(
    cx: C,
    writer: W,
    value: usize,
) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
{
    match crate::options::length::<OPT>() {
        crate::options::Width::Variable => c::encode(cx, writer, value),
        width => {
            let bo = crate::options::byteorder::<OPT>();

            macro_rules! fixed {
                ($ty:ty) => {{
                    let Ok(value) = <$ty>::try_from(value) else {
                        return Err(cx.message("Size type out of bounds for value type"));
                    };

                    <$ty as UnsignedOps>::write_bytes(value, cx, writer, bo)
                }};
            }

            crate::options::width_arm!(width, fixed)
        }
    }
}

/// Governs how isize values are encoded into a [`Writer`].
///
/// Note that this is separate from [`encode_usize`], since a negative value has
/// to be narrowed as a signed value rather than reinterpreted as a `usize` at
/// the width of the host.
#[inline]
pub(crate) fn encode_isize<C, W, const OPT: Options>(
    cx: C,
    writer: W,
    value: isize,
) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
{
    match crate::options::length::<OPT>() {
        crate::options::Width::Variable => c::encode(cx, writer, zig::encode(value)),
        width => {
            let bo = crate::options::byteorder::<OPT>();

            macro_rules! fixed {
                ($signed:ty, $unsigned:ty) => {{
                    let Ok(value) = <$signed>::try_from(value) else {
                        return Err(cx.message("Size type out of bounds for value type"));
                    };

                    <$unsigned as UnsignedOps>::write_bytes(value as $unsigned, cx, writer, bo)
                }};
            }

            crate::options::signed_width_arm!(width, fixed)
        }
    }
}

/// Governs how isize values are decoded from a [`Reader`].
#[inline]
pub(crate) fn decode_isize<'de, C, R, const OPT: Options>(
    cx: C,
    reader: R,
) -> Result<isize, C::Error>
where
    C: Context,
    R: Reader<'de>,
{
    match crate::options::length::<OPT>() {
        crate::options::Width::Variable => Ok(zig::decode(c::decode::<_, _, usize>(cx, reader)?)),
        width => {
            let bo = crate::options::byteorder::<OPT>();

            macro_rules! fixed {
                ($signed:ty, $unsigned:ty) => {{
                    let value = <$unsigned as UnsignedOps>::read_bytes(cx, reader, bo)?;
                    // NB: sign extended through the signed type of the width.
                    Ok(value as $signed as isize)
                }};
            }

            crate::options::signed_width_arm!(width, fixed)
        }
    }
}

/// Governs how usize lengths are decoded from a [`Reader`].
#[inline]
pub(crate) fn decode_usize<'de, C, R, const OPT: Options>(
    cx: C,
    reader: R,
) -> Result<usize, C::Error>
where
    C: Context,
    R: Reader<'de>,
{
    match crate::options::length::<OPT>() {
        crate::options::Width::Variable => c::decode(cx, reader),
        width => {
            let bo = crate::options::byteorder::<OPT>();

            macro_rules! fixed {
                ($ty:ty) => {{
                    #[allow(irrefutable_let_patterns)]
                    let Ok(value) =
                        usize::try_from(<$ty as UnsignedOps>::read_bytes(cx, reader, bo)?)
                    else {
                        return Err(cx.message("Value type out of bounds for usize"));
                    };

                    Ok(value)
                }};
            }

            crate::options::width_arm!(width, fixed)
        }
    }
}
