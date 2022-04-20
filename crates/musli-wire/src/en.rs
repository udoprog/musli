use core::{fmt, marker};

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use crate::tag::{Kind, Tag};
use musli::en::{Encoder, PackEncoder, PairEncoder, SequenceEncoder, VariantEncoder};
use musli::error::Error;
use musli_binary_common::writer::Writer;

/// A very simple encoder.
pub struct WireEncoder<'a, W, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    writer: &'a mut W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<'a, W, I, L> WireEncoder<'a, W, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, W, I, L> Encoder for WireEncoder<'a, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;

    type Pack = Self;
    type Some = Self;
    type Sequence = Self;
    type Map = Self;
    type Struct = Self;
    type Tuple = Self;
    type Variant = Self;

    #[inline]
    fn encode_unit(self) -> Result<(), Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<(), Self::Error> {
        self.encode_bytes(array.as_slice())
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Prefix, bytes.len());
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, bytes.len())?;
        }

        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<(), Self::Error> {
        let len = vectors.into_iter().map(|v| v.len()).sum();

        let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        for bytes in vectors {
            self.writer.write_bytes(bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(self, string: &str) -> Result<(), Self::Error> {
        self.encode_bytes(string.as_bytes())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<(), Self::Error> {
        L::encode_typed_usize(&mut self.writer, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<(), Self::Error> {
        L::encode_typed_usize(&mut self.writer, value as usize)
    }

    #[inline]
    fn encode_bool(self, value: bool) -> Result<(), Self::Error> {
        self.writer
            .write_byte(Tag::new(Kind::Byte, if value { 1 } else { 0 }).byte())
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<(), Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(self, value: u8) -> Result<(), Self::Error> {
        let (tag, embedded) = Tag::with_byte(Kind::Byte, value);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            self.writer.write_byte(value)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<(), Self::Error> {
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<(), Self::Error> {
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<(), Self::Error> {
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<(), Self::Error> {
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<(), Self::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<(), Self::Error> {
        I::encode_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<(), Self::Error> {
        I::encode_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<(), Self::Error> {
        I::encode_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<(), Self::Error> {
        I::encode_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<(), Self::Error> {
        self.encode_u32(value.to_bits())
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<(), Self::Error> {
        self.encode_u64(value.to_bits())
    }

    #[inline]
    fn encode_some(self) -> Result<Self::Some, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 1).byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut *self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error> {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::collect_from_display(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut *self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_struct(self, len: usize) -> Result<Self::Struct, Self::Error> {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::collect_from_display(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut *self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple(self, len: usize) -> Result<Self::Tuple, Self::Error> {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::collect_from_display(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut *self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_unit_struct(self) -> Result<(), Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::Variant, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
    }
}

impl<W, I, L> PackEncoder for WireEncoder<'_, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;
    type Encoder<'this> = WireEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer))
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> SequenceEncoder for WireEncoder<'a, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;
    type Next<'this> = WireEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::Next<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer))
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> PairEncoder for WireEncoder<'a, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;
    type First<'this> = WireEncoder<'this, W, I, L> where Self: 'this;
    type Second<'this> = WireEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn encode_first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer))
    }

    #[inline]
    fn encode_second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer))
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> VariantEncoder for WireEncoder<'a, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;

    type VariantTag<'this> = WireEncoder<'this, W, I, L> where Self: 'this;
    type VariantValue = Self;

    #[inline]
    fn encode_variant_tag(&mut self) -> Result<Self::VariantTag<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer))
    }

    #[inline]
    fn encode_variant_value(self) -> Result<Self::VariantValue, Self::Error> {
        Ok(self)
    }
}

struct Overflow;

impl fmt::Display for Overflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "integer overflow")
    }
}
