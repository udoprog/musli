use core::fmt;

use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;
use musli_common::fixed_bytes::FixedBytes;
use musli_common::int::{continuation as c, UsizeEncoding, Variable};
use musli_common::writer::Writer;
use musli_storage::en::StorageEncoder;

use crate::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use crate::tag::{
    Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, ISIZE, U128, U16, U32, U64, U8, USIZE,
};

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

#[musli::encoder]
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
    fn encode_unit<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer
            .write_byte(cx, Tag::from_mark(Mark::Unit).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack<'buf, C>(self, _: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SelfPackEncoder::new(self.writer, FixedBytes::new()))
    }

    #[inline]
    fn encode_array<'buf, C, const N: usize>(
        self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_bytes(cx, array.as_slice())
    }

    #[inline]
    fn encode_bytes<'buf, C>(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Bytes, bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<'buf, C>(
        mut self,
        cx: &mut C,
        vectors: &[&[u8]],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Bytes, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string<'buf, C>(mut self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::String, string.len())?;
        self.writer.write_bytes(cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize<'buf, C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), USIZE, value)
    }

    #[inline]
    fn encode_isize<'buf, C>(mut self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), ISIZE, value)
    }

    #[inline]
    fn encode_bool<'buf, C>(mut self, cx: &mut C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        const TRUE: Tag = Tag::from_mark(Mark::True);
        const FALSE: Tag = Tag::from_mark(Mark::False);

        self.writer
            .write_byte(cx, if value { TRUE } else { FALSE }.byte())
    }

    #[inline]
    fn encode_char<'buf, C>(mut self, cx: &mut C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        const CHAR: Tag = Tag::from_mark(Mark::Char);
        self.writer.write_byte(cx, CHAR.byte())?;
        c::encode(cx, self.writer.borrow_mut(), value as u32)
    }

    #[inline]
    fn encode_u8<'buf, C>(mut self, cx: &mut C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U8, value)
    }

    #[inline]
    fn encode_u16<'buf, C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U16, value)
    }

    #[inline]
    fn encode_u32<'buf, C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U32, value)
    }

    #[inline]
    fn encode_u64<'buf, C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U64, value)
    }

    #[inline]
    fn encode_u128<'buf, C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U128, value)
    }

    #[inline]
    fn encode_i8<'buf, C>(mut self, cx: &mut C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I8, value)
    }

    #[inline]
    fn encode_i16<'buf, C>(mut self, cx: &mut C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I16, value)
    }

    #[inline]
    fn encode_i32<'buf, C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I32, value)
    }

    #[inline]
    fn encode_i64<'buf, C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I64, value)
    }

    #[inline]
    fn encode_i128<'buf, C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I128, value)
    }

    #[inline]
    fn encode_f32<'buf, C>(mut self, cx: &mut C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F32, value.to_bits())
    }

    #[inline]
    fn encode_f64<'buf, C>(mut self, cx: &mut C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F64, value.to_bits())
    }

    #[inline]
    fn encode_some<'buf, C>(mut self, cx: &mut C) -> Result<Self::Some, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        const SOME: Tag = Tag::from_mark(Mark::Some);
        self.writer.write_byte(cx, SOME.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        const NONE: Tag = Tag::from_mark(Mark::None);
        self.writer.write_byte(cx, NONE.byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence<'buf, C>(
        mut self,
        cx: &mut C,
        len: usize,
    ) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant<'buf, C>(mut self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);
        self.writer.write_byte(cx, VARIANT.byte())?;
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
    fn next<'buf, C>(&mut self, cx: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.count = match self.count.checked_add(1) {
            Some(count) => count,
            None => return Err(cx.message("overflow")),
        };

        Ok(StorageEncoder::new(self.pack_buf.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix(
            cx,
            self.writer.borrow_mut(),
            Kind::Bytes,
            self.pack_buf.len(),
        )?;
        self.writer.write_bytes(cx, self.pack_buf.as_slice())?;
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
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
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
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
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
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second<'buf, C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
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
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<'buf, C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<'buf, C, W>(
    cx: &mut C,
    mut writer: W,
    kind: Kind,
    len: usize,
) -> Result<(), C::Error>
where
    C: Context<'buf, Input = W::Error>,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(kind, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        Variable::encode_usize(cx, writer, len)?;
    }

    Ok(())
}
