use crate::tag::{Kind, Tag};
use musli::error::Error;
use musli_common::int::continuation as c;
use musli_common::int::zigzag as zig;
use musli_common::int::{Signed, Unsigned};
use musli_common::reader::Reader;
use musli_common::writer::Writer;

#[inline]
pub(crate) fn encode_typed_unsigned<W, T>(writer: W, bits: u8, value: T) -> Result<(), W::Error>
where
    W: Writer,
    T: Unsigned,
{
    encode_typed(writer, Kind::Number, bits, value)
}

#[inline]
pub(crate) fn decode_typed_unsigned<'de, R, T>(reader: R) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: Unsigned,
{
    decode_typed(reader, Kind::Number)
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
fn decode_typed<'de, R, T>(mut reader: R, kind: Kind) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: Unsigned,
{
    let tag = Tag::from_byte(reader.read_byte()?);

    if tag.kind() != kind {
        return Err(R::Error::message(format_args!(
            "expected {kind:?}, got {tag:?}"
        )));
    }

    c::decode(reader)
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
pub(crate) fn decode_typed_signed<'de, R, T>(reader: R) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: Signed,
    T::Unsigned: Unsigned<Signed = T>,
{
    let value: T::Unsigned = decode_typed(reader, Kind::Number)?;
    Ok(zig::decode(value))
}
