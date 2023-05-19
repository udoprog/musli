use musli::Context;

use musli_common::int::continuation as c;
use musli_common::int::zigzag as zig;
use musli_common::int::{Signed, Unsigned};
use musli_common::reader::Reader;
use musli_common::writer::Writer;

use crate::tag::{Kind, Tag};

#[inline]
pub(crate) fn encode_typed_unsigned<W, T>(writer: W, bits: u8, value: T) -> Result<(), W::Error>
where
    W: Writer,
    T: Unsigned,
{
    encode_typed(writer, Kind::Number, bits, value)
}

#[inline]
pub(crate) fn decode_typed_unsigned<'de, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
where
    C: Context<R::Error>,
    R: Reader<'de>,
    T: Unsigned,
{
    decode_typed(cx, reader, Kind::Number)
}

#[inline]
fn encode_typed<W, T>(mut writer: W, kind: Kind, bits: u8, value: T) -> Result<(), W::Error>
where
    W: Writer,
    T: Unsigned,
{
    writer.write_byte(Tag::new(kind, bits).byte())?;
    c::encode(writer, value)
}

#[inline]
fn decode_typed<'de, C, R, T>(cx: &mut C, mut reader: R, kind: Kind) -> Result<T, C::Error>
where
    C: Context<R::Error>,
    R: Reader<'de>,
    T: Unsigned,
{
    let tag = Tag::from_byte(reader.read_byte(cx)?);

    if tag.kind() != kind {
        return Err(cx.message(format_args!("expected {kind:?}, got {tag:?}")));
    }

    c::decode(cx, reader)
}

#[inline]
pub(crate) fn encode_typed_signed<W, T>(writer: W, bits: u8, value: T) -> Result<(), W::Error>
where
    W: Writer,
    T: Signed,
{
    encode_typed(writer, Kind::Number, bits, zig::encode(value))
}

#[inline]
pub(crate) fn decode_typed_signed<'de, C, R, T>(cx: &mut C, reader: R) -> Result<T, C::Error>
where
    C: Context<R::Error>,
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: Unsigned<Signed = T>,
{
    let value: T::Unsigned = decode_typed(cx, reader, Kind::Number)?;
    Ok(zig::decode(value))
}
