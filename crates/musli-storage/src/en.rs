use core::fmt;
use core::marker;

use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli_common::int::{IntegerEncoding, UsizeEncoding, Variable};
use musli_common::writer::Writer;

/// The alias for a [StorageEncoder] that is used for packs.
pub type PackEncoder<'a, W> = StorageEncoder<'a, W, Variable, Variable>;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<'a, W, I, L>
where
    W: ?Sized,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    writer: &'a mut W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<'a, W, I, L> StorageEncoder<'a, W, I, L>
where
    W: ?Sized,
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

#[musli::encoder]
impl<'a, W, I, L> Encoder for StorageEncoder<'a, W, I, L>
where
    W: ?Sized + Writer,
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

    #[inline(always)]
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<Self::Ok, Self::Error> {
        self.writer.write_array(array)
    }

    #[inline(always)]
    fn encode_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(&mut *self.writer, bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(self, vectors: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        let len = vectors.iter().map(|v| v.len()).sum();
        L::encode_usize(&mut *self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string(self, string: &str) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(&mut *self.writer, string.len())?;
        self.writer.write_bytes(string.as_bytes())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_usize(self, value: usize) -> Result<Self::Ok, Self::Error> {
        L::encode_usize(self.writer, value)
    }

    #[inline(always)]
    fn encode_isize(self, value: isize) -> Result<Self::Ok, Self::Error> {
        self.encode_usize(value as usize)
    }

    #[inline(always)]
    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(if value { 1 } else { 0 })
    }

    #[inline(always)]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline(always)]
    fn encode_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(value)
    }

    #[inline(always)]
    fn encode_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer, value)
    }

    #[inline(always)]
    fn encode_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer, value)
    }

    #[inline(always)]
    fn encode_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer, value)
    }

    #[inline(always)]
    fn encode_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        I::encode_unsigned(self.writer, value)
    }

    #[inline(always)]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(value as u8)
    }

    #[inline(always)]
    fn encode_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer, value)
    }

    #[inline(always)]
    fn encode_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer, value)
    }

    #[inline(always)]
    fn encode_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer, value)
    }

    #[inline(always)]
    fn encode_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        I::encode_signed(self.writer, value)
    }

    #[inline(always)]
    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value.to_bits())
    }

    #[inline(always)]
    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(value.to_bits())
    }

    #[inline]
    fn encode_some(self) -> Result<Self::Some, Self::Error> {
        self.writer.write_byte(1)?;
        Ok(self)
    }

    #[inline]
    fn encode_none(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(0)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error> {
        L::encode_usize(&mut *self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        // NB: A tuple has statically known fixed length.
        Ok(self)
    }

    #[inline]
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error> {
        L::encode_usize(&mut *self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(self, len: usize) -> Result<Self::Struct, Self::Error> {
        L::encode_usize(&mut *self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::Variant, Self::Error> {
        Ok(self)
    }
}

impl<'a, W, I, L> SequenceEncoder for StorageEncoder<'a, W, I, L>
where
    W: ?Sized + Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> PairsEncoder for StorageEncoder<'a, W, I, L>
where
    W: ?Sized + Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> PairEncoder for StorageEncoder<'a, W, I, L>
where
    W: ?Sized + Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type First<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;
    type Second<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W, I, L> VariantEncoder for StorageEncoder<'a, W, I, L>
where
    W: ?Sized + Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Tag<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;
    type Variant<'this> = StorageEncoder<'this, W, I, L> where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
