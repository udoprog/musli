use core::fmt;
use core::marker;

use musli::en::{
    Encode, Encoder, MapEncoder, MapEntryEncoder, MapPairsEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use musli::{Context, Mode};
use musli_common::options::{self, Options};
use musli_common::writer::Writer;

const DEFAULT_OPTIONS: options::Options = options::new().build();

/// The alias for a [StorageEncoder] that is used for packs.
pub type PackEncoder<W, E> = StorageEncoder<W, DEFAULT_OPTIONS, E>;

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<W, const F: Options, E> {
    writer: W,
    _marker: marker::PhantomData<E>,
}

impl<W, const F: Options, E> StorageEncoder<W, F, E> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _marker: marker::PhantomData,
        }
    }
}

#[musli::encoder]
impl<W, const F: Options, E: 'static> Encoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;

    type Pack<'this, C> = Self where C: 'this + Context;
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
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline(always)]
    fn encode_unit<C>(self, cx: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        SequenceEncoder::end(self.encode_sequence(cx, 0)?, cx)
    }

    #[inline(always)]
    fn encode_pack<C>(self, _: &C) -> Result<Self::Pack<'_, C>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline(always)]
    fn encode_array<C, const N: usize>(
        mut self,
        cx: &C,
        array: [u8; N],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_bytes(cx, &array)
    }

    #[inline(always)]
    fn encode_bytes<C>(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(cx, bytes)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_bytes_vectored<C>(mut self, cx: &C, vectors: &[&[u8]]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let len = vectors.iter().map(|v| v.len()).sum();
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(cx, bytes)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn encode_string<C>(mut self, cx: &C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(cx, string.as_bytes())?;
        Ok(())
    }

    #[inline(always)]
    fn encode_usize<C>(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_isize<C>(self, cx: &C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_usize(cx, value as usize)
    }

    #[inline(always)]
    fn encode_bool<C>(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(cx, if value { 1 } else { 0 })
    }

    #[inline(always)]
    fn encode_char<C>(self, cx: &C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u32(cx, value as u32)
    }

    #[inline(always)]
    fn encode_u8<C>(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(cx, value)
    }

    #[inline(always)]
    fn encode_u16<C>(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u32<C>(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u64<C>(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_u128<C>(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_unsigned::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i8<C>(self, cx: &C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u8(cx, value as u8)
    }

    #[inline(always)]
    fn encode_i16<C>(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i32<C>(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i64<C>(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_i128<C>(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_signed::<_, _, _, F>(cx, self.writer.borrow_mut(), value)
    }

    #[inline(always)]
    fn encode_f32<C>(self, cx: &C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u32(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_f64<C>(self, cx: &C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.encode_u64(cx, value.to_bits())
    }

    #[inline(always)]
    fn encode_some<C>(mut self, cx: &C) -> Result<Self::Some, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(cx, 1)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_none<C>(mut self, cx: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.writer.write_byte(cx, 0)?;
        Ok(())
    }

    #[inline(always)]
    fn encode_sequence<C>(mut self, cx: &C, len: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_tuple<C>(self, _: &C, _: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        // NB: A tuple has statically known fixed length.
        Ok(self)
    }

    #[inline(always)]
    fn encode_map<C>(mut self, cx: &C, len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_map_pairs<C>(mut self, cx: &C, len: usize) -> Result<Self::MapPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_struct<C>(mut self, cx: &C, len: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::encode_usize::<_, _, F>(cx, self.writer.borrow_mut(), len)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_variant<C>(self, _: &C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline(always)]
    fn encode_tuple_variant<M, C, T>(
        mut self,
        cx: &C,
        tag: &T,
        _: usize,
    ) -> Result<Self::TupleVariant, C::Error>
    where
        M: Mode,
        C: Context<Input = Self::Error>,
        T: Encode<M>,
    {
        let encoder = StorageEncoder::<_, F, E>::new(self.writer.borrow_mut());
        Encode::<M>::encode(tag, cx, encoder)?;
        Ok(self)
    }

    #[inline(always)]
    fn encode_struct_variant<M, C, T>(
        mut self,
        cx: &C,
        tag: &T,
        _: usize,
    ) -> Result<Self::StructVariant, C::Error>
    where
        M: Mode,
        C: Context<Input = Self::Error>,
        T: Encode<M>,
    {
        let encoder = StorageEncoder::<_, F, E>::new(self.writer.borrow_mut());
        Encode::<M>::encode(tag, cx, encoder)?;
        Ok(self)
    }
}

impl<W, const F: Options, E: 'static> SequenceEncoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;
    type Encoder<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options, E: 'static> MapEncoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;
    type Entry<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn entry<C>(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options, E: 'static> MapEntryEncoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;
    type MapKey<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;
    type MapValue<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn map_key<C>(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_value<C>(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options, E: 'static> MapPairsEncoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;
    type MapPairsKey<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;
    type MapPairsValue<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn map_pairs_key<C>(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn map_pairs_value<C>(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options, E: 'static> StructEncoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;
    type Field<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn field<C>(&mut self, cx: &C) -> Result<Self::Field<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEncoder::entry(self, cx)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options, E: 'static> StructFieldEncoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;
    type FieldName<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;
    type FieldValue<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.map_key(cx)
    }

    #[inline]
    fn field_value<C>(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.map_value(cx)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<W, const F: Options, E: 'static> VariantEncoder for StorageEncoder<W, F, E>
where
    W: Writer,
{
    type Ok = ();
    type Error = E;
    type Tag<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;
    type Variant<'this> = StorageEncoder<W::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}
