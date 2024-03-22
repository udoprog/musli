use core::fmt;
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, PackDecoder, SequenceDecoder,
    SizeHint, StructDecoder, StructFieldDecoder, StructFieldsDecoder, ValueVisitor, VariantDecoder,
};
use musli::Context;

use crate::options::Options;
use crate::reader::Reader;

/// A very simple decoder suitable for storage decoding.
pub struct StorageDecoder<R, const F: Options, C: ?Sized> {
    reader: R,
    _marker: PhantomData<C>,
}

impl<R, const F: Options, C: ?Sized> StorageDecoder<R, F, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            _marker: PhantomData,
        }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct LimitedStorageDecoder<R, const F: Options, C: ?Sized> {
    remaining: usize,
    decoder: StorageDecoder<R, F, C>,
}

#[musli::decoder]
impl<'de, R, const F: Options, C: ?Sized + Context> Decoder<'de> for StorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<U> = StorageDecoder<R, F, U> where U: Context;
    type DecodePack = Self;
    type DecodeSome = Self;
    type DecodeSequence = LimitedStorageDecoder<R, F, C>;
    type DecodeTuple = Self;
    type DecodeMap = LimitedStorageDecoder<R, F, C>;
    type DecodeStruct = LimitedStorageDecoder<R, F, C>;
    type DecodeVariant = Self;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(StorageDecoder::new(self.reader))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage decoder")
    }

    #[inline]
    fn decode_unit(mut self, cx: &C) -> Result<(), C::Error> {
        let mark = cx.mark();
        let count = musli_common::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?;

        if count != 0 {
            return Err(cx.marked_message(mark, ExpectedEmptySequence { actual: count }));
        }

        Ok(())
    }

    #[inline]
    fn decode_pack(self, _: &C) -> Result<Self::DecodePack, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_array<const N: usize>(mut self, cx: &C) -> Result<[u8; N], C::Error> {
        self.reader.read_array(cx)
    }

    #[inline]
    fn decode_bytes<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let len = musli_common::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
    {
        struct Visitor<V>(V);

        impl<'de, C, V> ValueVisitor<'de, C, [u8]> for Visitor<V>
        where
            C: ?Sized + Context,
            V: ValueVisitor<'de, C, str>,
        {
            type Ok = V::Ok;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline]
            fn visit_owned(self, cx: &C, bytes: Vec<u8>) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8_owned(bytes).map_err(cx.map())?;
                self.0.visit_owned(cx, string)
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline]
            fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(cx, Visitor(visitor))
    }

    #[inline]
    fn decode_bool(mut self, cx: &C) -> Result<bool, C::Error> {
        let mark = cx.mark();
        let byte = self.reader.read_byte(cx)?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(cx.marked_message(mark, BadBoolean { actual: b })),
        }
    }

    #[inline]
    fn decode_char(self, cx: &C) -> Result<char, C::Error> {
        let mark = cx.mark();
        let num = self.decode_u32(cx)?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.marked_message(mark, BadCharacter { actual: num })),
        }
    }

    #[inline]
    fn decode_u8(mut self, cx: &C) -> Result<u8, C::Error> {
        self.reader.read_byte(cx)
    }

    #[inline]
    fn decode_u16(self, cx: &C) -> Result<u16, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_u64(self, cx: &C) -> Result<u64, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_u128(self, cx: &C) -> Result<u128, C::Error> {
        musli_common::int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_i8(self, cx: &C) -> Result<i8, C::Error> {
        Ok(self.decode_u8(cx)? as i8)
    }

    #[inline]
    fn decode_i16(self, cx: &C) -> Result<i16, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_i32(self, cx: &C) -> Result<i32, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_i64(self, cx: &C) -> Result<i64, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_i128(self, cx: &C) -> Result<i128, C::Error> {
        musli_common::int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_usize(self, cx: &C) -> Result<usize, C::Error> {
        musli_common::int::decode_usize::<_, _, F>(cx, self.reader)
    }

    #[inline]
    fn decode_isize(self, cx: &C) -> Result<isize, C::Error> {
        Ok(self.decode_usize(cx)? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self, cx: &C) -> Result<f32, C::Error> {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self, cx: &C) -> Result<f64, C::Error> {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option(mut self, cx: &C) -> Result<Option<Self::DecodeSome>, C::Error> {
        let b = self.reader.read_byte(cx)?;
        Ok(if b == 1 { Some(self) } else { None })
    }

    #[inline]
    fn decode_sequence(self, cx: &C) -> Result<Self::DecodeSequence, C::Error> {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_tuple(self, _: &C, _: usize) -> Result<Self::DecodeTuple, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_map(self, cx: &C) -> Result<Self::DecodeMap, C::Error> {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_struct(self, cx: &C, _: Option<usize>) -> Result<Self::DecodeStruct, C::Error> {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_variant(self, _: &C) -> Result<Self::DecodeVariant, C::Error> {
        Ok(self)
    }
}

impl<'de, R, const F: Options, C: ?Sized + Context> PackDecoder<'de> for StorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_next(&mut self, _: &C) -> Result<Self::DecodeNext<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, R, const F: Options, C> LimitedStorageDecoder<R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    #[inline]
    fn new(cx: &C, mut decoder: StorageDecoder<R, F, C>) -> Result<Self, C::Error> {
        let remaining = musli_common::int::decode_usize::<_, _, F>(cx, &mut decoder.reader)?;
        Ok(Self { remaining, decoder })
    }
}

impl<'de, R, const F: Options, C: ?Sized + Context> SequenceDecoder<'de>
    for LimitedStorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn decode_next(&mut self, _: &C) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }
}

#[musli::map_decoder]
impl<'de, R, const F: Options, C: ?Sized + Context> MapDecoder<'de>
    for LimitedStorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeEntry<'this> = StorageDecoder<R::Mut<'this>, F, C>
    where
        Self: 'this;
    type IntoMapEntries = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn into_map_entries(self, _: &C) -> Result<Self::IntoMapEntries, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_entry(&mut self, _: &C) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }
}

impl<'de, R, const F: Options, C: ?Sized + Context> MapEntryDecoder<'de> for StorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeMapKey<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;
    type DecodeMapValue = Self;

    #[inline]
    fn decode_map_key(&mut self, _: &C) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_map_value(self, _: &C) -> Result<Self::DecodeMapValue, C::Error> {
        Ok(self)
    }

    #[inline]
    fn skip_map_value(self, _: &C) -> Result<bool, C::Error> {
        Ok(false)
    }
}

#[musli::struct_decoder]
impl<'de, R, const F: Options, C: ?Sized + Context> StructDecoder<'de>
    for LimitedStorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeField<'this> = StorageDecoder<R::Mut<'this>, F, C>
    where
        Self: 'this;

    type IntoStructFields = Self;

    #[inline]
    fn size_hint(&self, cx: &C) -> SizeHint {
        MapDecoder::size_hint(self, cx)
    }

    #[inline]
    fn into_struct_fields(self, _: &C) -> Result<Self::IntoStructFields, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_field(&mut self, cx: &C) -> Result<Option<Self::DecodeField<'_>>, C::Error> {
        MapDecoder::decode_entry(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapDecoder::end(self, cx)
    }
}

impl<'de, R, const F: Options, C: ?Sized + Context> StructFieldDecoder<'de>
    for StorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeFieldName<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;
    type DecodeFieldValue = Self;

    #[inline]
    fn decode_field_name(&mut self, cx: &C) -> Result<Self::DecodeFieldName<'_>, C::Error> {
        MapEntryDecoder::decode_map_key(self, cx)
    }

    #[inline]
    fn decode_field_value(self, cx: &C) -> Result<Self::DecodeFieldValue, C::Error> {
        MapEntryDecoder::decode_map_value(self, cx)
    }

    #[inline]
    fn skip_field_value(self, cx: &C) -> Result<bool, C::Error> {
        MapEntryDecoder::skip_map_value(self, cx)
    }
}

impl<'de, R, const F: Options, C: ?Sized + Context> MapEntriesDecoder<'de>
    for LimitedStorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeMapEntryKey<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;
    type DecodeMapEntryValue<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_map_entry_key(
        &mut self,
        _: &C,
    ) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn decode_map_entry_value(&mut self, _: &C) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_map_entry_value(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(false)
    }
}

impl<'de, R, const F: Options, C: ?Sized + Context> StructFieldsDecoder<'de>
    for LimitedStorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeStructFieldName<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;
    type DecodeStructFieldValue<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_struct_field_name(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(cx.message("Ran out of struct fields to decode"));
        }

        self.remaining -= 1;
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn decode_struct_field_value(
        &mut self,
        _: &C,
    ) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_struct_field_value(&mut self, _: &C) -> Result<bool, C::Error> {
        Ok(false)
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, R, const F: Options, C: ?Sized + Context> VariantDecoder<'de> for StorageDecoder<R, F, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeTag<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;
    type DecodeVariant<'this> = StorageDecoder<R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_tag(&mut self, _: &C) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self, _: &C) -> Result<Self::DecodeVariant<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn skip_value(&mut self, _: &C) -> Result<bool, C::Error> {
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
