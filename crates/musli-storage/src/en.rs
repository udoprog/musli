use core::fmt;
use core::marker;

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder};
use musli_common::writer::Writer;

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
    type Tuple = Self;
    type Map = Self;
    type Struct = Self;
    type TupleStruct = Self;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline]
    fn encode_unit(self) -> Result<Self::Ok, Self::Error> {
        SequenceEncoder::end(self.encode_sequence(0)?)
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_array<const N: usize>(mut self, array: [u8; N]) -> Result<Self::Ok, Self::Error> {
        self.writer.write_array(array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        let len = vectors.into_iter().map(|v| v.len()).sum();
        L::encode_usize(self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<Self::Ok, Self::Error> {
        self.encode_usize(value as usize)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(if value { 1 } else { 0 })
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value.to_bits())
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(value.to_bits())
    }

    #[inline]
    fn encode_some(mut self) -> Result<Self::Some, Self::Error> {
        self.writer.write_byte(1)?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(0)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::Sequence, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        // NB: tuple has statically known fixed length.
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::Map, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, fields: usize) -> Result<Self::Struct, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), fields)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple_struct(mut self, len: usize) -> Result<Self::TupleStruct, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_unit_struct(mut self) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(self.writer.borrow_mut(), 0)?;
        Ok(())
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::Variant, Self::Error> {
        Ok(self)
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
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W, I, L> PairsEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
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
    type First<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;
    type Second<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
