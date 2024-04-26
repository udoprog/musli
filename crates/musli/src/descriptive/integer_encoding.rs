use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned};
use crate::{Context, Reader, Writer};

use super::tag::{Kind, NumberKind, Tag};

#[inline]
pub(crate) fn encode_typed_unsigned<C, W, T>(
    cx: &C,
    writer: W,
    bits: u8,
    value: T,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: Unsigned,
{
    encode_typed(cx, writer, bits, value)
}

#[inline]
pub(crate) fn decode_typed_unsigned<'de, C, R, T>(cx: &C, reader: R) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: Unsigned + TryFrom<T::Signed>,
{
    let (value, kind): (T, NumberKind) = decode_typed(cx, reader)?;

    match kind {
        NumberKind::Signed => {
            let value = zig::decode(value);

            let Ok(value) = T::try_from(value) else {
                return Err(cx.message(format_args!("Unsigned value outside of signed range")));
            };

            Ok(value)
        }
        NumberKind::Unsigned | NumberKind::Float => Ok(value),
        kind => Err(cx.message(format_args!(
            "Expected signed or unsigned number, got {:?}",
            kind
        ))),
    }
}

#[inline]
fn encode_typed<C, W, T>(cx: &C, mut writer: W, bits: u8, value: T) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: Unsigned,
{
    writer.write_byte(cx, Tag::new(Kind::Number, bits).byte())?;
    c::encode(cx, writer, value)
}

#[inline]
fn decode_typed<'de, C, R, T>(cx: &C, mut reader: R) -> Result<(T, NumberKind), C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: Unsigned,
{
    let tag = Tag::from_byte(reader.read_byte(cx)?);

    if tag.kind() != Kind::Number {
        return Err(cx.message(format_args!("Expected {:?}, got {tag:?}", Kind::Number)));
    }

    let kind = tag.number_kind();
    Ok((c::decode(cx, reader)?, kind))
}

#[inline]
pub(crate) fn encode_typed_signed<C, W, T>(
    cx: &C,
    writer: W,
    bits: u8,
    value: T,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: Signed,
{
    encode_typed(cx, writer, bits, zig::encode(value))
}

#[inline]
pub(crate) fn decode_typed_signed<'de, C, R, T>(cx: &C, reader: R) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: Signed + TryFrom<<T as Signed>::Unsigned>,
{
    let (value, kind): (T::Unsigned, NumberKind) = decode_typed(cx, reader)?;

    match kind {
        NumberKind::Signed => Ok(zig::decode(value)),
        NumberKind::Unsigned => {
            let Ok(value) = T::try_from(value) else {
                return Err(cx.message(format_args!("Unsigned value outside of signed range")));
            };

            Ok(value)
        }
        kind => Err(cx.message(format_args!(
            "Expected signed or unsigned number, got {:?}",
            kind
        ))),
    }
}
