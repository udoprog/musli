use core::{fmt, marker};

use crate::integer_encoding::{WireIntegerEncoding, WireUsizeEncoding};
use crate::tag::{Kind, Tag};
use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::error::Error;
use musli_common::fixed_bytes::FixedBytes;
use musli_common::writer::Writer;
use musli_storage::en::StorageEncoder;
use musli_storage::int::Variable;

/// A very simple encoder.
pub struct WireEncoder<W, I, L, const P: usize>
where
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    writer: W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, I, L, const P: usize> WireEncoder<W, I, L, P>
where
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
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

pub struct WirePackEncoder<W, I, L, const P: usize>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    writer: W,
    pack_buf: FixedBytes<P, W::Error>,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, I, L, const P: usize> WirePackEncoder<W, I, L, P>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W, pack_buf: FixedBytes<P, W::Error>) -> Self {
        Self {
            writer,
            pack_buf,
            _marker: marker::PhantomData,
        }
    }
}

#[musli::encoder]
impl<W, I, L, const P: usize> Encoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;

    type Pack = WirePackEncoder<W, I, L, P>;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Struct = Self;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire encoder")
    }

    #[inline(always)]
    fn encode_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(WirePackEncoder::new(self.writer, FixedBytes::new()))
    }

    #[inline(always)]
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<Self::Ok, Self::Error> {
        self.encode_bytes(array.as_slice())
    }

    #[inline(always)]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode_prefix::<W, L>(&mut self.writer, bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix::<W, L>(&mut self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string(self, string: &str) -> Result<Self::Ok, Self::Error> {
        self.encode_bytes(string.as_bytes())
    }

    #[inline(always)]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        L::encode_typed_usize(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, Self::Error> {
        L::encode_typed_usize(&mut self.writer, value as usize)
    }

    #[inline(always)]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_byte(Tag::new(Kind::Byte, if value { 1 } else { 0 }).byte())
    }

    #[inline(always)]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline(always)]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, Self::Error> {
        let (tag, embedded) = Tag::with_byte(Kind::Byte, value);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            self.writer.write_byte(value)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_unsigned(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(value as u8)
    }

    #[inline(always)]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        I::encode_typed_signed(&mut self.writer, value)
    }

    #[inline(always)]
    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value.to_bits())
    }

    #[inline(always)]
    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(value.to_bits())
    }

    #[inline(always)]
    fn encode_some(mut self) -> Result<Self::Some, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 1).byte())?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::Sequence, Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, len: usize) -> Result<Self::Tuple, Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::Map, Self::Error> {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::message(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, len: usize) -> Result<Self::Struct, Self::Error> {
        let len = len
            .checked_mul(2)
            .ok_or_else(|| Self::Error::message(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            L::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_variant(mut self) -> Result<Self::Variant, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
    }
}

impl<W, I, L, const P: usize> SequenceEncoder for WirePackEncoder<W, I, L, P>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<&'this mut FixedBytes<P, W::Error>, Variable, Variable> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(StorageEncoder::new(&mut self.pack_buf))
    }

    #[inline]
    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        encode_prefix::<W, L>(&mut self.writer, self.pack_buf.len())?;
        self.writer.write_bytes(self.pack_buf.as_slice())?;
        Ok(())
    }
}

impl<W, I, L, const P: usize> SequenceEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W, I, L, const P: usize> PairsEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W, I, L, const P: usize> PairEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type First<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;
    type Second<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W, I, L, const P: usize> VariantEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Tag<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;
    type Variant<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
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
    L: WireUsizeEncoding,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(tag.byte())?;

    if !embedded {
        L::encode_usize(writer, len)?;
    }

    Ok(())
}
