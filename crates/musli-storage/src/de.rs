use core::fmt;

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
pub struct StorageDecoder<R, const F: Options> {
    reader: R,
}

impl<R, const F: Options> StorageDecoder<R, F> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct LimitedStorageDecoder<R, const F: Options> {
    remaining: usize,
    decoder: StorageDecoder<R, F>,
}

#[musli::decoder]
impl<'de, R, const F: Options, C> Decoder<'de, C> for StorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Decoder<U> = Self where U: Context;
    type Pack = Self;
    type Some = Self;
    type Sequence = LimitedStorageDecoder<R, F>;
    type Tuple = Self;
    type Map = LimitedStorageDecoder<R, F>;
    type Struct = LimitedStorageDecoder<R, F>;
    type Variant = Self;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::Decoder<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage decoder")
    }

    #[inline(always)]
    fn decode_unit(mut self, cx: &C) -> Result<(), C::Error> {
        let mark = cx.mark();
        let count = musli_common::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?;

        if count != 0 {
            return Err(cx.marked_message(mark, ExpectedEmptySequence { actual: count }));
        }

        Ok(())
    }

    #[inline(always)]
    fn decode_pack(self, _: &C) -> Result<Self::Pack, C::Error> {
        Ok(self)
    }

    #[inline(always)]
    fn decode_array<const N: usize>(mut self, cx: &C) -> Result<[u8; N], C::Error> {
        self.reader.read_array(cx)
    }

    #[inline(always)]
    fn decode_bytes<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let len = musli_common::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline(always)]
    fn decode_string<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
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
    fn decode_bool(mut self, cx: &C) -> Result<bool, C::Error> {
        let mark = cx.mark();
        let byte = self.reader.read_byte(cx)?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(cx.marked_message(mark, BadBoolean { actual: b })),
        }
    }

    #[inline(always)]
    fn decode_char(self, cx: &C) -> Result<char, C::Error> {
        let mark = cx.mark();
        let num = self.decode_u32(cx)?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.marked_message(mark, BadCharacter { actual: num })),
        }
    }

    #[inline(always)]
    fn decode_u8(mut self, cx: &C) -> Result<u8, C::Error> {
        self.reader.read_byte(cx)
    }

    #[inline(always)]
    fn decode_u16(self, cx: &C) -> Result<u16, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u64(self, cx: &C) -> Result<u64, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u128(self, cx: &C) -> Result<u128, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i8(self, cx: &C) -> Result<i8, C::Error> {
        Ok(self.decode_u8(cx)? as i8)
    }

    #[inline(always)]
    fn decode_i16(self, cx: &C) -> Result<i16, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i32(self, cx: &C) -> Result<i32, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i64(self, cx: &C) -> Result<i64, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i128(self, cx: &C) -> Result<i128, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_usize(self, cx: &C) -> Result<usize, C::Error> {
        musli_common::int::decode_usize::<_, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_isize(self, cx: &C) -> Result<isize, C::Error> {
        Ok(self.decode_usize(cx)? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f32(self, cx: &C) -> Result<f32, C::Error> {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f64(self, cx: &C) -> Result<f64, C::Error> {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option(mut self, cx: &C) -> Result<Option<Self::Some>, C::Error> {
        let b = self.reader.read_byte(cx)?;
        Ok(if b == 1 { Some(self) } else { None })
    }

    #[inline]
    fn decode_sequence(self, cx: &C) -> Result<Self::Sequence, C::Error> {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_tuple(self, _: &C, _: usize) -> Result<Self::Tuple, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_map(self, cx: &C) -> Result<Self::Map, C::Error> {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_struct(self, cx: &C, _: Option<usize>) -> Result<Self::Struct, C::Error> {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_variant(self, _: &C) -> Result<Self::Variant, C::Error> {
        Ok(self)
    }
}

impl<'de, R, const F: Options, C> PackDecoder<'de, C> for StorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Decoder<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn next(&mut self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, R, const F: Options> LimitedStorageDecoder<R, F>
where
    R: Reader<'de>,
{
    #[inline]
    fn new<C>(cx: &C, mut decoder: StorageDecoder<R, F>) -> Result<Self, C::Error>
    where
        C: Context,
    {
        let remaining = musli_common::int::decode_usize::<_, _, F>(cx, &mut decoder.reader)?;
        Ok(Self { remaining, decoder })
    }
}

impl<'de, R, const F: Options, C> SequenceDecoder<'de, C> for LimitedStorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Decoder<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next(&mut self, _: &C) -> Result<Option<Self::Decoder<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

#[musli::map_decoder]
impl<'de, R, const F: Options, C> MapDecoder<'de, C> for LimitedStorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Entry<'this> = StorageDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type MapPairs = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn into_map_pairs(self, _: &C) -> Result<Self::MapPairs, C::Error> {
        Ok(self)
    }

    #[inline]
    fn entry(&mut self, _: &C) -> Result<Option<Self::Entry<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, R, const F: Options, C> MapEntryDecoder<'de, C> for StorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type MapKey<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;
    type MapValue = Self;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn map_value(self, _: &C) -> Result<Self::MapValue, C::Error> {
        Ok(self)
    }

    #[inline]
    fn skip_map_value(self, _: &C) -> Result<bool, C::Error> {
        Ok(false)
    }
}

#[musli::struct_decoder]
impl<'de, R, const F: Options, C> StructDecoder<'de, C> for LimitedStorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Field<'this> = StorageDecoder<R::Mut<'this>, F>
    where
        Self: 'this;

    type StructPairs = Self;

    #[inline]
    fn size_hint(&self, cx: &C) -> SizeHint {
        MapDecoder::size_hint(self, cx)
    }

    #[inline]
    fn into_struct_pairs(self, _: &C) -> Result<Self::StructPairs, C::Error> {
        Ok(self)
    }

    #[inline]
    fn field(&mut self, cx: &C) -> Result<Option<Self::Field<'_>>, C::Error> {
        MapDecoder::entry(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapDecoder::end(self, cx)
    }
}

impl<'de, R, const F: Options, C> StructFieldDecoder<'de, C> for StorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type FieldName<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;
    type FieldValue = Self;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        MapEntryDecoder::map_key(self, cx)
    }

    #[inline]
    fn field_value(self, cx: &C) -> Result<Self::FieldValue, C::Error> {
        MapEntryDecoder::map_value(self, cx)
    }

    #[inline]
    fn skip_field_value(self, cx: &C) -> Result<bool, C::Error> {
        MapEntryDecoder::skip_map_value(self, cx)
    }
}

impl<'de, R, const F: Options, C> MapPairsDecoder<'de, C> for LimitedStorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type MapPairsKey<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;
    type MapPairsValue<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_map_pairs_value(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(false)
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, R, const F: Options, C> StructPairsDecoder<'de, C> for LimitedStorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type FieldName<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;
    type FieldValue<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(cx.message("Ran out of struct fields to decode"));
        }

        self.remaining -= 1;
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn field_value(&mut self, _: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_field_value(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(false)
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, R, const F: Options, C> VariantDecoder<'de, C> for StorageDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Tag<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;
    type Variant<'this> = StorageDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn tag(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn skip_variant(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(false)
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
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
