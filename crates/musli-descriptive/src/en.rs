use core::fmt;

use crate::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use crate::tag::{
    Kind, Tag, ABSENT, F32, F64, FALSE, I128, I16, I32, I64, I8, ISIZE, PRESENT, TRUE, U128, U16,
    U32, U64, U8, USIZE, VARIANT,
};
use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli_common::encoding::Variable;
use musli_common::fixed_bytes::FixedBytes;
use musli_common::writer::Writer;
use musli_storage::en::StorageEncoder;
use musli_storage::integer_encoding::UsizeEncoding;

/// A very simple encoder.
pub struct SelfEncoder<W, const P: usize> {
    writer: W,
}

impl<W, const P: usize> SelfEncoder<W, P> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }
}

pub struct SelfPackEncoder<W, const P: usize>
where
    W: Writer,
{
    writer: W,
    pack_buf: FixedBytes<P, W::Error>,
}

impl<W, const P: usize> SelfPackEncoder<W, P>
where
    W: Writer,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W, pack_buf: FixedBytes<P, W::Error>) -> Self {
        Self { writer, pack_buf }
    }
}

impl<W, const P: usize> Encoder for SelfEncoder<W, P>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;

    type Pack = SelfPackEncoder<W, P>;
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

    #[inline]
    fn encode_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(SelfPackEncoder::new(self.writer, FixedBytes::new()))
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<Self::Ok, Self::Error> {
        self.encode_bytes(array.as_slice())
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode_bytes_tag::<W>(&mut self.writer, bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_bytes_tag::<W>(&mut self.writer, len)?;

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
        encode_typed_unsigned(&mut self.writer, USIZE, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(&mut self.writer, ISIZE, value)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_byte(Tag::new(Kind::Marker, if value { TRUE } else { FALSE }).byte())
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(&mut self.writer, U8, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(&mut self.writer, U16, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(&mut self.writer, U32, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(&mut self.writer, U64, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(&mut self.writer, U128, value)
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(&mut self.writer, I8, value)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(&mut self.writer, I16, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(&mut self.writer, I32, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(&mut self.writer, I64, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(&mut self.writer, I128, value)
    }

    #[inline]
    fn encode_f32(mut self, value: f32) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(&mut self.writer, F32, value.to_bits())
    }

    #[inline]
    fn encode_f64(mut self, value: f64) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(&mut self.writer, F64, value.to_bits())
    }

    #[inline]
    fn encode_some(mut self) -> Result<Self::Some, Self::Error> {
        self.writer
            .write_byte(Tag::new(Kind::Marker, PRESENT).byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_byte(Tag::new(Kind::Marker, ABSENT).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::Sequence, Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            Variable::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, len: usize) -> Result<Self::Sequence, Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            Variable::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::Map, Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Map, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            Variable::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, len: usize) -> Result<Self::Struct, Self::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Map, len);
        self.writer.write_byte(tag.byte())?;

        if !embedded {
            Variable::encode_usize(&mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_variant(mut self) -> Result<Self::Variant, Self::Error> {
        self.writer
            .write_byte(Tag::new(Kind::Marker, VARIANT).byte())?;
        Ok(self)
    }
}

impl<W, const P: usize> SequenceEncoder for SelfPackEncoder<W, P>
where
    W: Writer,
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
        encode_bytes_tag::<W>(&mut self.writer, self.pack_buf.len())?;
        self.writer.write_bytes(self.pack_buf.as_slice())?;
        Ok(())
    }
}

impl<W, const P: usize> SequenceEncoder for SelfEncoder<W, P>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = SelfEncoder<W::Mut<'this>, P> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W, const P: usize> PairsEncoder for SelfEncoder<W, P>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = SelfEncoder<W::Mut<'this>, P> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W, const P: usize> PairEncoder for SelfEncoder<W, P>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;
    type First<'this> = SelfEncoder<W::Mut<'this>, P> where Self: 'this;
    type Second<'this> = SelfEncoder<W::Mut<'this>, P> where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W, const P: usize> VariantEncoder for SelfEncoder<W, P>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;
    type Tag<'this> = SelfEncoder<W::Mut<'this>, P> where Self: 'this;
    type Variant<'this> = SelfEncoder<W::Mut<'this>, P> where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
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
fn encode_bytes_tag<W>(writer: &mut W, len: usize) -> Result<(), W::Error>
where
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(Kind::Bytes, len);
    writer.write_byte(tag.byte())?;

    if !embedded {
        Variable::encode_usize(writer, len)?;
    }

    Ok(())
}
