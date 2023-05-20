use core::fmt;
use core::marker;

use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;
use musli_common::int::{IntegerEncoding, UsizeEncoding, Variable};
use musli_common::writer::Writer;

/// The alias for a [StorageEncoder] that is used for packs.
pub type PackEncoder<W> = StorageEncoder<W, Variable, Variable>;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<W, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    writer: W,
    _marker: marker::PhantomData<(I, L)>,
}

impl<W, I, L> StorageEncoder<W, I, L>
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
impl<W, I, L> Encoder for StorageEncoder<W, I, L>
where
    W: Writer,
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

    #[inline(always)]
    fn encode_unit<C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        SequenceEncoder::end(self.encode_sequence(cx, 0)?, cx)
    }

    #[inline(always)]
    fn encode_pack<C>(self, _: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline(always)]
    fn encode_array<C, const N: usize>(
        mut self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_array(cx, array)
    }

    #[inline(always)]
    fn encode_bytes<C>(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_usize(cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
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
        L::encode_usize(cx, self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string<C>(mut self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_usize(cx, self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(cx, string.as_bytes())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_usize<C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_usize(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_isize<C>(self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_usize(cx, value as usize)
    }

    #[inline(always)]
    fn encode_bool<C>(mut self, cx: &mut C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(cx, if value { 1 } else { 0 })
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
        self.writer.write_byte(cx, value)
    }

    #[inline(always)]
    fn encode_u16<C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_unsigned(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u32<C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_unsigned(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u64<C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_unsigned(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u128<C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_unsigned(cx, self.writer.borrow_mut(), value)
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
        I::encode_signed(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i32<C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_signed(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i64<C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_signed(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i128<C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::encode_signed(cx, self.writer.borrow_mut(), value)
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
        self.writer.write_byte(cx, 1)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none<C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(cx, 0)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_sequence<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_usize(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_tuple<C>(self, _: &mut C, _: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        // NB: A tuple has statically known fixed length.
        Ok(self)
    }

    #[inline(always)]
    fn encode_map<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_usize(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_struct<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::encode_usize(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_variant<C>(self, _: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }
}

impl<W, I, L> SequenceEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L> PairsEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L> PairEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type First<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;
    type Second<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn first<C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second<C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, I, L> VariantEncoder for StorageEncoder<W, I, L>
where
    W: Writer,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Ok = ();
    type Error = W::Error;
    type Tag<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;
    type Variant<'this> = StorageEncoder<W::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}
