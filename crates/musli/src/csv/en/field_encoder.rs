use core::borrow::BorrowMut;
use core::fmt;
use core::marker::PhantomData;

use crate::en::{Encode, Encoder, EntriesEncoder, MapEncoder, SequenceEncoder, VariantEncoder};
use crate::hint::{MapHint, SequenceHint};
use crate::{Context, Writer};

use super::super::encoding::Config;
use super::{EncState, Pass, finish_record, write_field};

const NESTED: &str = "CSV fields cannot contain containers such as maps, sequences or structs";
const VARIANT_DATA: &str = "Only unit variants can be stored in CSV fields";

/// What happens once a field has been written.
pub(crate) trait FieldEnd {
    /// Called once the field has been written. `empty` indicates if the field
    /// was empty.
    fn end<W, C>(self, cx: C, writer: &mut W, cfg: &Config, empty: bool) -> Result<(), C::Error>
    where
        W: Writer,
        C: Context;
}

/// A field inside of a record, which records whether it was empty.
impl FieldEnd for &mut bool {
    #[inline]
    fn end<W, C>(self, _: C, _: &mut W, _: &Config, empty: bool) -> Result<(), C::Error>
    where
        W: Writer,
        C: Context,
    {
        *self = empty;
        Ok(())
    }
}

/// A field which makes up a whole record.
pub(crate) struct EndRecord<S> {
    state: S,
    pass: Pass,
    active: bool,
}

impl<S> EndRecord<S> {
    #[inline]
    pub(crate) fn new(state: S, pass: Pass, active: bool) -> Self {
        Self {
            state,
            pass,
            active,
        }
    }
}

impl<S> FieldEnd for EndRecord<S>
where
    S: BorrowMut<EncState>,
{
    #[inline]
    fn end<W, C>(mut self, cx: C, writer: &mut W, cfg: &Config, empty: bool) -> Result<(), C::Error>
    where
        W: Writer,
        C: Context,
    {
        if !self.active {
            return Ok(());
        }

        finish_record(
            cx,
            writer,
            cfg,
            self.state.borrow_mut(),
            self.pass,
            1,
            empty,
        )
    }
}

/// Encoder for a single field.
pub(crate) struct FieldEncoder<W, C, M, E> {
    cx: C,
    writer: W,
    cfg: Config,
    end: E,
    _marker: PhantomData<M>,
}

impl<W, C, M, E> FieldEncoder<W, C, M, E>
where
    W: Writer,
    C: Context,
    M: 'static,
    E: FieldEnd,
{
    #[inline]
    pub(crate) fn new(cx: C, writer: W, cfg: Config, end: E) -> Self {
        Self {
            cx,
            writer,
            cfg,
            end,
            _marker: PhantomData,
        }
    }

    /// Write the given field.
    #[inline]
    fn write(mut self, bytes: &[u8]) -> Result<(), C::Error> {
        write_field(self.cx, &mut self.writer, &self.cfg, bytes)?;
        self.end
            .end(self.cx, &mut self.writer, &self.cfg, bytes.is_empty())
    }
}

#[crate::trait_defaults(crate)]
impl<W, C, M, E> Encoder for FieldEncoder<W, C, M, E>
where
    W: Writer,
    C: Context,
    M: 'static,
    E: FieldEnd,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeSome = Self;
    type EncodeVariant = FieldVariantEncoder<W, C, M, E>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be encoded as a CSV field")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.as_encode().encode(self)
    }

    #[inline]
    fn encode_empty(self) -> Result<(), Self::Error> {
        self.write(b"")
    }

    #[inline]
    fn encode_bool(self, value: bool) -> Result<(), Self::Error> {
        self.write(if value { b"true" } else { b"false" })
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<(), Self::Error> {
        self.write(value.encode_utf8(&mut [0; 4]).as_bytes())
    }

    #[inline]
    fn encode_u8(self, value: u8) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u16(self, value: u16) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u32(self, value: u32) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u64(self, value: u64) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u128(self, value: u128) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i16(self, value: i16) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i32(self, value: i32) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i64(self, value: i64) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i128(self, value: i128) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_usize(self, value: usize) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<(), Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<(), Self::Error> {
        let mut buffer = ryu::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<(), Self::Error> {
        let mut buffer = ryu::Buffer::new();
        self.write(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_string(self, string: &str) -> Result<(), Self::Error> {
        self.write(string.as_bytes())
    }

    #[inline]
    fn encode_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
        Err(self.cx.message("Bytes cannot be stored in CSV fields"))
    }

    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        self.write(b"")
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        Ok(FieldVariantEncoder {
            cx: self.cx,
            writer: self.writer,
            cfg: self.cfg,
            end: self.end,
            empty: true,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn encode_sequence(self, _: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn encode_map(self, _: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn encode_map_entries(self, _: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        Err(self.cx.message(NESTED))
    }

    #[inline]
    fn encode_sequence_variant<T>(
        self,
        _: &T,
        _: impl SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        Err(self.cx.message(VARIANT_DATA))
    }

    #[inline]
    fn encode_map_variant<T>(
        self,
        _: &T,
        _: impl MapHint,
    ) -> Result<Self::EncodeMapVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        Err(self.cx.message(VARIANT_DATA))
    }
}

/// Encoder for a unit variant stored in a field, where the tag is the content
/// of the field.
pub(crate) struct FieldVariantEncoder<W, C, M, E> {
    cx: C,
    writer: W,
    cfg: Config,
    end: E,
    empty: bool,
    _marker: PhantomData<M>,
}

impl<W, C, M, E> VariantEncoder for FieldVariantEncoder<W, C, M, E>
where
    W: Writer,
    C: Context,
    M: 'static,
    E: FieldEnd,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeTag<'this>
        = FieldEncoder<W::Mut<'this>, C, M, &'this mut bool>
    where
        Self: 'this;
    type EncodeData<'this>
        = EmptyEncoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, Self::Error> {
        Ok(FieldEncoder::new(
            self.cx,
            self.writer.borrow_mut(),
            self.cfg,
            &mut self.empty,
        ))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, Self::Error> {
        Ok(EmptyEncoder::new(self.cx))
    }

    #[inline]
    fn finish_variant(mut self) -> Result<(), Self::Error> {
        self.end
            .end(self.cx, &mut self.writer, &self.cfg, self.empty)
    }
}

/// Encoder for the data of a unit variant, which only accepts empty values.
pub(crate) struct EmptyEncoder<C, M> {
    cx: C,
    _marker: PhantomData<M>,
}

impl<C, M> EmptyEncoder<C, M> {
    #[inline]
    fn new(cx: C) -> Self {
        Self {
            cx,
            _marker: PhantomData,
        }
    }
}

#[crate::trait_defaults(crate)]
impl<C, M> Encoder for EmptyEncoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodePack = EmptyEncoder<C, M>;
    type EncodeSequence = EmptyEncoder<C, M>;
    type EncodeMap = EmptyEncoder<C, M>;
    type EncodeMapEntries = EmptyEncoder<C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{VARIANT_DATA}")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.as_encode().encode(self)
    }

    #[inline]
    fn encode_empty(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_sequence(self, _: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_map(self, _: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(self, _: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        Ok(self)
    }
}

impl<C, M> SequenceEncoder for EmptyEncoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = EmptyEncoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Err(self.cx.message(VARIANT_DATA))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<C, M> MapEncoder for EmptyEncoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = crate::__priv::Never<(C, M)>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        Err(self.cx.message(VARIANT_DATA))
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<C, M> EntriesEncoder for EmptyEncoder<C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = EmptyEncoder<C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = EmptyEncoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        Err(self.cx.message(VARIANT_DATA))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        Err(self.cx.message(VARIANT_DATA))
    }

    #[inline]
    fn finish_entries(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
