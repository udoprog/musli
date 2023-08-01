use core::{fmt, marker};

use musli::context::Buffer;
use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;
use musli_common::writer::{BufferWriter, Writer};
use musli_storage::en::StorageEncoder;
use musli_storage::int::Variable;

use crate::error::Error;
use crate::integer_encoding::{WireIntegerEncoding, WireUsizeEncoding};
use crate::tag::{Kind, Tag, MAX_INLINE_LEN};

/// A very simple encoder.
pub struct WireEncoder<W, I, L>
where
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    writer: W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, I, L> WireEncoder<W, I, L>
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

pub struct WirePackEncoder<W, B, I, L>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    writer: W,
    buffer: BufferWriter<B, W::Error>,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, B, I, L> WirePackEncoder<W, B, I, L>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W, buffer: B) -> Self {
        Self {
            writer,
            buffer: BufferWriter::new(buffer),
            _marker: marker::PhantomData,
        }
    }
}

#[musli::encoder]
impl<W, I, L> Encoder for WireEncoder<W, I, L>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;

    type Pack<B> = WirePackEncoder<W, B, I, L> where B: Buffer;
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
    fn encode_unit<C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_pack<C>(self, cx: &mut C) -> Result<Self::Pack<C::Buf>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let mut buffer = cx.alloc();

        if !buffer.write(&[0]) {
            return Err(cx.message("Pack buffer too small"));
        }

        Ok(WirePackEncoder::new(self.writer, buffer))
    }

    #[inline(always)]
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

    #[inline(always)]
    fn encode_bytes<C>(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, L>(cx.adapt(), &mut self.writer, bytes.len())?;
        self.writer.write_bytes(cx.adapt(), bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_bytes_vectored<C>(
        mut self,
        cx: &mut C,
        vectors: &[&[u8]],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix::<_, _, L>(cx.adapt(), &mut self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx.adapt(), bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string<C>(self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_bytes(cx, string.as_bytes())
    }

    #[inline(always)]
    fn encode_usize<C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_typed_usize(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_isize<C>(mut self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_typed_usize(cx.adapt(), &mut self.writer, value as usize)
    }

    #[inline(always)]
    fn encode_bool<C>(mut self, cx: &mut C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(
            cx.adapt(),
            Tag::new(Kind::Continuation, if value { 1 } else { 0 }).byte(),
        )
    }

    #[inline(always)]
    fn encode_char<C>(self, cx: &mut C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u32(cx, value as u32)
    }

    #[inline(always)]
    fn encode_u8<C>(mut self, cx: &mut C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u16<C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u32<C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u64<C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u128<C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_unsigned(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i8<C>(self, cx: &mut C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u8(cx, value as u8)
    }

    #[inline(always)]
    fn encode_i16<C>(mut self, cx: &mut C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i32<C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i64<C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i128<C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_typed_signed(cx.adapt(), &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_f32<C>(self, cx: &mut C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u32(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_f64<C>(self, cx: &mut C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u64(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_some<C>(mut self, cx: &mut C) -> Result<Self::Some, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 1).byte())?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none<C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_map<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let Some(len) = len.checked_mul(2) else {
            return Err(cx.message("Map length overflow"));
        };

        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_struct<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let Some(len) = len.checked_mul(2) else {
            return Err(cx.message("Struct length overflow"));
        };

        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx.adapt(), tag.byte())?;

        if !embedded {
            L::encode_usize(cx.adapt(), &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_variant<C>(mut self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
    }
}

impl<W, B, I, L> SequenceEncoder for WirePackEncoder<W, B, I, L>
where
    W: Writer,
    B: Buffer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = StorageEncoder<&'this mut BufferWriter<B, W::Error>, Variable, Variable, Error> where Self: 'this, B: Buffer;

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
        static PAD: [u8; 1024] = [0; 1024];

        let mut buffer = self.buffer.into_inner();
        let len = buffer.len().wrapping_sub(1);

        let (tag, mut rem) = if len <= MAX_INLINE_LEN {
            (Tag::new(Kind::Prefix, len as u8), 0)
        } else {
            let pow = len.next_power_of_two();
            let rem = pow - len;

            let Ok(pow) = usize::try_from(pow.trailing_zeros()) else {
                return Err(cx.message("Pack too large"));
            };

            if pow > MAX_INLINE_LEN {
                return Err(cx.message("Pack too large"));
            }

            (Tag::new(Kind::Pack, pow as u8), rem)
        };

        if !buffer.write_at(0, &[tag.byte()]) {
            return Err(cx.message("Pack buffer overflow"));
        }

        self.writer.write_buffer(cx.adapt(), buffer)?;

        while rem > 0 {
            let len = rem.min(PAD.len());
            self.writer.write_bytes(cx.adapt(), &PAD[..len])?;
            rem -= len;
        }

        Ok(())
    }
}

impl<W, I, L> SequenceEncoder for WireEncoder<W, I, L>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = WireEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L> PairsEncoder for WireEncoder<W, I, L>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = WireEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L> PairEncoder for WireEncoder<W, I, L>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type First<'this> = WireEncoder<W::Mut<'this>, I, L> where Self: 'this;
    type Second<'this> = WireEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn first<C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second<C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L> VariantEncoder for WireEncoder<W, I, L>
where
    W: Writer,
    Error: From<W::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Ok = ();
    type Error = Error;
    type Tag<'this> = WireEncoder<W::Mut<'this>, I, L> where Self: 'this;
    type Variant<'this> = WireEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
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
fn encode_prefix<C, W, L>(cx: &mut C, writer: &mut W, len: usize) -> Result<(), C::Error>
where
    C: Context<Input = W::Error>,
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
