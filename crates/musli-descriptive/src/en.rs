use core::fmt;

use musli::en::{
    Encoder, MapEncoder, MapEntryEncoder, MapPairsEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use musli::{Buf, Context, Encode};
use musli_storage::en::StorageEncoder;

use crate::error::Error;
use crate::int::continuation as c;
use crate::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use crate::options::Options;
use crate::tag::{
    Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, ISIZE, MAX_INLINE_LEN, U128, U16, U32, U64,
    U8, USIZE,
};
use crate::writer::{BufWriter, Writer};

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

pub struct SelfPackEncoder<W, B, const F: Options>
where
    W: Writer,
{
    writer: W,
    buffer: BufWriter<B>,
}

impl<W, B, const F: Options> SelfPackEncoder<W, B, F>
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
impl<W, const F: Options> Encoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;

    type Pack<'this, C> = SelfPackEncoder<W, C::Buf<'this>, F> where C: 'this + Context;
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
        write!(f, "type supported by the descriptive encoder")
    }

    #[inline]
    fn encode_unit<C>(mut self, cx: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer
            .write_byte(cx, Tag::from_mark(Mark::Unit).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack<C>(self, cx: &C) -> Result<Self::Pack<'_, C>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let Some(buf) = cx.alloc() else {
            return Err(cx.message("Failed to allocate pack buffer"));
        };

        Ok(SelfPackEncoder::new(self.writer, buf))
    }

    #[inline]
    fn encode_array<C, const N: usize>(self, cx: &C, array: [u8; N]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_bytes(cx, array.as_slice())
    }

    #[inline]
    fn encode_bytes<C>(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Bytes, bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<C>(mut self, cx: &C, vectors: &[&[u8]]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let len = vectors.iter().map(|v| v.len()).sum();
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Bytes, len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string<C>(mut self, cx: &C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::String, string.len())?;
        self.writer.write_bytes(cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize<C>(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), USIZE, value)
    }

    #[inline]
    fn encode_isize<C>(mut self, cx: &C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), ISIZE, value)
    }

    #[inline]
    fn encode_bool<C>(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const TRUE: Tag = Tag::from_mark(Mark::True);
        const FALSE: Tag = Tag::from_mark(Mark::False);

        self.writer
            .write_byte(cx, if value { TRUE } else { FALSE }.byte())
    }

    #[inline]
    fn encode_char<C>(mut self, cx: &C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const CHAR: Tag = Tag::from_mark(Mark::Char);
        self.writer.write_byte(cx, CHAR.byte())?;
        c::encode(cx, self.writer.borrow_mut(), value as u32)
    }

    #[inline]
    fn encode_u8<C>(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U8, value)
    }

    #[inline]
    fn encode_u16<C>(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U16, value)
    }

    #[inline]
    fn encode_u32<C>(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U32, value)
    }

    #[inline]
    fn encode_u64<C>(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U64, value)
    }

    #[inline]
    fn encode_u128<C>(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), U128, value)
    }

    #[inline]
    fn encode_i8<C>(mut self, cx: &C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I8, value)
    }

    #[inline]
    fn encode_i16<C>(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I16, value)
    }

    #[inline]
    fn encode_i32<C>(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I32, value)
    }

    #[inline]
    fn encode_i64<C>(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I64, value)
    }

    #[inline]
    fn encode_i128<C>(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_signed(cx, self.writer.borrow_mut(), I128, value)
    }

    #[inline]
    fn encode_f32<C>(mut self, cx: &C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F32, value.to_bits())
    }

    #[inline]
    fn encode_f64<C>(mut self, cx: &C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_typed_unsigned(cx, self.writer.borrow_mut(), F64, value.to_bits())
    }

    #[inline]
    fn encode_some<C>(mut self, cx: &C) -> Result<Self::Some, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const SOME: Tag = Tag::from_mark(Mark::Some);
        self.writer.write_byte(cx, SOME.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none<C>(mut self, cx: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const NONE: Tag = Tag::from_mark(Mark::None);
        self.writer.write_byte(cx, NONE.byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence<C>(mut self, cx: &C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple<C>(mut self, cx: &C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map<C>(mut self, cx: &C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_pairs<C>(mut self, cx: &C, len: usize) -> Result<Self::MapPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct<C>(mut self, cx: &C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        encode_prefix::<_, _, F>(cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant<C>(mut self, cx: &C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);
        self.writer.write_byte(cx, VARIANT.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_unit_variant<C, T>(self, cx: &C, tag: &T) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
        T: Encode<C::Mode>,
    {
        let mut variant = self.encode_variant(cx)?;
        tag.encode(cx, variant.tag(cx)?)?;
        variant.variant(cx)?.encode_unit(cx)?;
        VariantEncoder::end(variant, cx)?;
        Ok(())
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
        T: Encode<C::Mode>,
    {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);
        self.writer.write_byte(cx, VARIANT.byte())?;
        tag.encode(cx, SelfEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_tuple(cx, len)
    }

    #[inline]
    fn encode_struct_variant<C, T>(
        mut self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::StructVariant, C::Error>
    where
        C: Context<Input = Self::Error>,
        T: Encode<C::Mode>,
    {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);
        self.writer.write_byte(cx, VARIANT.byte())?;
        tag.encode(cx, SelfEncoder::<_, F>::new(self.writer.borrow_mut()))?;
        self.encode_struct(cx, len)
    }
}

impl<W, B, const F: Options> SequenceEncoder for SelfPackEncoder<W, B, F>
where
    W: Writer,
    B: Buf,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = StorageEncoder<&'this mut BufWriter<B>, F, Error> where Self: 'this;

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

impl<W, const F: Options> SequenceEncoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Encoder<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> MapEncoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Entry<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn entry<C>(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> MapEntryEncoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type MapKey<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapValue<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_key<C>(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_value<C>(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> MapPairsEncoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type MapPairsKey<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapPairsValue<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_pairs_key<C>(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_pairs_value<C>(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options> StructEncoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Field<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

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

impl<W, const F: Options> StructFieldEncoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type FieldName<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type FieldValue<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

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

impl<W, const F: Options> VariantEncoder for SelfEncoder<W, F>
where
    W: Writer,
{
    type Ok = ();
    type Error = Error;
    type Tag<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;
    type Variant<'this> = SelfEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(SelfEncoder::new(self.writer.borrow_mut()))
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
fn encode_prefix<C, W, const F: Options>(
    cx: &C,
    mut writer: W,
    kind: Kind,
    len: usize,
) -> Result<(), C::Error>
where
    C: Context<Input = Error>,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(kind, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        crate::int::encode_usize::<_, _, F>(cx, writer, len)?;
    }

    Ok(())
}
