use core::fmt;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntryEncoder, MapPairsEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use musli::{Buf, Context};
use musli_storage::en::StorageEncoder;

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
        C: ?Sized + Context,
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
        C: ?Sized + Context,
    {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx, tag.byte())?;

        if !embedded {
            crate::int::encode_usize::<_, _, F>(cx, &mut self.writer, len)?;
        }

        Ok(())
    }
}

pub struct WirePackEncoder<W, B, const F: Options> {
    writer: W,
    buffer: BufWriter<B>,
}

impl<W, B, const F: Options> WirePackEncoder<W, B, F> {
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
impl<C, W, const F: Options> Encoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type WithContext<U> = Self where U: Context;
    type EncodePack<'this> = WirePackEncoder<W, C::Buf<'this>, F> where C: 'this;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeTuple = Self;
    type EncodeMap = Self;
    type EncodeMapPairs = Self;
    type EncodeStruct = Self;
    type EncodeVariant = Self;
    type EncodeTupleVariant = Self;
    type EncodeStructVariant = Self;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire encoder")
    }

    #[inline]
    fn encode_unit(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self, cx: &C) -> Result<Self::EncodePack<'_>, C::Error> {
        let Some(buf) = cx.alloc() else {
            return Err(cx.message("Failed to allocate pack buffer"));
        };

        Ok(WirePackEncoder::new(self.writer, buf))
    }

    #[inline]
    fn encode_array<const N: usize>(self, cx: &C, array: [u8; N]) -> Result<Self::Ok, C::Error> {
        self.encode_bytes(cx, array.as_slice())
    }

    #[inline]
    fn encode_bytes(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        encode_prefix::<_, _, F>(cx, &mut self.writer, bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, cx: &C, vectors: &[&[u8]]) -> Result<Self::Ok, C::Error> {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix::<_, _, F>(cx, &mut self.writer, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(self, cx: &C, string: &str) -> Result<Self::Ok, C::Error> {
        self.encode_bytes(cx, string.as_bytes())
    }

    #[inline]
    fn encode_usize(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_length::<_, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_isize(mut self, cx: &C, value: isize) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_length::<_, _, F>(cx, &mut self.writer, value as usize)
    }

    #[inline]
    fn encode_bool(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(
            cx,
            Tag::new(Kind::Continuation, if value { 1 } else { 0 }).byte(),
        )
    }

    #[inline]
    fn encode_char(self, cx: &C, value: char) -> Result<Self::Ok, C::Error> {
        self.encode_u32(cx, value as u32)
    }

    #[inline]
    fn encode_u8(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_u16(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_u32(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_u64(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_u128(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_i8(self, cx: &C, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(cx, value as u8)
    }

    #[inline]
    fn encode_i16(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_i32(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_i64(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_i128(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(cx, &mut self.writer, value)
    }

    #[inline]
    fn encode_f32(self, cx: &C, value: f32) -> Result<Self::Ok, C::Error> {
        self.encode_u32(cx, value.to_bits())
    }

    #[inline]
    fn encode_f64(self, cx: &C, value: f64) -> Result<Self::Ok, C::Error> {
        self.encode_u64(cx, value.to_bits())
    }

    #[inline]
    fn encode_some(mut self, cx: &C) -> Result<Self::EncodeSome, C::Error> {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 1).byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, cx: &C, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(cx, tag.byte())?;

        if !embedded {
            crate::int::encode_usize::<_, _, F>(cx, &mut self.writer, len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, cx: &C, len: usize) -> Result<Self::EncodeTuple, C::Error> {
        self.encode_tuple_len(cx, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, cx: &C, len: usize) -> Result<Self::EncodeMap, C::Error> {
        self.encode_map_len(cx, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_pairs(self, cx: &C, len: usize) -> Result<Self::EncodeMapPairs, C::Error> {
        self.encode_map(cx, len)
    }

    #[inline]
    fn encode_struct(mut self, cx: &C, len: usize) -> Result<Self::EncodeStruct, C::Error> {
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
    fn encode_variant(mut self, cx: &C) -> Result<Self::EncodeVariant, C::Error> {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 2).byte())?;
        tag.encode(cx, WireEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_tuple(cx, len)
    }

    #[inline]
    fn encode_struct_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(cx, Tag::new(Kind::Sequence, 2).byte())?;
        tag.encode(cx, WireEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_struct(cx, len)
    }
}

impl<C, W, B, const F: Options> SequenceEncoder<C> for WirePackEncoder<W, B, F>
where
    C: ?Sized + Context,
    W: Writer,
    B: Buf,
{
    type Ok = ();
    type EncodeNext<'this> = StorageEncoder<&'this mut BufWriter<B>, F> where Self: 'this, B: Buf;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(&mut self.buffer))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
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

impl<C, W, const F: Options> SequenceEncoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type EncodeNext<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C, W, const F: Options> MapEncoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type Entry<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn entry(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C, W, const F: Options> MapPairsEncoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type MapPairsKey<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapPairsValue<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C, W, const F: Options> MapEntryEncoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type MapKey<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapValue<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_value(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C, W, const F: Options> StructEncoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type Field<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field(&mut self, cx: &C) -> Result<Self::Field<'_>, C::Error> {
        MapEncoder::entry(self, cx)
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C, W, const F: Options> StructFieldEncoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type FieldName<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type FieldValue<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        self.map_key(cx)
    }

    #[inline]
    fn field_value(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        self.map_value(cx)
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C, W, const F: Options> VariantEncoder<C> for WireEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type EncodeTag<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;
    type EncodeValue<'this> = WireEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self, _: &C) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self, _: &C) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<C, W, const F: Options>(cx: &C, writer: &mut W, len: usize) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        crate::int::encode_usize::<_, _, F>(cx, writer, len)?;
    }

    Ok(())
}
