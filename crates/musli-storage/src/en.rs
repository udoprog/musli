use core::fmt;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntriesEncoder, MapEntryEncoder, SequenceEncoder,
    StructEncoder, StructFieldEncoder, VariantEncoder,
};
use musli::Context;

use crate::options::{self, Options};
use crate::writer::Writer;

const DEFAULT_OPTIONS: options::Options = options::new().build();

/// The alias for a [StorageEncoder] that is used for packs.
pub type PackEncoder<'a, W, C> = StorageEncoder<'a, W, DEFAULT_OPTIONS, C>;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<'a, W, const F: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
}

impl<'a, W, const F: Options, C: ?Sized> StorageEncoder<'a, W, F, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(cx: &'a C, writer: W) -> Self {
        Self { cx, writer }
    }
}

#[musli::encoder]
impl<'a, W, const F: Options, C> Encoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U> = StorageEncoder<'this, W, F, U> where U: 'this + Context;
    type EncodePack = StorageEncoder<'a, W, F, C>;
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
        Ok(StorageEncoder::new(cx, self.writer))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<Self::Ok, Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.encode(self.cx, self)
    }

    #[inline]
    fn encode_unit(self) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::end(self.encode_sequence(0)?)
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_array<const N: usize>(mut self, array: &[u8; N]) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(self.cx, array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(self.cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(mut self, len: usize, vectors: I) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(self.cx, bytes.as_ref())?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(
            self.cx,
            self.writer.borrow_mut(),
            string.len(),
        )?;
        self.writer.write_bytes(self.cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<Self::Ok, C::Error> {
        self.encode_usize(value as usize)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, if value { 1 } else { 0 })
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, C::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(self.cx, self.writer.borrow_mut(), value)
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
        self.writer.write_byte(self.cx, 1)?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, 0)?;
        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, len: usize) -> Result<Self::EncodeSequence, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_tuple(self, _: usize) -> Result<Self::EncodeSequence, C::Error> {
        // NB: A tuple has statically known fixed length.
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, len: usize) -> Result<Self::EncodeMap, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(mut self, len: usize) -> Result<Self::EncodeMapEntries, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_struct(mut self, len: usize) -> Result<Self::EncodeStruct, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, C::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_tuple_variant<T>(
        mut self,
        tag: &T,
        _: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        StorageEncoder::<_, F, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        Ok(self)
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
        StorageEncoder::<_, F, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        musli_common::int::encode_usize::<_, _, F>(self.cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }
}

impl<'a, W, const F: Options, C> SequenceEncoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> MapEncoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> MapEntryEncoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeMapValue<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> MapEntriesEncoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeMapEntryValue<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_map_entry_value(&mut self) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> StructEncoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeField<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_field(&mut self) -> Result<Self::EncodeField<'_>, C::Error> {
        MapEncoder::encode_entry(self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const F: Options, C> StructFieldEncoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeFieldName<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeFieldValue<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

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

impl<'a, W, const F: Options, C> VariantEncoder for StorageEncoder<'a, W, F, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;
    type EncodeValue<'this> = StorageEncoder<'a, W::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}
