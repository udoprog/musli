use musli::Context;

use musli_common::int::continuation as c;
use musli_common::int::zigzag as zig;
use musli_common::int::{Signed, Unsigned};
use musli_common::reader::Reader;
use musli_common::writer::Writer;

use crate::error::Error;
use crate::tag::{Kind, Tag};

#[inline]
pub(crate) fn encode_typed_unsigned<C, W, T>(
    cx: &mut C,
    writer: W,
    bits: u8,
    value: T,
) -> Result<(), C::Error>
where
    C: Context<Input = Error>,
    W: Writer,
    Error: From<W::Error>,
    T: Unsigned,
{
    encode_typed(cx, writer, Kind::Number, bits, value)
}

#[inline]
pub(crate) fn decode_typed_unsigned<'de, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
where
    C: Context<Input = Error>,
    R: Reader<'de>,
    Error: From<R::Error>,
    T: Unsigned,
{
    decode_typed(cx, reader, Kind::Number)
}

#[inline]
fn encode_typed<C, W, T>(
    cx: &mut C,
    mut writer: W,
    kind: Kind,
    bits: u8,
    value: T,
) -> Result<(), C::Error>
where
    C: Context<Input = Error>,
    W: Writer,
    Error: From<W::Error>,
    T: Unsigned,
{
    writer.write_byte(cx.adapt(), Tag::new(kind, bits).byte())?;
    c::encode(cx.adapt(), writer, value)
}

#[inline]
fn decode_typed<'de, C, R, T>(cx: &mut C, mut reader: R, kind: Kind) -> Result<T, C::Error>
where
    C: Context<Input = Error>,
    R: Reader<'de>,
    Error: From<R::Error>,
    T: Unsigned,
{
    let tag = Tag::from_byte(reader.read_byte(cx.adapt())?);

    if tag.kind() != kind {
        return Err(cx.message(format_args!("Expected {kind:?}, got {tag:?}")));
    }

    c::decode(cx.adapt(), reader)
}

#[inline]
pub(crate) fn encode_typed_signed<C, W, T>(
    cx: &mut C,
    writer: W,
    bits: u8,
    value: T,
) -> Result<(), C::Error>
where
    C: Context<Input = Error>,
    W: Writer,
    Error: From<W::Error>,
    T: Signed,
{
    encode_typed(cx, writer, Kind::Number, bits, zig::encode(value))
}

#[inline]
pub(crate) fn decode_typed_signed<'de, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
where
    C: Context<Input = Error>,
    R: Reader<'de>,
    Error: From<R::Error>,
    T: Signed,
    T::Unsigned: Unsigned<Signed = T>,
{
    let value: T::Unsigned = decode_typed(cx, reader, Kind::Number)?;
    Ok(zig::decode(value))
}
