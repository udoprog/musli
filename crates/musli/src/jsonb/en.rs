use core::fmt;
use core::marker::PhantomData;

use crate::en::{
    Encode, Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder,
};
use crate::hint::{MapHint, SequenceHint};
use crate::storage::en::StorageEncoder;
use crate::writer::BufWriter;
use crate::{Context, Options, Writer};

use super::integer_encoding::{encode_typed_signed, encode_typed_unsigned};
use super::tag::{
    F32, F64, I8, I16, I32, I64, I128, ISIZE, Kind, Tag, U8, U16, U32, U64, U128, USIZE,
};

/// A very simple encoder.
pub struct JsonbEncoder<const OPT: Options, W, C, M> {
    cx: C,
    writer: W,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, W, C, M> JsonbEncoder<OPT, W, C, M> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: C, writer: W) -> Self {
        Self {
            cx,
            writer,
            _marker: PhantomData,
        }
    }
}

pub struct SelfPackEncoder<const OPT: Options, W, C, M>
where
    C: Context,
{
    cx: C,
    writer: W,
    buffer: BufWriter<C::Allocator>,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, W, C, M> SelfPackEncoder<OPT, W, C, M>
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
            _marker: PhantomData,
        }
    }
}

#[crate::trait_defaults(crate)]
impl<const OPT: Options, W, C, M> Encoder for JsonbEncoder<OPT, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodePack = SelfPackEncoder<OPT, W, C, M>;
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
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the jsonb encoder")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.encode(self)
    }

    #[inline]
    fn encode_empty(self) -> Result<(), Self::Error> {
        Err(self.cx.message("empty values are not supported"))
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Ok(SelfPackEncoder::new(self.cx, self.writer))
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: &[u8; N]) -> Result<(), Self::Error> {
        self.encode_bytes(array)
    }

    #[inline]
    fn encode_bytes(self, bytes: &[u8]) -> Result<(), Self::Error> {
        Err(self.cx.message(format_args!(
            "bytes are not supported (length: {})",
            bytes.len()
        )))
    }

    #[inline]
    fn encode_bytes_vectored<I>(self, len: usize, _vectors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item: AsRef<[u8]>>,
    {
        Err(self.cx.message(format_args!(
            "vectored bytes are not supported (length: {len})"
        )))
    }

    #[inline]
    fn encode_string(self, string: &str) -> Result<(), Self::Error> {
        Err(self
            .cx
            .message(format_args!("strings are not supported (string: {string})")))
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<(), Self::Error> {
        const TRUE: Tag = Tag::new(Kind::True, 0);
        const FALSE: Tag = Tag::new(Kind::False, 0);

        self.writer
            .write_byte(self.cx, if value { TRUE } else { FALSE }.bits())
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<(), Self::Error> {
        let mut buf = [0; 4];
        let encoded = value.encode_utf8(&mut buf);
        self.encode_string(encoded)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U8, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U16, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U32, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U64, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), U128, value)
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<(), Self::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I8, value)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<(), Self::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I16, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<(), Self::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I32, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<(), Self::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I64, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<(), Self::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), I128, value)
    }

    #[inline]
    fn encode_f32(mut self, value: f32) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), F32, value.to_bits())
    }

    #[inline]
    fn encode_f64(mut self, value: f64) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), F64, value.to_bits())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<(), Self::Error> {
        encode_typed_unsigned(self.cx, self.writer.borrow_mut(), USIZE, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<(), Self::Error> {
        encode_typed_signed(self.cx, self.writer.borrow_mut(), ISIZE, value)
    }

    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<(), Self::Error> {
        const NONE: Tag = Tag::new(Kind::Null, 0);
        self.writer.write_byte(self.cx, NONE.bits())?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(self, hint: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        let size = hint.require(self.cx)?;

        Err(self
            .cx
            .message(format_args!("sequences are not supported (size: {size})")))
    }

    #[inline]
    fn encode_map(self, hint: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        let size = hint.require(self.cx)?;

        Err(self
            .cx
            .message(format_args!("maps are not supported (size: {size})")))
    }

    #[inline]
    fn encode_map_entries(self, hint: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        let size = hint.require(self.cx)?;

        Err(self
            .cx
            .message(format_args!("map entries are not supported (size: {size})")))
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        Err(self.cx.message("variants are not supported"))
    }

    #[inline]
    fn encode_unit_variant<T>(self, _tag: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        Err(self
            .cx
            .message(format_args!("unit variants are not supported")))
    }

    #[inline]
    fn encode_sequence_variant<T>(
        self,
        _tag: &T,
        hint: impl SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        self.encode_sequence(hint)
    }

    #[inline]
    fn encode_map_variant<T>(
        self,
        _tag: &T,
        hint: impl MapHint,
    ) -> Result<Self::EncodeMapVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        self.encode_map(hint)
    }
}

impl<const OPT: Options, W, C, M> SequenceEncoder for SelfPackEncoder<OPT, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = StorageEncoder<OPT, true, &'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        let _buffer = self.buffer.into_inner();
        Err(self.cx.message("packed sequences are not supported"))
    }
}

impl<const OPT: Options, W, C, M> SequenceEncoder for JsonbEncoder<OPT, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, W, C, M> MapEncoder for JsonbEncoder<OPT, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, W, C, M> EntryEncoder for JsonbEncoder<OPT, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeKey<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeValue<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, W, C, M> EntriesEncoder for JsonbEncoder<OPT, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entries(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, W, C, M> VariantEncoder for JsonbEncoder<OPT, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeTag<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeData<'this>
        = JsonbEncoder<OPT, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Encode a length prefix.
#[inline]
fn encode_prefix<const OPT: Options, W, C>(
    cx: C,
    _writer: W,
    _kind: Kind,
    _len: usize,
) -> Result<(), C::Error>
where
    W: Writer,
    C: Context,
{
    Err(cx.message("unsupported prefix"))
}
