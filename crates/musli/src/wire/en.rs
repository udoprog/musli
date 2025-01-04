use core::fmt;

use crate::en::{
    Encode, Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder,
};
use crate::hint::{MapHint, SequenceHint};
use crate::storage::en::StorageEncoder;
use crate::writer::BufWriter;
use crate::{Context, Options, Writer};

use super::tag::{Kind, Tag};

/// A very simple encoder.
pub struct WireEncoder<W, const OPT: Options, C> {
    cx: C,
    writer: W,
}

impl<W, const OPT: Options, C> WireEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: C, writer: W) -> Self {
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
            crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_sequence_len(&mut self, len: usize) -> Result<(), C::Error> {
        let (tag, embedded) = Tag::with_len(Kind::Sequence, len);
        self.writer.write_byte(self.cx, tag.byte())?;

        if !embedded {
            crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;
        }

        Ok(())
    }
}

pub struct WirePackEncoder<W, const OPT: Options, C>
where
    C: Context,
{
    cx: C,
    writer: W,
    buffer: BufWriter<C::Allocator>,
}

impl<W, const OPT: Options, C> WirePackEncoder<W, OPT, C>
where
    C: Context,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: C, writer: W) -> Self {
        Self {
            cx,
            writer,
            buffer: BufWriter::new(cx.alloc()),
        }
    }
}

#[crate::encoder(crate)]
impl<W, const OPT: Options, C> Encoder for WireEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<U>
        = WireEncoder<W, OPT, U>
    where
        U: Context<Allocator = <Self::Cx as Context>::Allocator>;
    type EncodePack = WirePackEncoder<W, OPT, C>;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeVariant = Self;
    type EncodeSequenceVariant = Self;
    type EncodeMapVariant = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: U) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context<Allocator = <Self::Cx as Context>::Allocator>,
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
        value.as_encode().encode(self)
    }

    #[inline]
    fn encode_empty(mut self) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_byte(self.cx, Tag::new(Kind::Sequence, 0).byte())?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        Ok(WirePackEncoder::new(self.cx, self.writer))
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
        I: IntoIterator<Item: AsRef<[u8]>>,
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
        crate::wire::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
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
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_length::<_, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, C::Error> {
        crate::wire::int::encode_length::<_, _, OPT>(
            self.cx,
            self.writer.borrow_mut(),
            value as usize,
        )
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

impl<W, const OPT: Options, C> SequenceEncoder for WirePackEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this>
        = StorageEncoder<OPT, true, &'this mut BufWriter<C::Allocator>, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_sequence(mut self) -> Result<Self::Ok, C::Error> {
        let buffer = self.buffer.into_inner();
        encode_prefix::<_, _, OPT>(self.cx, self.writer.borrow_mut(), buffer.len())?;
        self.writer.extend(self.cx, buffer)?;
        Ok(())
    }
}

impl<W, const OPT: Options, C> SequenceEncoder for WireEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const OPT: Options, C> MapEncoder for WireEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(WireEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const OPT: Options, C> EntriesEncoder for WireEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntryKey<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

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

impl<W, const OPT: Options, C> EntryEncoder for WireEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeKey<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;
    type EncodeValue<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

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

impl<W, const OPT: Options, C> VariantEncoder for WireEncoder<W, OPT, C>
where
    C: Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;
    type EncodeData<'this>
        = WireEncoder<W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

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
fn encode_prefix<C, W, const OPT: Options>(cx: C, mut writer: W, len: usize) -> Result<(), C::Error>
where
    C: Context,
    W: Writer,
{
    let (tag, embedded) = Tag::with_len(Kind::Prefix, len);
    writer.write_byte(cx, tag.byte())?;

    if !embedded {
        crate::int::encode_usize::<_, _, OPT>(cx, writer, len)?;
    }

    Ok(())
}
