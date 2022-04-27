use core::fmt;
use core::marker;

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use musli::en::{Encoder, PackEncoder, PairEncoder, PairsEncoder, SequenceEncoder};
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

    type Pack<'this> = Self;
    type Some<'this> = Self;
    type Sequence<'this> = Self;
    type Tuple<'this> = Self;
    type Map<'this> = Self;
    type Struct<'this> = Self;
    type TupleStruct<'this> = Self;
    type StructVariant<'this> = Self;
    type TupleVariant<'this> = Self;
    type UnitVariant<'this> = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline]
    fn encode_unit(self) -> Result<Self::Ok, Self::Error> {
        self.encode_sequence(0, |_| Ok(()))
    }

    #[inline]
    fn encode_pack<T>(self, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Pack<'_>) -> Result<(), Self::Error>,
    {
        encoder(self)
    }

    #[inline]
    fn encode_array<const N: usize>(mut self, array: [u8; N]) -> Result<Self::Ok, Self::Error> {
        self.writer.write_array(array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(&mut self.writer, bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        let len = vectors.into_iter().map(|v| v.len()).sum();
        L::encode_usize(&mut self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(&mut self.writer, string.len())?;
        self.writer.write_bytes(string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(&mut self.writer, value)
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
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(&mut self.writer, value)
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
    fn encode_some<T>(mut self, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Some<'_>) -> Result<(), Self::Error>,
    {
        self.writer.write_byte(1)?;
        encoder(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(0)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Sequence<'_>) -> Result<(), Self::Error>,
    {
        L::encode_usize(&mut self.writer, len)?;
        encoder(self)
    }

    #[inline]
    fn encode_tuple<T>(self, _: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Sequence<'_>) -> Result<(), Self::Error>,
    {
        // NB: tuple has statically known fixed length.
        encoder(self)
    }

    #[inline]
    fn encode_map<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Map<'_>) -> Result<(), Self::Error>,
    {
        L::encode_usize(&mut self.writer, len)?;
        encoder(self)
    }

    #[inline]
    fn encode_struct<T>(mut self, fields: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Struct<'_>) -> Result<(), Self::Error>,
    {
        L::encode_usize(&mut self.writer, fields)?;
        encoder(self)
    }

    #[inline]
    fn encode_tuple_struct<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::TupleStruct<'_>) -> Result<(), Self::Error>,
    {
        L::encode_usize(&mut self.writer, len)?;
        encoder(self)
    }

    #[inline]
    fn encode_unit_struct(mut self) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(&mut self.writer, 0)?;
        Ok(())
    }

    #[inline]
    fn encode_struct_variant<T>(self, _: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::StructVariant<'_>) -> Result<(), Self::Error>,
    {
        encoder(self)
    }

    #[inline]
    fn encode_tuple_variant<T>(self, _: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::TupleVariant<'_>) -> Result<(), Self::Error>,
    {
        encoder(self)
    }

    #[inline]
    fn encode_unit_variant<T>(self, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::UnitVariant<'_>) -> Result<(), Self::Error>,
    {
        encoder(self)
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
}

impl<W, I, L> PairsEncoder for StorageEncoder<W, I, L>
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
    fn first<'a, F, O>(&'a mut self, encoder: F) -> Result<O, Self::Error>
    where
        F: FnOnce(Self::First<'a>) -> Result<O, Self::Error>,
    {
        encoder(StorageEncoder::new(&mut self.writer))
    }

    #[inline]
    fn second<'a, F, O>(&'a mut self, encoder: F) -> Result<O, Self::Error>
    where
        F: FnOnce(Self::First<'a>) -> Result<O, Self::Error>,
    {
        encoder(StorageEncoder::new(&mut self.writer))
    }
}
