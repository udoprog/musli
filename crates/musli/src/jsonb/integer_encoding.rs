use crate::int::{Signed, Unsigned};
use crate::{Context, Reader, Writer};

use super::tag::{Kind, Tag};

#[inline]
pub(crate) fn encode_typed_unsigned<C, W, T>(
    cx: C,
    writer: W,
    bits: u8,
    value: T,
) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
    T: Unsigned,
{
    encode_typed(cx, writer, bits, value)
}

#[inline]
pub(crate) fn decode_typed_unsigned<'de, C, R, T>(cx: C, mut reader: R) -> Result<T, C::Error>
where
    C: Context,
    R: Reader<'de>,
    T: Unsigned + TryFrom<T::Signed>,
{
    let tag = Tag::from_byte(reader.read_byte(cx)?);

    match tag.kind() {
        Kind::Int => Err(cx.message("Unsupported number")),
        _ => Err(cx.message(format_args!("Expected {:?}, got {tag:?}", Kind::Int))),
    }
}

#[inline]
fn encode_typed<C, W, T>(cx: C, _writer: W, _bits: u8, _value: T) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
    T: Unsigned,
{
    Err(cx.message("Unsupported number"))
}

#[inline]
pub(crate) fn encode_typed_signed<C, W, T>(
    cx: C,
    _writer: W,
    _bits: u8,
    _value: T,
) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
    T: Signed,
{
    Err(cx.message("Unsupported number"))
}

#[inline]
pub(crate) fn decode_typed_signed<'de, C, R, T>(cx: C, mut reader: R) -> Result<T, C::Error>
where
    C: Context,
    R: Reader<'de>,
    T: Signed + TryFrom<<T as Signed>::Unsigned>,
{
    let tag = Tag::from_byte(reader.read_byte(cx)?);

    match tag.kind() {
        Kind::Int => Err(cx.message("Unsupported number")),
        _ => Err(cx.message(format_args!("Expected {:?}, got {tag:?}", Kind::Int))),
    }
}
