use core::fmt;

use musli::context::Buffer;
use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;
use musli_common::int::{continuation as c, UsizeEncoding, Variable};
use musli_common::writer::{BufferWriter, Writer};
use musli_storage::en::StorageEncoder;

use crate::error::Error;
use crate::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use crate::tag::{
    Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, ISIZE, U128, U16, U32, U64, U8, USIZE,
};

/// A very simple encoder.
pub struct SelfEncoder<W> {
    writer: W,
}

impl<W> SelfEncoder<W> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }
}

pub struct SelfPackEncoder<W, B>
where
    W: Writer,
{
    writer: W,
    buffer: BufferWriter<B, W::Error>,
}

impl<W, B> SelfPackEncoder<W, B>
where
    W: Writer,
    Error: From<W::Error>,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W, buffer: B) -> Self {
        Self {
            writer,
            buffer: BufferWriter::new(buffer),
        }
    }
}

#[musli::encoder]
impl<W> Encoder for SelfEncoder<W>
where
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;

    type Pack<B> = SelfPackEncoder<W, B> where B: Buffer;
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
    fn encode_unit<C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::from_mark(Mark::Unit).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack<C>(self, cx: &mut C) -> Result<Self::Pack<C::Buf>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfPackEncoder::new(self.writer, cx.alloc()))
    }

    #[inline]
    fn encode_array<C, const N: usize>(
        self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_bytes(cx, array.as_slice())
    }

    #[inline]
    fn encode_bytes<C>(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Bytes, bytes.len())?;
        self.writer.write_bytes(cx.adapt(), bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<C>(
        mut self,
        cx: &mut C,
        vectors: &[&[u8]],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Bytes, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx.adapt(), bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string<C>(mut self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::String, string.len())?;
        self.writer.write_bytes(cx.adapt(), string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize<C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), USIZE, value)
    }

    #[inline]
    fn encode_isize<C>(mut self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), ISIZE, value)
    }

    #[inline]
    fn encode_bool<C>(mut self, cx: &mut C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const TRUE: Tag = Tag::from_mark(Mark::True);
        const FALSE: Tag = Tag::from_mark(Mark::False);

        self.writer
            .write_byte(cx.adapt(), if value { TRUE } else { FALSE }.byte())
    }

    #[inline]
    fn encode_char<C>(mut self, cx: &mut C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const CHAR: Tag = Tag::from_mark(Mark::Char);
        self.writer.write_byte(cx.adapt(), CHAR.byte())?;
        c::encode(cx.adapt(), self.writer.borrow_mut(), value as u32)
    }

    #[inline]
    fn encode_u8<C>(mut self, cx: &mut C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U8, value)
    }

    #[inline]
    fn encode_u16<C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U16, value)
    }

    #[inline]
    fn encode_u32<C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U32, value)
    }

    #[inline]
    fn encode_u64<C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U64, value)
    }

    #[inline]
    fn encode_u128<C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U128, value)
    }

    #[inline]
    fn encode_i8<C>(mut self, cx: &mut C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I8, value)
    }

    #[inline]
    fn encode_i16<C>(mut self, cx: &mut C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I16, value)
    }

    #[inline]
    fn encode_i32<C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I32, value)
    }

    #[inline]
    fn encode_i64<C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I64, value)
    }

    #[inline]
    fn encode_i128<C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I128, value)
    }

    #[inline]
    fn encode_f32<C>(mut self, cx: &mut C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F32, value.to_bits())
    }

    #[inline]
    fn encode_f64<C>(mut self, cx: &mut C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F64, value.to_bits())
    }

    #[inline]
    fn encode_some<C>(mut self, cx: &mut C) -> Result<Self::Some, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const SOME: Tag = Tag::from_mark(Mark::Some);
        self.writer.write_byte(cx.adapt(), SOME.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none<C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const NONE: Tag = Tag::from_mark(Mark::None);
        self.writer.write_byte(cx.adapt(), NONE.byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant<C>(mut self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);
        self.writer.write_byte(cx.adapt(), VARIANT.byte())?;
        Ok(self)
    }
}

impl<W, B> SequenceEncoder for SelfPackEncoder<W, B>
where
    W: Writer,
    Error: From<W::Error>,
    B: Buffer,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = StorageEncoder<&'this mut BufferWriter<B, W::Error>, Variable, Variable, Error> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(&mut self.buffer))
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let buffer = self.buffer.into_inner();
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Bytes, buffer.len())?;
        self.writer.write_buffer(cx.adapt(), buffer)?;
        Ok(())
    }
}

impl<W> SequenceEncoder for SelfEncoder<W>
where
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = SelfEncoder<W::Mut<'this>> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W> PairsEncoder for SelfEncoder<W>
where
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = SelfEncoder<W::Mut<'this>> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W> PairEncoder for SelfEncoder<W>
where
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;
    type First<'this> = SelfEncoder<W::Mut<'this>> where Self: 'this;
    type Second<'this> = SelfEncoder<W::Mut<'this>> where Self: 'this;

    #[inline]
    fn first<C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second<C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W> VariantEncoder for SelfEncoder<W>
where
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;
    type Tag<'this> = SelfEncoder<W::Mut<'this>> where Self: 'this;
    type Variant<'this> = SelfEncoder<W::Mut<'this>> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<C, W>(cx: &mut C, mut writer: W, kind: Kind, len: usize) -> Result<(), C::Error>
where
    C: Context<Input = Error>,
    W: Writer,
    Error: From<W::Error>,
{
    let (tag, embedded) = Tag::with_len(kind, len);
    writer.write_byte(cx.adapt(), tag.byte())?;

    if !embedded {
        Variable::encode_usize(cx.adapt(), writer, len)?;
    }

    Ok(())
}
