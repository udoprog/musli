use core::{fmt, marker};

use crate::integer_encoding::{TypedIntegerEncoding, TypedUsizeEncoding};
use crate::tag::{Kind, Tag};
use musli::en::{Encoder, PackEncoder, PairEncoder, PairsEncoder, SequenceEncoder};
use musli::error::Error;
use musli_binary_common::fixed_bytes::FixedBytes;
use musli_binary_common::writer::Writer;
use musli_storage::en::StorageEncoder;

/// A very simple encoder.
pub struct WireEncoder<W, I, L, const P: usize>
where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    writer: W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, I, L, const P: usize> WireEncoder<W, I, L, P>
where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self {
            writer,
            _marker: marker::PhantomData,
        }
    }
}

pub struct WirePackEncoder<'a, I, L, const P: usize, E>
where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    pack_buf: &'a mut FixedBytes<P, E>,
    _marker: marker::PhantomData<(I, L)>,
}

impl<'a, I, L, const P: usize, E> WirePackEncoder<'a, I, L, P, E>
where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(pack_buf: &'a mut FixedBytes<P, E>) -> Self {
        Self {
            pack_buf,
            _marker: marker::PhantomData,
        }
    }
}

impl<W, I, L, const P: usize> Encoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;

    type Pack<'this> = WirePackEncoder<'this, I, L, P, W::Error>;
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
        write!(f, "type supported by the wire encoder")
    }

    #[inline]
    fn encode_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack<T>(mut self, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Pack<'_>) -> Result<(), Self::Error>,
    {
        let mut pack_buf = FixedBytes::new();
        encoder(WirePackEncoder::new(&mut pack_buf))?;
        encode_prefix::<W, L>(&mut self.writer, pack_buf.len())?;
        self.writer.write_bytes(pack_buf.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<Self::Ok, Self::Error> {
        self.encode_bytes(array.as_slice())
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode_prefix::<W, L>(&mut self.writer, bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
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
    fn encode_string(self, string: &str) -> Result<Self::Ok, Self::Error> {
        self.encode_bytes(string.as_bytes())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        L::encode_typed_usize(&mut self.writer, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, Self::Error> {
        L::encode_typed_usize(&mut self.writer, value as usize)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_byte(Tag::new(Kind::Byte, if value { 1 } else { 0 }).byte())
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, Self::Error> {
        let (tag, embedded) = Tag::with_byte(Kind::Byte, value);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            self.writer.write_byte(value)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
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
        self.writer.write_byte(Tag::new(Kind::Sequence, 1).byte())?;
        encoder(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Sequence<'_>) -> Result<(), Self::Error>,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        encoder(self)
    }

    #[inline]
    fn encode_tuple<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Sequence<'_>) -> Result<(), Self::Error>,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        encoder(self)
    }

    #[inline]
    fn encode_map<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Map<'_>) -> Result<(), Self::Error>,
    {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::message(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        encoder(self)
    }

    #[inline]
    fn encode_struct<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::Struct<'_>) -> Result<(), Self::Error>,
    {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::message(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        encoder(self)
    }

    #[inline]
    fn encode_tuple_struct<T>(mut self, len: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::TupleStruct<'_>) -> Result<(), Self::Error>,
    {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::message(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        encoder(self)
    }

    #[inline]
    fn encode_unit_struct(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_struct_variant<T>(mut self, _: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::StructVariant<'_>) -> Result<(), Self::Error>,
    {
        self.writer.write_byte(Tag::new(Kind::Sequence, 2).byte())?;
        encoder(self)
    }

    #[inline]
    fn encode_tuple_variant<T>(mut self, _: usize, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::TupleVariant<'_>) -> Result<(), Self::Error>,
    {
        self.writer.write_byte(Tag::new(Kind::Sequence, 2).byte())?;
        encoder(self)
    }

    #[inline]
    fn encode_unit_variant<T>(mut self, encoder: T) -> Result<Self::Ok, Self::Error>
    where
        T: FnOnce(Self::UnitVariant<'_>) -> Result<(), Self::Error>,
    {
        self.writer.write_byte(Tag::new(Kind::Sequence, 2).byte())?;
        encoder(self)
    }
}

impl<'a, I, L, const P: usize, E> PackEncoder for WirePackEncoder<'a, I, L, P, E>
where
    E: Error,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Ok = ();
    type Error = E;
    type Encoder<'this> = StorageEncoder<&'this mut FixedBytes<P, E>, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(&mut *self.pack_buf))
    }
}

impl<W, I, L, const P: usize> PackEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = WireEncoder<&'this mut W, I, L, P> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(WireEncoder::new(&mut self.writer))
    }
}

impl<W, I, L, const P: usize> SequenceEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = WireEncoder<&'this mut W, I, L, P> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(WireEncoder::new(&mut self.writer))
    }
}

impl<W, I, L, const P: usize> PairsEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = WireEncoder<&'this mut W, I, L, P> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(WireEncoder::new(&mut self.writer))
    }
}

impl<W, I, L, const P: usize> PairEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type First<'this> = WireEncoder<&'this mut W, I, L, P> where Self: 'this;
    type Second = Self;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(WireEncoder::new(&mut self.writer))
    }

    #[inline]
    fn second(self) -> Result<Self::Second, Self::Error> {
        Ok(self)
    }
}

struct Overflow;

impl fmt::Display for Overflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "integer overflow")
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<W, L>(writer: &mut W, len: usize) -> Result<(), W::Error>
where
    W: Writer,
    L: TypedUsizeEncoding,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(tag.byte())?;

    if !embedded {
        L::encode_usize(writer, len)?;
    }

    Ok(())
}
