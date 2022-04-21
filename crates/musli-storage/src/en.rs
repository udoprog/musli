use core::marker;

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use musli::en::{Encoder, PackEncoder, PairEncoder, SequenceEncoder, VariantEncoder};
use musli_binary_common::writer::Writer;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<'a, W, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    writer: &'a mut W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<'a, W, I, L> StorageEncoder<'a, W, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, W, I, L> Encoder for StorageEncoder<'a, W, I, L>
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
        SequenceEncoder::finish(self.encode_sequence(0)?)
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<(), Self::Error> {
        self.writer.write_array(array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        L::encode_usize(&mut self.writer, bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<(), Self::Error> {
        let len = vectors.into_iter().map(|v| v.len()).sum();
        L::encode_usize(&mut self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<(), Self::Error> {
        L::encode_usize(&mut self.writer, string.len())?;
        self.writer.write_bytes(string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<(), Self::Error> {
        L::encode_usize(&mut self.writer, value)
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<(), Self::Error> {
        self.encode_usize(value as usize)
    }

    #[inline]
    fn encode_bool(self, value: bool) -> Result<(), Self::Error> {
        self.writer.write_byte(if value { 1 } else { 0 })
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<(), Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(self, value: u8) -> Result<(), Self::Error> {
        self.writer.write_byte(value)
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
        self.writer.write_byte(1)?;
        Ok(self)
    }

    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        self.writer.write_byte(0)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error> {
        L::encode_usize(&mut *self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error> {
        L::encode_usize(&mut *self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(self, fields: usize) -> Result<Self::Struct, Self::Error> {
        L::encode_usize(&mut *self.writer, fields)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(self, len: usize) -> Result<Self::Tuple, Self::Error> {
        L::encode_usize(&mut *self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_unit_struct(self) -> Result<(), Self::Error> {
        L::encode_usize(&mut *self.writer, 0)?;
        Ok(())
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::Variant, Self::Error> {
        Ok(self)
    }
}

impl<W, I, L> PackEncoder for StorageEncoder<'_, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> SequenceEncoder for StorageEncoder<'a, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;
    type Next<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::Next<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> PairEncoder for StorageEncoder<'a, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;
    type First<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;
    type Second<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn encode_first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn encode_second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> VariantEncoder for StorageEncoder<'a, W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = W::Error;

    type VariantTag<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;
    type VariantValue = Self;

    #[inline]
    fn encode_variant_tag(&mut self) -> Result<Self::VariantTag<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn encode_variant_value(self) -> Result<Self::VariantValue, Self::Error> {
        Ok(self)
    }
}
