use core::fmt;

use crate::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use crate::tag::{
    Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, ISIZE, U128, U16, U32, U64, U8, USIZE,
};
use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::error::Error;
use musli_common::fixed_bytes::FixedBytes;
use musli_common::int::{continuation as c, UsizeEncoding, Variable};
use musli_common::writer::Writer;
use musli_storage::en::StorageEncoder;

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
    count: usize,
}

impl<W, const P: usize> SelfPackEncoder<W, P>
where
    W: Writer,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W, pack_buf: FixedBytes<P, W::Error>) -> Self {
        Self {
            writer,
            pack_buf,
            count: 0,
        }
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
        write!(f, "type supported by the descriptive encoder")
    }

    #[inline]
    fn encode_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(Tag::from_mark(Mark::Unit).byte())?;
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
        encode_prefix(self.writer.borrow_mut(), Kind::Bytes, bytes.len())?;
        self.writer.write_bytes(bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, vectors: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix(self.writer.borrow_mut(), Kind::Bytes, len)?;

        for bytes in vectors {
            self.writer.write_bytes(bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, Self::Error> {
        encode_prefix(self.writer.borrow_mut(), Kind::String, string.len())?;
        self.writer.write_bytes(string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), USIZE, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(self.writer.borrow_mut(), ISIZE, value)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        const TRUE: Tag = Tag::from_mark(Mark::True);
        const FALSE: Tag = Tag::from_mark(Mark::False);

        self.writer
            .write_byte(if value { TRUE } else { FALSE }.byte())
    }

    #[inline]
    fn encode_char(mut self, value: char) -> Result<Self::Ok, Self::Error> {
        const CHAR: Tag = Tag::from_mark(Mark::Char);
        self.writer.write_byte(CHAR.byte())?;
        c::encode(self.writer.borrow_mut(), value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), U8, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), U16, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), U32, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), U64, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), U128, value)
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(self.writer.borrow_mut(), I8, value)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(self.writer.borrow_mut(), I16, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(self.writer.borrow_mut(), I32, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(self.writer.borrow_mut(), I64, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        encode_typed_signed(self.writer.borrow_mut(), I128, value)
    }

    #[inline]
    fn encode_f32(mut self, value: f32) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), F32, value.to_bits())
    }

    #[inline]
    fn encode_f64(mut self, value: f64) -> Result<Self::Ok, Self::Error> {
        encode_typed_unsigned(self.writer.borrow_mut(), F64, value.to_bits())
    }

    #[inline]
    fn encode_some(mut self) -> Result<Self::Some, Self::Error> {
        const SOME: Tag = Tag::from_mark(Mark::Some);
        self.writer.write_byte(SOME.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, Self::Error> {
        const NONE: Tag = Tag::from_mark(Mark::None);
        self.writer.write_byte(NONE.byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::Sequence, Self::Error> {
        encode_prefix(self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, len: usize) -> Result<Self::Sequence, Self::Error> {
        encode_prefix(self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::Map, Self::Error> {
        encode_prefix(self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, len: usize) -> Result<Self::Struct, Self::Error> {
        encode_prefix(self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(mut self) -> Result<Self::Variant, Self::Error> {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);
        self.writer.write_byte(VARIANT.byte())?;
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
        self.count = match self.count.checked_add(1) {
            Some(count) => count,
            None => return Err(Self::Error::message("overflow")),
        };

        Ok(StorageEncoder::new(self.pack_buf.borrow_mut()))
    }

    #[inline]
    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        encode_prefix(self.writer.borrow_mut(), Kind::Bytes, self.pack_buf.len())?;
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

/// Encode a length prefix.
#[inline]
fn encode_prefix<W>(mut writer: W, kind: Kind, len: usize) -> Result<(), W::Error>
where
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(kind, len);
    writer.write_byte(tag.byte())?;

    if !embedded {
        Variable::encode_usize(writer, len)?;
    }

    Ok(())
}
