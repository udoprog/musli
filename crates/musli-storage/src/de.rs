use core::fmt;
use core::marker;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, MapDecoder, MapEntryDecoder, MapPairsDecoder, PackDecoder, SequenceDecoder, SizeHint,
    StructDecoder, StructFieldDecoder, StructPairsDecoder, ValueVisitor, VariantDecoder,
};
use musli::Context;
use musli_common::options::Options;
use musli_common::reader::Reader;

/// A very simple decoder suitable for storage decoding.
pub struct StorageDecoder<R, const F: Options, E> {
    reader: R,
    _marker: marker::PhantomData<E>,
}

impl<R, const F: Options, E> StorageDecoder<R, F, E> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            _marker: marker::PhantomData,
        }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct LimitedStorageDecoder<R, const F: Options, E> {
    remaining: usize,
    decoder: StorageDecoder<R, F, E>,
}

#[musli::decoder]
impl<'de, R, const F: Options, E: 'static> Decoder<'de> for StorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type Pack = Self;
    type Some = Self;
    type Sequence = LimitedStorageDecoder<R, F, E>;
    type Tuple = Self;
    type Map = LimitedStorageDecoder<R, F, E>;
    type MapPairs = LimitedStorageDecoder<R, F, E>;
    type Struct = LimitedStorageDecoder<R, F, E>;
    type StructPairs = LimitedStorageDecoder<R, F, E>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage decoder")
    }

    #[inline(always)]
    fn decode_unit<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let mark = cx.mark();
        let count =
            musli_common::int::decode_usize::<_, _, F>(cx.adapt(), self.reader.borrow_mut())?;

        if count != 0 {
            return Err(cx.marked_message(mark, ExpectedEmptySequence { actual: count }));
        }

        Ok(())
    }

    #[inline(always)]
    fn decode_pack<C>(self, _: &C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline(always)]
    fn decode_array<C, const N: usize>(mut self, cx: &C) -> Result<[u8; N], C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.reader.read_array(cx.adapt())
    }

    #[inline(always)]
    fn decode_bytes<C, V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, [u8]>,
    {
        let len = musli_common::int::decode_usize::<_, _, F>(cx.adapt(), self.reader.borrow_mut())?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline(always)]
    fn decode_string<C, V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, str>,
    {
        struct Visitor<V>(V);

        impl<'de, C, V> ValueVisitor<'de, C, [u8]> for Visitor<V>
        where
            C: Context,
            V: ValueVisitor<'de, C, str>,
        {
            type Ok = V::Ok;

            #[inline(always)]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline(always)]
            fn visit_owned(self, cx: &C, bytes: Vec<u8>) -> Result<Self::Ok, C::Error> {
                let string =
                    musli_common::str::from_utf8_owned(bytes).map_err(|error| cx.custom(error))?;
                self.0.visit_owned(cx, string)
            }

            #[inline(always)]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string =
                    musli_common::str::from_utf8(bytes).map_err(|error| cx.custom(error))?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline(always)]
            fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string =
                    musli_common::str::from_utf8(bytes).map_err(|error| cx.custom(error))?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(cx, Visitor(visitor))
    }

    #[inline(always)]
    fn decode_bool<C>(mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let mark = cx.mark();
        let byte = self.reader.read_byte(cx.adapt())?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(cx.marked_message(mark, BadBoolean { actual: b })),
        }
    }

    #[inline(always)]
    fn decode_char<C>(self, cx: &C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let mark = cx.mark();
        let num = self.decode_u32(cx)?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.marked_message(mark, BadCharacter { actual: num })),
        }
    }

    #[inline(always)]
    fn decode_u8<C>(mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.reader.read_byte(cx.adapt())
    }

    #[inline(always)]
    fn decode_u16<C>(self, cx: &C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_u32<C>(self, cx: &C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_u64<C>(self, cx: &C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_u128<C>(self, cx: &C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_i8<C>(self, cx: &C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self.decode_u8(cx)? as i8)
    }

    #[inline(always)]
    fn decode_i16<C>(self, cx: &C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_signed::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_i32<C>(self, cx: &C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_signed::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_i64<C>(self, cx: &C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_signed::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_i128<C>(self, cx: &C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_signed::<_, _, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_usize<C>(self, cx: &C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        musli_common::int::decode_usize::<_, _, F>(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_isize<C>(self, cx: &C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self.decode_usize(cx)? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f32<C>(self, cx: &C) -> Result<f32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f64<C>(self, cx: &C) -> Result<f64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option<C>(mut self, cx: &C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let b = self.reader.read_byte(cx.adapt())?;
        Ok(if b == 1 { Some(self) } else { None })
    }

    #[inline]
    fn decode_sequence<C>(self, cx: &C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_tuple<C>(self, _: &C, _: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn decode_map<C>(self, cx: &C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_map_pairs<C>(self, cx: &C) -> Result<Self::MapPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_struct<C>(self, cx: &C, _: Option<usize>) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_struct_pairs<C>(self, cx: &C, _: Option<usize>) -> Result<Self::StructPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_variant<C>(self, _: &C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }
}

impl<'de, R, const F: Options, E: 'static> PackDecoder<'de> for StorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type Decoder<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, const F: Options, E: 'static> LimitedStorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    #[inline]
    fn new<C>(cx: &C, mut decoder: StorageDecoder<R, F, E>) -> Result<Self, C::Error>
    where
        C: Context<Input = E>,
    {
        let remaining =
            musli_common::int::decode_usize::<_, _, F>(cx.adapt(), &mut decoder.reader)?;
        Ok(Self { remaining, decoder })
    }
}

impl<'de, R, const F: Options, E: 'static> SequenceDecoder<'de> for LimitedStorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type Decoder<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, const F: Options, E: 'static> MapDecoder<'de> for LimitedStorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;

    type Entry<'this> = StorageDecoder<R::Mut<'this>, F, E>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn entry<C>(&mut self, _: &C) -> Result<Option<Self::Entry<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, const F: Options, E: 'static> MapEntryDecoder<'de> for StorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type MapKey<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;
    type MapValue = Self;

    #[inline]
    fn map_key<C>(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn map_value<C>(self, _: &C) -> Result<Self::MapValue, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn skip_map_value<C>(self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(false)
    }
}

impl<'de, R, const F: Options, E: 'static> StructDecoder<'de> for LimitedStorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;

    type Field<'this> = StorageDecoder<R::Mut<'this>, F, E>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        MapDecoder::size_hint(self)
    }

    #[inline]
    fn field<C>(&mut self, cx: &C) -> Result<Option<Self::Field<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapDecoder::entry(self, cx)
    }

    #[inline]
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapDecoder::end(self, cx)
    }
}

impl<'de, R, const F: Options, E: 'static> StructFieldDecoder<'de> for StorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type FieldName<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;
    type FieldValue = Self;

    #[inline]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEntryDecoder::map_key(self, cx)
    }

    #[inline]
    fn field_value<C>(self, cx: &C) -> Result<Self::FieldValue, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEntryDecoder::map_value(self, cx)
    }

    #[inline]
    fn skip_field_value<C>(self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEntryDecoder::skip_map_value(self, cx)
    }
}

impl<'de, R, const F: Options, E: 'static> MapPairsDecoder<'de> for LimitedStorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type MapPairsKey<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;
    type MapPairsValue<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn map_pairs_key<C>(&mut self, _: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn map_pairs_value<C>(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_map_pairs_value<C>(&mut self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(false)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, const F: Options, E: 'static> StructPairsDecoder<'de>
    for LimitedStorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type FieldName<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;
    type FieldValue<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Err(cx.message("Ran out of struct fields to decode"));
        }

        self.remaining -= 1;
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn field_value<C>(&mut self, _: &C) -> Result<Self::FieldValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_field_value<C>(&mut self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(false)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, const F: Options, E: 'static> VariantDecoder<'de> for StorageDecoder<R, F, E>
where
    R: Reader<'de>,
    E: From<R::Error>,
{
    type Error = E;
    type Tag<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;
    type Variant<'this> = StorageDecoder<R::Mut<'this>, F, E> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn skip_variant<C>(&mut self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(false)
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

struct ExpectedEmptySequence {
    actual: usize,
}

impl fmt::Display for ExpectedEmptySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Expected empty sequence, but was {actual}",)
    }
}

struct BadBoolean {
    actual: u8,
}

impl fmt::Display for BadBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Bad boolean byte 0x{actual:02x}")
    }
}

struct BadCharacter {
    actual: u32,
}

impl fmt::Display for BadCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Bad character number {actual}")
    }
}
