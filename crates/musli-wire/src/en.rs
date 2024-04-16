use core::fmt;

use musli::en::{
    Encode, Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder,
};
use musli::hint::{MapHint, SequenceHint};
use musli::{Buf, Context};
use musli_storage::en::StorageEncoder;
use musli_utils::writer::BufWriter;
use musli_utils::{Options, Writer};

use crate::tag::{Kind, Tag};

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
            musli_utils::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_sequence_len(&mut self, len: usize) -> Result<(), C::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            musli_utils::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }
}

pub struct WireSequenceEncoder<'a, W, B, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
    buffer: BufWriter<B>,
}

impl<'a, W, B, const OPT: Options, C: ?Sized> WireSequenceEncoder<'a, W, B, OPT, C> {
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
    type EncodePack = WireSequenceEncoder<'a, W, C::Buf<'a>, OPT, C>;
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

        Ok(WireSequenceEncoder::new(self.cx, self.writer, buf))
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
        self.encode_sequence_len(hint.size)?;
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
    fn encode_variant(mut self) -> Result<Self::EncodeVariant, C::Error> {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        Ok(self)
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
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        WireEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_sequence(hint)
    }

    #[inline]
    fn encode_map_variant<T>(
        mut self,
        tag: &T,
        hint: &MapHint,
    ) -> Result<Self::EncodeSequenceVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 2).byte())?;
        WireEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        self.encode_map(hint)
    }
}

impl<'a, W, B, const OPT: Options, C> SequenceEncoder for WireSequenceEncoder<'a, W, B, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
    B: Buf,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this> = StorageEncoder<'a, &'this mut BufWriter<B>, OPT, C> where Self: 'this, B: Buf;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_sequence(mut self) -> Result<Self::Ok, C::Error> {
        let buffer = self.buffer.into_inner();
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), buffer.len())?;
        self.writer.write_buffer(self.cx, buffer)?;
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
    type EncodeNext<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> MapEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> EntriesEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntryKey<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeEntryValue<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entries(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> EntryEncoder for WireEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeKey<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;
    type EncodeValue<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, C::Error> {
        Ok(())
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
    type EncodeData<'this> = WireEncoder<'a, W::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, C::Error> {
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
        musli_utils::int::encode_usize::<_, _, OPT>(cx, writer, len)?;
    }

    Ok(())
}
