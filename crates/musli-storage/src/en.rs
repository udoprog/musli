use core::fmt;
use core::marker;

use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;
use musli_common::int::{IntegerEncoding, UsizeEncoding, Variable};
use musli_common::writer::Writer;

/// The alias for a [StorageEncoder] that is used for packs.
pub type PackEncoder<W, E> = StorageEncoder<W, Variable, Variable, E>;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<W, I, L, E>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    writer: W,
    _marker: marker::PhantomData<(I, L, E)>,
}

impl<W, I, L, E> StorageEncoder<W, I, L, E>
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

#[musli::encoder]
impl<W, I, L, E> Encoder for StorageEncoder<W, I, L, E>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
    E: From<W::Error>,
    E: musli::error::Error,
{
    type Ok = ();
    type Error = E;

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

    #[inline(always)]
    fn encode_unit<'buf, C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        SequenceEncoder::end(self.encode_sequence(cx, 0)?, cx)
    }

    #[inline(always)]
    fn encode_pack<'buf, C>(self, _: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline(always)]
    fn encode_array<'buf, C, const N: usize>(
        mut self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_array(cx.adapt(), array)
    }

    #[inline(always)]
    fn encode_bytes<'buf, C>(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_usize(cx.adapt(), self.writer.borrow_mut(), bytes.len())?;
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
        L::encode_usize(cx.adapt(), self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx.adapt(), bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string<'buf, C>(mut self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_usize(cx.adapt(), self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(cx.adapt(), string.as_bytes())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_usize<'buf, C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_usize(cx.adapt(), self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_isize<'buf, C>(self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_usize(cx, value as usize)
    }

    #[inline(always)]
    fn encode_bool<'buf, C>(mut self, cx: &mut C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer
            .write_byte(cx.adapt(), if value { 1 } else { 0 })
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
        self.writer.write_byte(cx.adapt(), value)
    }

    #[inline(always)]
    fn encode_u16<'buf, C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_unsigned(cx.adapt(), self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u32<'buf, C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_unsigned(cx.adapt(), self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u64<'buf, C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_unsigned(cx.adapt(), self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u128<'buf, C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_unsigned(cx.adapt(), self.writer.borrow_mut(), value)
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
        I::encode_signed(cx.adapt(), self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i32<'buf, C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_signed(cx.adapt(), self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i64<'buf, C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_signed(cx.adapt(), self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i128<'buf, C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::encode_signed(cx.adapt(), self.writer.borrow_mut(), value)
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
        self.writer.write_byte(cx.adapt(), 1)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_byte(cx.adapt(), 0)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_sequence<'buf, C>(
        mut self,
        cx: &mut C,
        len: usize,
    ) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_usize(cx.adapt(), self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_tuple<'buf, C>(self, _: &mut C, _: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        // NB: A tuple has statically known fixed length.
        Ok(self)
    }

    #[inline(always)]
    fn encode_map<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_usize(cx.adapt(), self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_struct<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::encode_usize(cx.adapt(), self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_variant<'buf, C>(self, _: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(self)
    }
}

impl<W, I, L, E> SequenceEncoder for StorageEncoder<W, I, L, E>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
    E: From<W::Error>,
    E: musli::error::Error,
{
    type Ok = ();
    type Error = E;
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, I, L, E> where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L, E> PairsEncoder for StorageEncoder<W, I, L, E>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
    E: From<W::Error>,
    E: musli::error::Error,
{
    type Ok = ();
    type Error = E;
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, I, L, E> where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L, E> PairEncoder for StorageEncoder<W, I, L, E>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
    E: From<W::Error>,
    E: musli::error::Error,
{
    type Ok = ();
    type Error = E;
    type First<'this> = StorageEncoder<W::Mut<'this>, I, L, E> where Self: 'this;
    type Second<'this> = StorageEncoder<W::Mut<'this>, I, L, E> where Self: 'this;

    #[inline]
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second<'buf, C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L, E> VariantEncoder for StorageEncoder<W, I, L, E>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
    E: From<W::Error>,
    E: musli::error::Error,
{
    type Ok = ();
    type Error = E;
    type Tag<'this> = StorageEncoder<W::Mut<'this>, I, L, E> where Self: 'this;
    type Variant<'this> = StorageEncoder<W::Mut<'this>, I, L, E> where Self: 'this;

    #[inline]
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<'buf, C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}
