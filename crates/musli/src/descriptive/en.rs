use core::fmt;

use crate::en::{
    Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder,
};
use crate::hint::{MapHint, SequenceHint};
use crate::int::continuation as c;
use crate::storage::en::StorageEncoder;
use crate::writer::BufWriter;
use crate::{Context, Encode, Options, Writer};

use super::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use super::tag::{
    Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, ISIZE, U128, U16, U32, U64, U8, USIZE,
};

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

pub struct SelfPackEncoder<'a, W, const OPT: Options, C>
where
    C: ?Sized + Context,
{
    cx: &'a C,
    writer: W,
    buffer: BufWriter<'a, C::Allocator>,
}

impl<'a, W, const OPT: Options, C> SelfPackEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, writer: W) -> Self {
        Self {
            cx,
            writer,
            buffer: BufWriter::new(cx.alloc()),
        }
    }
}

#[crate::encoder(crate)]
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
    type EncodePack = SelfPackEncoder<'a, W, OPT, C>;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeVariant = Self;
    type EncodeSequenceVariant = Self;
    type EncodeMapVariant = Self;

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
    fn encode_empty(mut self) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_byte(self.cx, Tag::from_mark(Mark::Unit).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        Ok(SelfPackEncoder::new(self.cx, self.writer))
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
    fn encode_sequence(mut self, hint: &SequenceHint) -> Result<Self::EncodeSequence, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Sequence, hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, hint: &MapHint) -> Result<Self::EncodeMap, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Map, hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(mut self, hint: &MapHint) -> Result<Self::EncodeMapEntries, C::Error> {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Map, hint.size)?;
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
        variant.encode_data()?.encode_empty()?;
        VariantEncoder::finish_variant(variant)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence_variant<T>(
        mut self,
        tag: &T,
        hint: &SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer.write_byte(self.cx, VARIANT.byte())?;
        SelfEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_sequence(hint)
    }

    #[inline]
    fn encode_map_variant<T>(
        mut self,
        tag: &T,
        hint: &MapHint,
    ) -> Result<Self::EncodeMapVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer.write_byte(self.cx, VARIANT.byte())?;
        SelfEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_map(hint)
    }
}

impl<'a, W, const OPT: Options, C> SequenceEncoder for SelfPackEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this> = StorageEncoder<'a, &'this mut BufWriter<'a, C::Allocator>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_sequence(mut self) -> Result<Self::Ok, C::Error> {
        let buffer = self.buffer.into_inner();
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), Kind::Bytes, buffer.len())?;
        self.writer.extend(self.cx, buffer)?;
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
    type EncodeNext<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> MapEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> EntryEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeKey<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeValue<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> EntriesEncoder for SelfEncoder<'a, W, OPT, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntryKey<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeEntryValue<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entries(self) -> Result<Self::Ok, C::Error> {
        Ok(())
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
    type EncodeData<'this> = SelfEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(SelfEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, C::Error> {
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
        crate::int::encode_usize::<_, _, OPT>(cx, writer, len)?;
    }

    Ok(())
}
