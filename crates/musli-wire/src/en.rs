use core::{fmt, marker};

use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;
use musli_common::fixed_bytes::FixedBytes;
use musli_common::writer::Writer;
use musli_storage::en::StorageEncoder;
use musli_storage::int::Variable;

use crate::error::Error;
use crate::integer_encoding::{WireIntegerEncoding, WireUsizeEncoding};
use crate::tag::{Kind, Tag};

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
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    writer: W,
    pack_buf: FixedBytes<P>,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, I, L, const P: usize> WirePackEncoder<W, I, L, P>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W, pack_buf: FixedBytes<P>) -> Self {
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
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;

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
    fn encode_unit<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_pack<'buf, C>(self, _: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WirePackEncoder::new(self.writer, FixedBytes::new()))
    }

    #[inline(always)]
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

    #[inline(always)]
    fn encode_bytes<'buf, C>(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix::<_, _, L>(cx.adapt(), &mut self.writer, bytes.len())?;
        self.writer.write_bytes(cx.adapt(), bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_bytes_vectored<'buf, C>(
        mut self,
        cx: &mut C,
        vectors: &[&[u8]],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix::<_, _, L>(cx.adapt(), &mut self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx.adapt(), bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string<'buf, C>(self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_bytes(cx, string.as_bytes())
    }

    #[inline(always)]
    fn encode_usize<'buf, C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_typed_usize(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_isize<'buf, C>(mut self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_typed_usize(cx.adapt(), &mut self.writer, value as usize)
    }

    #[inline(always)]
    fn encode_bool<'buf, C>(mut self, cx: &mut C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_byte(
            cx.adapt(),
            Tag::new(Kind::Byte, if value { 1 } else { 0 }).byte(),
        )
    }

    #[inline(always)]
    fn encode_char<'buf, C>(self, cx: &mut C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_u32(cx, value as u32)
    }

    #[inline(always)]
    fn encode_u8<'buf, C>(mut self, cx: &mut C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let (tag, embedded) = Tag::with_byte(Kind::Byte, value);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            self.writer.write_byte(cx.adapt(), value)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_u16<'buf, C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u32<'buf, C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u64<'buf, C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u128<'buf, C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i8<'buf, C>(self, cx: &mut C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_u8(cx, value as u8)
    }

    #[inline(always)]
    fn encode_i16<'buf, C>(mut self, cx: &mut C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i32<'buf, C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i64<'buf, C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i128<'buf, C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_f32<'buf, C>(self, cx: &mut C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_u32(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_f64<'buf, C>(self, cx: &mut C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_u64(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_some<'buf, C>(mut self, cx: &mut C) -> Result<Self::Some, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 1).byte())?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 0).byte())?;
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
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_map<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let len = len.checked_mul(2).ok_or_else(|| cx.message(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_struct<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let len = len.checked_mul(2).ok_or_else(|| cx.message(Overflow))?;
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_variant<'buf, C>(mut self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
    }
}

impl<W, I, L, const P: usize> SequenceEncoder for WirePackEncoder<W, I, L, P>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = StorageEncoder<&'this mut FixedBytes<P>, Variable, Variable, Error> where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageEncoder::new(&mut self.pack_buf))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_prefix::<_, _, L>(cx.adapt(), &mut self.writer, self.pack_buf.len())?;
        self.writer
            .write_bytes(cx.adapt(), self.pack_buf.as_slice())?;
        Ok(())
    }
}

impl<W, I, L, const P: usize> SequenceEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L, const P: usize> PairsEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L, const P: usize> PairEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type First<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;
    type Second<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second<'buf, C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L, const P: usize> VariantEncoder for WireEncoder<W, I, L, P>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Tag<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;
    type Variant<'this> = WireEncoder<W::Mut<'this>, I, L, P> where Self: 'this;

    #[inline]
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<'buf, C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
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
fn encode_prefix<'buf, C, W, L>(cx: &mut C, writer: &mut W, len: usize) -> Result<(), C::Error>
where
    C: Context<'buf, Input = W::Error>,
    W: Writer,
    Error: From<W::Error>,
    L: WireUsizeEncoding,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(cx.adapt(), tag.byte())?;

    if !embedded {
        L::encode_usize(cx.adapt(), writer, len)?;
    }

    Ok(())
}
