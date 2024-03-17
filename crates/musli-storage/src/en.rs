use core::fmt;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntryEncoder, MapPairsEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use musli::Context;
use musli_common::options::{self, Options};
use musli_common::writer::Writer;

const DEFAULT_OPTIONS: options::Options = options::new().build();

/// The alias for a [StorageEncoder] that is used for packs.
pub type PackEncoder<W> = StorageEncoder<W, DEFAULT_OPTIONS>;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<W, const F: Options> {
    writer: W,
}

impl<W, const F: Options> StorageEncoder<W, F> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

#[musli::encoder]
impl<W, const F: Options, C> Encoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type Encoder<U> = Self where U: Context;
    type Pack<'this> = Self where C: 'this;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type MapPairs = Self;
    type Struct = Self;
    type Variant = Self;
    type TupleVariant = Self;
    type StructVariant = Self;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::Encoder<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline(always)]
    fn encode_unit(self, cx: &C) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::end(self.encode_sequence(cx, 0)?, cx)
    }

    #[inline(always)]
    fn encode_pack(self, _: &C) -> Result<Self::Pack<'_>, C::Error> {
        Ok(self)
    }

    #[inline(always)]
    fn encode_array<const N: usize>(
        mut self,
        cx: &C,
        array: [u8; N],
    ) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(cx, &array)
    }

    #[inline(always)]
    fn encode_bytes(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_bytes_vectored(mut self, cx: &C, vectors: &[&[u8]]) -> Result<Self::Ok, C::Error> {
        let len = vectors.iter().map(|v| v.len()).sum();
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string(mut self, cx: &C, string: &str) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(cx, string.as_bytes())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_usize(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_isize(self, cx: &C, value: isize) -> Result<Self::Ok, C::Error> {
        self.encode_usize(cx, value as usize)
    }

    #[inline(always)]
    fn encode_bool(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, if value { 1 } else { 0 })
    }

    #[inline(always)]
    fn encode_char(self, cx: &C, value: char) -> Result<Self::Ok, C::Error> {
        self.encode_u32(cx, value as u32)
    }

    #[inline(always)]
    fn encode_u8(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, value)
    }

    #[inline(always)]
    fn encode_u16(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u32(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u64(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u128(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i8(self, cx: &C, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(cx, value as u8)
    }

    #[inline(always)]
    fn encode_i16(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i32(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i64(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i128(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error> {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_f32(self, cx: &C, value: f32) -> Result<Self::Ok, C::Error> {
        self.encode_u32(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_f64(self, cx: &C, value: f64) -> Result<Self::Ok, C::Error> {
        self.encode_u64(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_some(mut self, cx: &C) -> Result<Self::Some, C::Error> {
        self.writer.write_byte(cx, 1)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(cx, 0)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_sequence(mut self, cx: &C, len: usize) -> Result<Self::Sequence, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_tuple(self, _: &C, _: usize) -> Result<Self::Sequence, C::Error> {
        // NB: A tuple has statically known fixed length.
        Ok(self)
    }

    #[inline(always)]
    fn encode_map(mut self, cx: &C, len: usize) -> Result<Self::Map, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_map_pairs(mut self, cx: &C, len: usize) -> Result<Self::MapPairs, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_struct(mut self, cx: &C, len: usize) -> Result<Self::Struct, C::Error> {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_variant(self, _: &C) -> Result<Self::Variant, C::Error> {
        Ok(self)
    }

    #[inline(always)]
    fn encode_tuple_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        _: usize,
    ) -> Result<Self::TupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let encoder = StorageEncoder::<_, F>::new(self.writer.borrow_mut());
        tag.encode(cx, encoder)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_struct_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::StructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let encoder = StorageEncoder::<_, F>::new(self.writer.borrow_mut());
        tag.encode(cx, encoder)?;
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }
}

impl<W, const F: Options, C> SequenceEncoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn next(&mut self, _: &C) -> Result<Self::Encoder<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> MapEncoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type Entry<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn entry(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> MapEntryEncoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type MapKey<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapValue<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_value(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> MapPairsEncoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type MapPairsKey<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;
    type MapPairsValue<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> StructEncoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type Field<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field(&mut self, cx: &C) -> Result<Self::Field<'_>, C::Error> {
        MapEncoder::entry(self, cx)
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> StructFieldEncoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type FieldName<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;
    type FieldValue<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        self.map_key(cx)
    }

    #[inline]
    fn field_value(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        self.map_value(cx)
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<W, const F: Options, C> VariantEncoder<C> for StorageEncoder<W, F>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Ok = ();
    type Tag<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;
    type Variant<'this> = StorageEncoder<W::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn tag(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error> {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}
