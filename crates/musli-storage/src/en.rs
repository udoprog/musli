use core::fmt;
use core::marker::PhantomData;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntriesEncoder, MapEntryEncoder, SequenceEncoder,
    StructEncoder, StructFieldEncoder, VariantEncoder,
};
use musli::Context;

use crate::options::{self, Options};
use crate::writer::Writer;

const DEFAULT_OPTIONS: options::Options = options::new().build();

/// The alias for a [StorageEncoder] that is used for packs.
pub type PackEncoder<W, C> = StorageEncoder<W, DEFAULT_OPTIONS, C>;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<W, const F: Options, C: ?Sized> {
    writer: W,
    _marker: PhantomData<C>,
}

impl<W, const F: Options, C: ?Sized> StorageEncoder<W, F, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _marker: PhantomData,
        }
    }
}

#[musli::encoder]
impl<W, const F: Options, C> Encoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<U> = StorageEncoder<W, F, U> where U: Context;
    type EncodePack<'this> = StorageEncoder<W, F, C> where Self::Cx: 'this;
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
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(StorageEncoder::new(self.writer))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline]
    fn encode_unit(self, cx: &C) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::end(self.encode_sequence(cx, 0)?, cx)
    }

    #[inline]
    fn encode_pack(self, _: &C) -> Result<Self::EncodePack<'_>, C::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_array<const N: usize>(
        mut self,
        cx: &C,
        array: &[u8; N],
    ) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(cx, array)
    }

    #[inline]
    fn encode_bytes(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(
        mut self,
        cx: &C,
        len: usize,
        vectors: I,
    ) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes.as_ref())?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, cx: &C, string: &str) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(self, cx: &C, value: isize) -> Result<Self::Ok, C::Error> {
        self.encode_usize(cx, value as usize)
    }

    #[inline]
    fn encode_bool(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, if value { 1 } else { 0 })
    }

    #[inline]
    fn encode_char(self, cx: &C, value: char) -> Result<Self::Ok, C::Error> {
        self.encode_u32(cx, value as u32)
    }

    #[inline]
    fn encode_u8(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, value)
    }

    #[inline]
    fn encode_u16(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, cx: &C, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(cx, value as u8)
    }

    #[inline]
    fn encode_i16(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_f32(self, cx: &C, value: f32) -> Result<Self::Ok, C::Error> {
        self.encode_u32(cx, value.to_bits())
    }

    #[inline]
    fn encode_f64(self, cx: &C, value: f64) -> Result<Self::Ok, C::Error> {
        self.encode_u64(cx, value.to_bits())
    }

    #[inline]
    fn encode_some(mut self, cx: &C) -> Result<Self::EncodeSome, C::Error> {
        self.writer.write_byte(cx, 1)?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, 0)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, cx: &C, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(self, _: &C, _: usize) -> Result<Self::EncodeSequence, C::Error> {
        // NB: A tuple has statically known fixed length.
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, cx: &C, len: usize) -> Result<Self::EncodeMap, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(
        mut self,
        cx: &C,
        len: usize,
    ) -> Result<Self::EncodeMapEntries, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, cx: &C, len: usize) -> Result<Self::EncodeStruct, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(self, _: &C) -> Result<Self::EncodeVariant, C::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_tuple_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        _: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let encoder = StorageEncoder::<_, F, _>::new(self.writer.borrow_mut());
        tag.encode(cx, encoder)?;
        Ok(self)
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
        let encoder = StorageEncoder::<_, F, _>::new(self.writer.borrow_mut());
        tag.encode(cx, encoder)?;
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }
}

impl<W, const F: Options, C> SequenceEncoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> MapEncoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_entry(&mut self, _: &C) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> MapEntryEncoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;
    type EncodeMapValue<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self, _: &C) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_value(&mut self, _: &C) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> MapEntriesEncoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;
    type EncodeMapEntryValue<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self, _: &C) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_entry_value(&mut self, _: &C) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> StructEncoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeField<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_field(&mut self, cx: &C) -> Result<Self::EncodeField<'_>, C::Error> {
        MapEncoder::encode_entry(self, cx)
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> StructFieldEncoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeFieldName<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;
    type EncodeFieldValue<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_field_name(&mut self, cx: &C) -> Result<Self::EncodeFieldName<'_>, C::Error> {
        MapEntryEncoder::encode_map_key(self, cx)
    }

    #[inline]
    fn encode_field_value(&mut self, cx: &C) -> Result<Self::EncodeFieldValue<'_>, C::Error> {
        MapEntryEncoder::encode_map_value(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error> {
        MapEntryEncoder::end(self, cx)
    }
}

impl<W, const F: Options, C> VariantEncoder for StorageEncoder<W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;
    type EncodeValue<'this> = StorageEncoder<W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self, _: &C) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self, _: &C) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}
