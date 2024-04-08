use core::fmt;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntriesEncoder, MapEntryEncoder, PackEncoder, SequenceEncoder,
    StructEncoder, StructFieldEncoder, TupleEncoder, VariantEncoder,
};
use musli::hint::{MapHint, SequenceHint, StructHint, TupleHint};
use musli::{Buf, Context};
use musli_storage::en::StorageEncoder;

use crate::options::Options;
use crate::tag::{Kind, Tag, MAX_INLINE_LEN};
use crate::writer::{BufWriter, Writer};

/// A very simple encoder.
pub struct WireEncoder<'a, W, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
}

impl<'a, W, const OPT: Options, C> WireEncoder<'a, W, OPT, C>
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
            musli_common::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_tuple_len(&mut self, len: usize) -> Result<(), C::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_common::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }
}

pub struct WirePackEncoder<'a, W, B, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
    buffer: BufWriter<B>,
}

impl<'a, W, B, const OPT: Options, C: ?Sized> WirePackEncoder<'a, W, B, OPT, C> {
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
impl<'a, W, const OPT: Options, C> Encoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U> = WireEncoder<'this, W, OPT, U> where U: 'this + Context;
    type EncodePack = WirePackEncoder<'a, W, C::Buf<'a>, OPT, C>;
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
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(self.cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(mut self, len: usize, vectors: I) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;

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
    fn collect_string<T>(self, value: &T) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let buf = self.cx.collect_string(value)?;
        self.encode_string(buf.as_ref())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_length::<_, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_length::<_, _, OPT>(
            self.cx,
            self.writer.borrow_mut(),
            value as usize,
        )
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
        crate::wire_int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, C::Error> {
        crate::wire_int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
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
    fn encode_sequence(mut self, hint: &SequenceHint) -> Result<Self::EncodeSequence, C::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, hint.size);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_common::int::encode_usize::<_, _, OPT>(
                self.cx,
                self.writer.borrow_mut(),
                hint.size,
            )?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_tuple(mut self, hint: &TupleHint) -> Result<Self::EncodeTuple, C::Error> {
        self.encode_tuple_len(hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, hint: &MapHint) -> Result<Self::EncodeMap, C::Error> {
        self.encode_map_len(hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(self, hint: &MapHint) -> Result<Self::EncodeMapEntries, C::Error> {
        self.encode_map(hint)
    }

    #[inline]
    fn encode_struct(mut self, hint: &StructHint) -> Result<Self::EncodeStruct, C::Error> {
        let Some(len) = hint.size.checked_mul(2) else {
            return Err(self.cx.message("Struct length overflow"));
        };

        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_common::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;
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
        hint: &TupleHint,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        WireEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_tuple(hint)
    }

    #[inline]
    fn encode_struct_variant<T>(
        mut self,
        tag: &T,
        hint: &StructHint,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        WireEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_struct(hint)
    }
}

impl<'a, W, B, const OPT: Options, C> PackEncoder for WirePackEncoder<'a, W, B, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
    B: Buf,
{
    type Cx = C;
    type Ok = ();
    type EncodePacked<'this> = StorageEncoder<'a, &'this mut BufWriter<B>, OPT, C> where Self: 'this, B: Buf;

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

impl<'a, W, const OPT: Options, C> SequenceEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeElement<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_element(&mut self) -> Result<Self::EncodeElement<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> TupleEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeTupleField<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_tuple_field(&mut self) -> Result<Self::EncodeTupleField<'_>, C::Error> {
        SequenceEncoder::encode_element(self)
    }

    #[inline]
    fn finish_tuple(self) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::finish_sequence(self)
    }
}

impl<'a, W, const OPT: Options, C> MapEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntry<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_map_entry(&mut self) -> Result<Self::EncodeMapEntry<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> MapEntriesEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeMapEntryValue<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_entry_value(&mut self) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map_entries(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> MapEntryEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeMapValue<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map_entry(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> StructEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeStructField<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_struct_field(&mut self) -> Result<Self::EncodeStructField<'_>, C::Error> {
        MapEncoder::encode_map_entry(self)
    }

    #[inline]
    fn finish_struct(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> StructFieldEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeFieldName<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeFieldValue<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

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

impl<'a, W, const OPT: Options, C> VariantEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeValue<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
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
    len: usize,
) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        musli_common::int::encode_usize::<_, _, OPT>(cx, writer, len)?;
    }

    Ok(())
}
