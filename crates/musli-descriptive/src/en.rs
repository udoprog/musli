use core::fmt;

use musli::en::{
    Encoder, MapEncoder, MapEntriesEncoder, MapEntryEncoder, PackEncoder, SequenceEncoder,
    StructEncoder, StructFieldEncoder, TupleEncoder, VariantEncoder,
};
use musli::{Buf, Context, Encode};
use musli_common::int::continuation as c;
use musli_storage::en::StorageEncoder;

use crate::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use crate::options::Options;
use crate::tag::{
    Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, ISIZE, MAX_INLINE_LEN, U128, U16, U32, U64,
    U8, USIZE,
};
use crate::writer::{BufWriter, Writer};

const VARIANT: Tag = Tag::from_mark(Mark::Variant);

/// A very simple encoder.
pub struct SelfEncoder<'a, W, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
}

impl<'a, W, const OPT: Options, C: ?Sized> SelfEncoder<'a, W, OPT, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, writer: W) -> Self {
        Self { cx, writer }
    }
}

pub struct SelfPackEncoder<'a, W, B, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
    buffer: BufWriter<B>,
}

impl<'a, W, B, const OPT: Options, C: ?Sized> SelfPackEncoder<'a, W, B, OPT, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, writer: W, buffer: B) -> Self {
        Self {
            cx,
            writer,
            buffer: BufWriter::new(buffer),
        }
    }
}

#[musli::encoder]
impl<'a, W, const OPT: Options, C> Encoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U> = SelfEncoder<'this, W, OPT, U> where U: 'this + Context;
    type EncodePack = SelfPackEncoder<'a, W, C::Buf<'a>, OPT, C>;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeTuple = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeStruct = Self;
    type EncodeVariant = Self;
    type EncodeTupleVariant = Self;
    type EncodeStructVariant = Self;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(SelfEncoder::new(cx, self.writer))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the descriptive encoder")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<Self::Ok, C::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.encode(self.cx, self)
    }

    #[inline]
    fn encode_unit(mut self) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_byte(self.cx, Tag::from_mark(Mark::Unit).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        let Some(buf) = self.cx.alloc() else {
            return Err(self.cx.message("Failed to allocate pack buffer"));
        };

        Ok(SelfPackEncoder::new(self.cx, self.writer, buf))
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: &[u8; N]) -> Result<Self::Ok, C::Error> {
        self.encode_bytes(array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Bytes, bytes.len())?;
        self.writer.write_bytes(self.cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(mut self, len: usize, vectors: I) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Bytes, len)?;

        for bytes in vectors {
            self.writer.write_bytes(self.cx, bytes.as_ref())?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, C::Error> {
        encode_prefix::<_, _, OPT>(
            self.cx,
            self.writer.borrow_mut(),
            Kind::String,
            string.len(),
        )?;
        self.writer.write_bytes(self.cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn collect_string<T>(self, value: &T) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let buf = self.cx.collect_string(value)?;
        self.encode_string(buf.as_ref())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), USIZE, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), ISIZE, value)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, C::Error> {
        const TRUE: Tag = Tag::from_mark(Mark::True);
        const FALSE: Tag = Tag::from_mark(Mark::False);

        self.writer
            .write_byte(self.cx, if value { TRUE } else { FALSE }.byte())
    }

    #[inline]
    fn encode_char(mut self, value: char) -> Result<Self::Ok, C::Error> {
        const CHAR: Tag = Tag::from_mark(Mark::Char);
        self.writer.write_byte(self.cx, CHAR.byte())?;
        c::encode(self.cx, self.writer.borrow_mut(), value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U8, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U16, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U32, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U64, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U128, value)
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I8, value)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I16, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I32, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I64, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, C::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I128, value)
    }

    #[inline]
    fn encode_f32(mut self, value: f32) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), F32, value.to_bits())
    }

    #[inline]
    fn encode_f64(mut self, value: f64) -> Result<Self::Ok, C::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), F64, value.to_bits())
    }

    #[inline]
    fn encode_some(mut self) -> Result<Self::EncodeSome, C::Error> {
        const SOME: Tag = Tag::from_mark(Mark::Some);
        self.writer.write_byte(self.cx, SOME.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, C::Error> {
        const NONE: Tag = Tag::from_mark(Mark::None);
        self.writer.write_byte(self.cx, NONE.byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Sequence, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::EncodeMap, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(mut self, len: usize) -> Result<Self::EncodeMapEntries, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, len: usize) -> Result<Self::EncodeStruct, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Map, len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(mut self) -> Result<Self::EncodeVariant, C::Error> {
        self.writer.write_byte(self.cx, VARIANT.byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_unit_variant<T>(self, tag: &T) -> Result<(), C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = self.encode_variant()?;
        variant.encode_tag()?.encode(tag)?;
        variant.encode_value()?.encode_unit()?;
        VariantEncoder::finish_variant(variant)?;
        Ok(())
    }

    #[inline]
    fn encode_tuple_variant<T>(
        mut self,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer.write_byte(self.cx, VARIANT.byte())?;
        SelfEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_tuple(len)
    }

    #[inline]
    fn encode_struct_variant<T>(
        mut self,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeStructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer.write_byte(self.cx, VARIANT.byte())?;
        SelfEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_struct(len)
    }
}

impl<'a, W, B, const OPT: Options, C> PackEncoder for SelfPackEncoder<'a, W, B, OPT, C>
where
    W: Writer,
    B: Buf,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodePacked<'this> = StorageEncoder<'a, &'this mut BufWriter<B>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_packed(&mut self) -> Result<Self::EncodePacked<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_pack(mut self) -> Result<Self::Ok, C::Error> {
        static PAD: [u8; 1024] = [0; 1024];

        let buffer = self.buffer.into_inner();
        let len = buffer.len();

        let (tag, mut rem) = if len <= MAX_INLINE_LEN {
            (Tag::new(Kind::Bytes, len as u8), 0)
        } else {
            let pow = len.next_power_of_two();
            let rem = pow - len;

            let Ok(pow) = usize::try_from(pow.trailing_zeros()) else {
                return Err(self.cx.message("Pack too large"));
            };

            if pow > MAX_INLINE_LEN {
                return Err(self.cx.message("Pack too large"));
            }

            (Tag::new(Kind::Pack, pow as u8), rem)
        };

        self.writer.write_bytes(self.cx, &[tag.byte()])?;
        self.writer.write_buffer(self.cx, buffer)?;

        while rem > 0 {
            let len = rem.min(PAD.len());
            self.writer.write_bytes(self.cx, &PAD[..len])?;
            rem -= len;
        }

        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> SequenceEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeElement<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_element(&mut self) -> Result<Self::EncodeElement<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> TupleEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeTupleField<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_tuple_field(&mut self) -> Result<Self::EncodeTupleField<'_>, C::Error> {
        SequenceEncoder::encode_element(self)
    }

    #[inline]
    fn finish_tuple(self) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::finish_sequence(self)
    }
}

impl<'a, W, const OPT: Options, C> MapEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntry<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_map_entry(&mut self) -> Result<Self::EncodeMapEntry<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> MapEntryEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeMapValue<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map_entry(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> MapEntriesEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeMapEntryValue<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_entry_value(&mut self) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map_entries(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> StructEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeStructField<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_struct_field(&mut self) -> Result<Self::EncodeStructField<'_>, C::Error> {
        MapEncoder::encode_map_entry(self)
    }

    #[inline]
    fn finish_struct(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> StructFieldEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeFieldName<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeFieldValue<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_field_name(&mut self) -> Result<Self::EncodeFieldName<'_>, C::Error> {
        MapEntryEncoder::encode_map_key(self)
    }

    #[inline]
    fn encode_field_value(&mut self) -> Result<Self::EncodeFieldValue<'_>, C::Error> {
        MapEntryEncoder::encode_map_value(self)
    }

    #[inline]
    fn finish_field(self) -> Result<Self::Ok, C::Error> {
        MapEntryEncoder::finish_map_entry(self)
    }
}

impl<'a, W, const OPT: Options, C> VariantEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeValue<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_variant(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<C, W, const OPT: Options>(
    cx: &C,
    mut writer: W,
    kind: Kind,
    len: usize,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(kind, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        musli_common::int::encode_usize::<_, _, OPT>(cx, writer, len)?;
    }

    Ok(())
}
