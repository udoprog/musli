use core::fmt;
use core::marker;

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use musli::en::{Encoder, PackEncoder, PairEncoder, SequenceEncoder};
use musli_binary_common::writer::Writer;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<W, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    writer: W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, I, L> StorageEncoder<W, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _marker: marker::PhantomData,
        }
    }
}

impl<W, I, L> Encoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;

    type Pack = Self;
    type Some = Self;
    type Sequence = Self;
    type Map = Self;
    type Struct = Self;
    type Tuple = Self;
    type StructVariant = Self;
    type TupleVariant = Self;
    type UnitVariant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline]
    fn encode_unit(self) -> Result<(), Self::Error> {
        SequenceEncoder::end(self.encode_sequence(0)?)
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_array<const N: usize>(mut self, array: [u8; N]) -> Result<(), Self::Error> {
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
    fn encode_bool(mut self, value: bool) -> Result<(), Self::Error> {
        self.writer.write_byte(if value { 1 } else { 0 })
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<(), Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<(), Self::Error> {
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
    fn encode_some(mut self) -> Result<Self::Some, Self::Error> {
        self.writer.write_byte(1)?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<(), Self::Error> {
        self.writer.write_byte(0)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::Sequence, Self::Error> {
        L::encode_usize(&mut self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::Map, Self::Error> {
        L::encode_usize(&mut self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, fields: usize) -> Result<Self::Struct, Self::Error> {
        L::encode_usize(&mut self.writer, fields)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, len: usize) -> Result<Self::Tuple, Self::Error> {
        L::encode_usize(&mut self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_unit_struct(mut self) -> Result<(), Self::Error> {
        L::encode_usize(&mut self.writer, 0)?;
        Ok(())
    }

    #[inline]
    fn encode_struct_variant(self, _: usize) -> Result<Self::StructVariant, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_tuple_variant(self, _: usize) -> Result<Self::TupleVariant, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_unit_variant(self) -> Result<Self::UnitVariant, Self::Error> {
        Ok(self)
    }
}

impl<W, I, L> PackEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<&'this mut W, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(&mut self.writer))
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<W, I, L> SequenceEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<&'this mut W, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(&mut self.writer))
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<W, I, L> PairEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type First<'this> = StorageEncoder<&'this mut W, I, L> where Self: 'this;
    type Second<'this> = StorageEncoder<&'this mut W, I, L> where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(StorageEncoder::new(&mut self.writer))
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(StorageEncoder::new(&mut self.writer))
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
