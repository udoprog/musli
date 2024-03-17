use core::fmt;

use musli::en::{
    Encoder, MapEncoder, MapEntryEncoder, MapPairsEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use musli::{Buf, Context, Encode};
use musli_storage::en::StorageEncoder;

use crate::int::continuation as c;
use crate::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use crate::options::Options;
use crate::tag::{
    Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, ISIZE, MAX_INLINE_LEN, U128, U16, U32, U64,
    U8, USIZE,
};
use crate::writer::{BufWriter, Writer};

const VARIANT: Tag = Tag::from_mark(Mark::Variant);

/// A very simple encoder.
pub struct SelfEncoder<W, const F: Options> {
    writer: W,
}

impl<W, const F: Options> SelfEncoder<W, F> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }
}

pub struct SelfPackEncoder<W, B, const F: Options> {
    writer: W,
    buffer: BufWriter<B>,
}

impl<W, B, const F: Options> SelfPackEncoder<W, B, F> {
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
impl<C: ?Sized + Context, W, const F: Options> Encoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type WithContext<U> = Self where U: Context;
    type EncodePack<'this> = SelfPackEncoder<W, C::Buf<'this>, F> where C: 'this;
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
        write!(f, "type supported by the descriptive encoder")
    }

    #[inline]
    fn encode_unit(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_byte(cx, Tag::from_mark(Mark::Unit).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self, cx: &C) -> Result<Self::EncodePack<'_>, C::Error> {
        let Some(buf) = cx.alloc() else {
            return Err(cx.message("Failed to allocate pack buffer"));
        };

        Ok(SelfPackEncoder::new(self.writer, buf))
    }

    #[inline]
    fn encode_array<const N: usize>(self, cx: &C, array: [u8; N]) -> Result<Self::Ok, C::Error> {
        self.encode_bytes(cx, array.as_slice())
    }

    #[inline]
    fn encode_bytes(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Bytes, bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(mut self, cx: &C, vectors: &[&[u8]]) -> Result<Self::Ok, C::Error> {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Bytes, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, cx: &C, string: &str) -> Result<Self::Ok, C::Error> {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::String, string.len())?;
        self.writer.write_bytes(cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), USIZE, value)
    }

    #[inline]
    fn encode_isize(mut self, cx: &C, value: isize) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(cx, self.writer.borrow_mut(), ISIZE, value)
    }

    #[inline]
    fn encode_bool(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error> {
        const TRUE: Tag = Tag::from_mark(Mark::True);
        const FALSE: Tag = Tag::from_mark(Mark::False);

        self.writer
            .write_byte(cx, if value { TRUE } else { FALSE }.byte())
    }

    #[inline]
    fn encode_char(mut self, cx: &C, value: char) -> Result<Self::Ok, C::Error> {
        const CHAR: Tag = Tag::from_mark(Mark::Char);
        self.writer.write_byte(cx, CHAR.byte())?;
        c::encode(cx, self.writer.borrow_mut(), value as u32)
    }

    #[inline]
    fn encode_u8(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U8, value)
    }

    #[inline]
    fn encode_u16(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U16, value)
    }

    #[inline]
    fn encode_u32(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U32, value)
    }

    #[inline]
    fn encode_u64(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U64, value)
    }

    #[inline]
    fn encode_u128(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U128, value)
    }

    #[inline]
    fn encode_i8(mut self, cx: &C, value: i8) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(cx, self.writer.borrow_mut(), I8, value)
    }

    #[inline]
    fn encode_i16(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(cx, self.writer.borrow_mut(), I16, value)
    }

    #[inline]
    fn encode_i32(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(cx, self.writer.borrow_mut(), I32, value)
    }

    #[inline]
    fn encode_i64(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(cx, self.writer.borrow_mut(), I64, value)
    }

    #[inline]
    fn encode_i128(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(cx, self.writer.borrow_mut(), I128, value)
    }

    #[inline]
    fn encode_f32(mut self, cx: &C, value: f32) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F32, value.to_bits())
    }

    #[inline]
    fn encode_f64(mut self, cx: &C, value: f64) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F64, value.to_bits())
    }

    #[inline]
    fn encode_some(mut self, cx: &C) -> Result<Self::EncodeSome, C::Error> {
        const SOME: Tag = Tag::from_mark(Mark::Some);
        self.writer.write_byte(cx, SOME.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        const NONE: Tag = Tag::from_mark(Mark::None);
        self.writer.write_byte(cx, NONE.byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, cx: &C, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, cx: &C, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, cx: &C, len: usize) -> Result<Self::EncodeMap, C::Error> {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_pairs(mut self, cx: &C, len: usize) -> Result<Self::EncodeMapPairs, C::Error> {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, cx: &C, len: usize) -> Result<Self::EncodeStruct, C::Error> {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(mut self, cx: &C) -> Result<Self::EncodeVariant, C::Error> {
        self.writer.write_byte(cx, VARIANT.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_unit_variant<T>(self, cx: &C, tag: &T) -> Result<(), C::Error>
    where
        T: Encode<C::Mode>,
    {
        let mut variant = self.encode_variant(cx)?;
        tag.encode(cx, variant.encode_tag(cx)?)?;
        variant.encode_value(cx)?.encode_unit(cx)?;
        VariantEncoder::end(variant, cx)?;
        Ok(())
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
        self.writer.write_byte(cx, VARIANT.byte())?;
        tag.encode(cx, SelfEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_tuple(cx, len)
    }

    #[inline]
    fn encode_struct_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeStructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer.write_byte(cx, VARIANT.byte())?;
        tag.encode(cx, SelfEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_struct(cx, len)
    }
}

impl<C: ?Sized + Context, W, B, const F: Options> SequenceEncoder<C> for SelfPackEncoder<W, B, F>
where
    W: Writer,
    B: Buf,
{
    type Ok = ();
    type EncodeNext<'this> = StorageEncoder<&'this mut BufWriter<B>, F> where Self: 'this;

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
            (Tag::new(Kind::Bytes, len as u8), 0)
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

impl<C: ?Sized + Context, W, const F: Options> SequenceEncoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type EncodeNext<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C: ?Sized + Context, W, const F: Options> MapEncoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Entry<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn entry(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C: ?Sized + Context, W, const F: Options> MapEntryEncoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type MapKey<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapValue<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_value(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C: ?Sized + Context, W, const F: Options> MapPairsEncoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type MapPairsKey<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapPairsValue<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C: ?Sized + Context, W, const F: Options> StructEncoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Field<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field(&mut self, cx: &C) -> Result<Self::Field<'_>, C::Error> {
        MapEncoder::entry(self, cx)
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<C: ?Sized + Context, W, const F: Options> StructFieldEncoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type FieldName<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type FieldValue<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

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

impl<C: ?Sized + Context, W, const F: Options> VariantEncoder<C> for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type EncodeTag<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type EncodeValue<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self, _: &C) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self, _: &C) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<C: ?Sized + Context, W, const F: Options>(
    cx: &C,
    mut writer: W,
    kind: Kind,
    len: usize,
) -> Result<(), C::Error>
where
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(kind, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        crate::int::encode_usize::<_, _, F>(cx, writer, len)?;
    }

    Ok(())
}
