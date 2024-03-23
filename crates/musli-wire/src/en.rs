use core::fmt;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntriesEncoder, MapEntryEncoder, SequenceEncoder,
    StructEncoder, StructFieldEncoder, VariantEncoder,
};
use musli::{Buf, Context};
use musli_storage::en::StorageEncoder;

use crate::options::Options;
use crate::tag::{Kind, Tag, MAX_INLINE_LEN};
use crate::writer::{BufWriter, Writer};

/// A very simple encoder.
pub struct WireEncoder<'a, W, const F: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
}

impl<'a, W, const F: Options, C> WireEncoder<'a, W, F, C>
where
    W: Writer,
    C: ?Sized + Context,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, writer: W) -> Self {
        Self { cx, writer }
    }

    #[inline]
    fn encode_map_len(&mut self, len: usize) -> Result<(), C::Error> {
        let Some(len) = len.checked_mul(2) else {
            return Err(self.cx.message("Map length overflow"));
        };

        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_tuple_len(&mut self, len: usize) -> Result<(), C::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }
}

pub struct WirePackEncoder<'a, W, B, const F: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
    buffer: BufWriter<B>,
}

impl<'a, W, B, const F: Options, C: ?Sized> WirePackEncoder<'a, W, B, F, C> {
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
impl<'a, W, const F: Options, C> Encoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U> = WireEncoder<'this, W, F, U> where U: 'this + Context;
    type EncodePack = WirePackEncoder<'a, W, C::Buf<'a>, F, C>;
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
        Ok(WireEncoder::new(cx, self.writer))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire encoder")
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
            .write_byte(self.cx, Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        let Some(buf) = self.cx.alloc() else {
            return Err(self.cx.message("Failed to allocate pack buffer"));
        };

        Ok(WirePackEncoder::new(self.cx, self.writer, buf))
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: &[u8; N]) -> Result<Self::Ok, C::Error> {
        self.encode_bytes(array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        encode_prefix::<_, _, F>(self.cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(self.cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(mut self, len: usize, vectors: I) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        encode_prefix::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(self.cx, bytes.as_ref())?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(self, string: &str) -> Result<Self::Ok, C::Error> {
        self.encode_bytes(string.as_bytes())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_length::<_, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_length::<_, _, F>(self.cx, self.writer.borrow_mut(), value as usize)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(
            self.cx,
            Tag::new(Kind::Continuation, if value { 1 } else { 0 }).byte(),
        )
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, C::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<Self::Ok, C::Error> {
        self.encode_u32(value.to_bits())
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<Self::Ok, C::Error> {
        self.encode_u64(value.to_bits())
    }

    #[inline]
    fn encode_some(mut self) -> Result<Self::EncodeSome, C::Error> {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 1).byte())?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, len: usize) -> Result<Self::EncodeTuple, C::Error> {
        self.encode_tuple_len(len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::EncodeMap, C::Error> {
        self.encode_map_len(len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(self, len: usize) -> Result<Self::EncodeMapEntries, C::Error> {
        self.encode_map(len)
    }

    #[inline]
    fn encode_struct(mut self, len: usize) -> Result<Self::EncodeStruct, C::Error> {
        let Some(len) = len.checked_mul(2) else {
            return Err(self.cx.message("Struct length overflow"));
        };

        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_variant(mut self) -> Result<Self::EncodeVariant, C::Error> {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
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
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        WireEncoder::<_, F, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_tuple(len)
    }

    #[inline]
    fn encode_struct_variant<T>(
        mut self,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        WireEncoder::<_, F, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_struct(len)
    }
}

impl<'a, W, B, const F: Options, C> SequenceEncoder for WirePackEncoder<'a, W, B, F, C>
where
    C: ?Sized + Context,
    W: Writer,
    B: Buf,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this> = StorageEncoder<'a, &'this mut BufWriter<B>, F, C> where Self: 'this, B: Buf;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn end(mut self) -> Result<Self::Ok, C::Error> {
        static PAD: [u8; 1024] = [0; 1024];

        let buffer = self.buffer.into_inner();
        let len = buffer.len();

        let (tag, mut rem) = if len <= MAX_INLINE_LEN {
            (Tag::new(Kind::Prefix, len as u8), 0)
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

impl<'a, W, const F: Options, C> SequenceEncoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> MapEncoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> MapEntriesEncoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeMapEntryValue<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_entry_value(&mut self) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> MapEntryEncoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeMapValue<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> StructEncoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeField<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_field(&mut self) -> Result<Self::EncodeField<'_>, C::Error> {
        MapEncoder::encode_entry(self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> StructFieldEncoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeFieldName<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeFieldValue<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_field_name(&mut self) -> Result<Self::EncodeFieldName<'_>, C::Error> {
        MapEntryEncoder::encode_map_key(self)
    }

    #[inline]
    fn encode_field_value(&mut self) -> Result<Self::EncodeFieldValue<'_>, C::Error> {
        MapEntryEncoder::encode_map_value(self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        MapEntryEncoder::end(self)
    }
}

impl<'a, W, const F: Options, C> VariantEncoder for WireEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeValue<'this> = WireEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<C, W, const F: Options>(cx: &C, mut writer: W, len: usize) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        musli_common::int::encode_usize::<_, _, F>(cx, writer, len)?;
    }

    Ok(())
}
