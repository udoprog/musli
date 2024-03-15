use core::fmt;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntryEncoder, MapPairsEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use musli::{Buf, Context};
use musli_storage::en::StorageEncoder;

use crate::error::Error;
use crate::options::Options;
use crate::tag::{Kind, Tag, MAX_INLINE_LEN};
use crate::writer::{BufWriter, Writer};

/// A very simple encoder.
pub struct WireEncoder<W, const F: Options> {
    writer: W,
}

impl<W, const F: Options> WireEncoder<W, F>
where
    W: Writer,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }

    #[inline]
    fn encode_map_len<C>(&mut self, cx: &C, len: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        let Some(len) = len.checked_mul(2) else {
            return Err(cx.message("Map length overflow"));
        };

        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx, tag.byte())?;

        if !embedded {
            crate::int::encode_usize::<_, _, F>(cx, &mut self.writer, len)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_tuple_len<C>(&mut self, cx: &C, len: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx, tag.byte())?;

        if !embedded {
            crate::int::encode_usize::<_, _, F>(cx, &mut self.writer, len)?;
        }

        Ok(())
    }
}

pub struct WirePackEncoder<W, B, const F: Options>
where
    W: Writer,
{
    writer: W,
    buffer: BufWriter<B>,
}

impl<W, B, const F: Options> WirePackEncoder<W, B, F>
where
    W: Writer,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W, buffer: B) -> Self {
        Self {
            writer,
            buffer: BufWriter::new(buffer),
        }
    }
}

#[musli::encoder]
impl<W, const F: Options> Encoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;

    type Pack<'this, C> = WirePackEncoder<W, C::Buf<'this>, F> where C: 'this + Context;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type MapPairs = Self;
    type Struct = Self;
    type Variant = Self;
    type TupleVariant = Self;
    type StructVariant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire encoder")
    }

    #[inline(always)]
    fn encode_unit<C>(mut self, cx: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_pack<C>(self, cx: &C) -> Result<Self::Pack<'_, C>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let Some(buf) = cx.alloc() else {
            return Err(cx.message("Failed to allocate pack buffer"));
        };

        Ok(WirePackEncoder::new(self.writer, buf))
    }

    #[inline(always)]
    fn encode_array<C, const N: usize>(self, cx: &C, array: [u8; N]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_bytes(cx, array.as_slice())
    }

    #[inline(always)]
    fn encode_bytes<C>(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, &mut self.writer, bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_bytes_vectored<C>(mut self, cx: &C, vectors: &[&[u8]]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix::<_, _, F>(cx, &mut self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string<C>(self, cx: &C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_bytes(cx, string.as_bytes())
    }

    #[inline(always)]
    fn encode_usize<C>(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_length::<_, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_isize<C>(mut self, cx: &C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_length::<_, _, F>(cx, &mut self.writer, value as usize)
    }

    #[inline(always)]
    fn encode_bool<C>(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(
            cx,
            Tag::new(Kind::Continuation, if value { 1 } else { 0 }).byte(),
        )
    }

    #[inline(always)]
    fn encode_char<C>(self, cx: &C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u32(cx, value as u32)
    }

    #[inline(always)]
    fn encode_u8<C>(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u16<C>(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u32<C>(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u64<C>(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_u128<C>(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i8<C>(self, cx: &C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u8(cx, value as u8)
    }

    #[inline(always)]
    fn encode_i16<C>(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i32<C>(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i64<C>(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_i128<C>(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline(always)]
    fn encode_f32<C>(self, cx: &C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u32(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_f64<C>(self, cx: &C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u64(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_some<C>(mut self, cx: &C) -> Result<Self::Some, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 1).byte())?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none<C>(mut self, cx: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence<C>(mut self, cx: &C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx, tag.byte())?;

        if !embedded {
            crate::int::encode_usize::<_, _, F>(cx, &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple<C>(mut self, cx: &C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_tuple_len(cx, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map<C>(mut self, cx: &C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_map_len(cx, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_pairs<C>(self, cx: &C, len: usize) -> Result<Self::MapPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_map(cx, len)
    }

    #[inline]
    fn encode_struct<C>(mut self, cx: &C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let Some(len) = len.checked_mul(2) else {
            return Err(cx.message("Struct length overflow"));
        };

        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx, tag.byte())?;

        if !embedded {
            crate::int::encode_usize::<_, _, F>(cx, &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_variant<C>(mut self, cx: &C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple_variant<C, T>(
        mut self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::TupleVariant, C::Error>
    where
        C: Context<Input = Self::Error>,
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 2).byte())?;
        tag.encode(cx, WireEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_tuple(cx, len)
    }

    #[inline]
    fn encode_struct_variant<C, T>(
        mut self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::TupleVariant, C::Error>
    where
        C: Context<Input = Self::Error>,
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 2).byte())?;
        tag.encode(cx, WireEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_struct(cx, len)
    }
}

impl<W, B, const F: Options> SequenceEncoder for WirePackEncoder<W, B, F>
where
    W: Writer,
    B: Buf,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = StorageEncoder<&'this mut BufWriter<B>, F, Error> where Self: 'this, B: Buf;

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(&mut self.buffer))
    }

    #[inline]
    fn end<C>(mut self, cx: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        static PAD: [u8; 1024] = [0; 1024];

        let buffer = self.buffer.into_inner();
        let len = buffer.len();

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

        self.writer.write_bytes(cx, &[tag.byte()])?;
        self.writer.write_buffer(cx, buffer)?;

        while rem > 0 {
            let len = rem.min(PAD.len());
            self.writer.write_bytes(cx, &PAD[..len])?;
            rem -= len;
        }

        Ok(())
    }
}

impl<W, const F: Options> SequenceEncoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> MapEncoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Entry<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn entry<C>(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> MapPairsEncoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type MapPairsKey<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapPairsValue<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_pairs_key<C>(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_pairs_value<C>(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> MapEntryEncoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type MapKey<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapValue<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_key<C>(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_value<C>(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> StructEncoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Field<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field<C>(&mut self, cx: &C) -> Result<Self::Field<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEncoder::entry(self, cx)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> StructFieldEncoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type FieldName<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type FieldValue<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.map_key(cx)
    }

    #[inline]
    fn field_value<C>(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.map_value(cx)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> VariantEncoder for WireEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Tag<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type Variant<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<C, W, const F: Options>(cx: &C, writer: &mut W, len: usize) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        crate::int::encode_usize::<_, _, F>(cx, writer, len)?;
    }

    Ok(())
}
