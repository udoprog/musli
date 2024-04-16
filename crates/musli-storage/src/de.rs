use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    DecodeUnsized, Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, PackDecoder,
    SequenceDecoder, SizeHint, TupleDecoder, ValueVisitor, VariantDecoder,
};
use musli::hint::{MapHint, SequenceHint};
use musli::{Context, Decode};
use musli_utils::{Options, Reader};

/// A very simple decoder suitable for storage decoding.
pub struct StorageDecoder<'a, R, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    reader: R,
}

impl<'a, R, const OPT: Options, C: ?Sized> StorageDecoder<'a, R, OPT, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(cx: &'a C, reader: R) -> Self {
        Self { cx, reader }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct LimitedStorageDecoder<'a, R, const OPT: Options, C: ?Sized> {
    remaining: usize,
    cx: &'a C,
    reader: R,
}

#[musli::decoder]
impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> Decoder<'de>
    for StorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<'this, U> = StorageDecoder<'this, R, OPT, U> where U: 'this + Context;
    type DecodePack = Self;
    type DecodeSome = Self;
    type DecodeSequence = LimitedStorageDecoder<'a, R, OPT, C>;
    type DecodeTuple = Self;
    type DecodeMapHint = LimitedStorageDecoder<'a, R, OPT, C>;
    type DecodeMap = LimitedStorageDecoder<'a, R, OPT, C>;
    type DecodeMapEntries = LimitedStorageDecoder<'a, R, OPT, C>;
    type DecodeVariant = Self;

    fn cx(&self) -> &C {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(StorageDecoder::new(cx, self.reader))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage decoder")
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode>,
    {
        self.cx.decode(self)
    }

    #[inline]
    fn decode_unsized<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsized<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        self.cx.decode_unsized(self, f)
    }

    #[inline]
    fn decode_unit(mut self) -> Result<(), C::Error> {
        let mark = self.cx.mark();
        let count = musli_utils::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?;

        if count != 0 {
            return Err(self
                .cx
                .marked_message(mark, ExpectedEmptySequence { actual: count }));
        }

        Ok(())
    }

    #[inline]
    fn decode_pack<F, O>(mut self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, C::Error>,
    {
        f(&mut self)
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], C::Error> {
        self.reader.read_array(self.cx)
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let len = musli_utils::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?;
        self.reader.read_bytes(self.cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, C::Error>
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
                let string = crate::str::from_utf8_owned(bytes).map_err(cx.map())?;
                self.0.visit_owned(cx, string)
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline]
            fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(Visitor(visitor))
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, C::Error> {
        let mark = self.cx.mark();
        let byte = self.reader.read_byte(self.cx)?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(self.cx.marked_message(mark, BadBoolean { actual: b })),
        }
    }

    #[inline]
    fn decode_char(self) -> Result<char, C::Error> {
        let cx = self.cx;
        let mark = self.cx.mark();
        let num = self.decode_u32()?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.marked_message(mark, BadCharacter { actual: num })),
        }
    }

    #[inline]
    fn decode_u8(mut self) -> Result<u8, C::Error> {
        self.reader.read_byte(self.cx)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, C::Error> {
        musli_utils::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, C::Error> {
        musli_utils::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, C::Error> {
        musli_utils::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, C::Error> {
        musli_utils::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, C::Error> {
        Ok(self.decode_u8()? as i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, C::Error> {
        musli_utils::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, C::Error> {
        musli_utils::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, C::Error> {
        musli_utils::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, C::Error> {
        musli_utils::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, C::Error> {
        musli_utils::int::decode_usize::<_, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, C::Error> {
        Ok(self.decode_usize()? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self) -> Result<f32, C::Error> {
        Ok(f32::from_bits(self.decode_u32()?))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self) -> Result<f64, C::Error> {
        let bits = self.decode_u64()?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, C::Error> {
        let b = self.reader.read_byte(self.cx)?;
        Ok(if b == 1 { Some(self) } else { None })
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, C::Error>,
    {
        let cx = self.cx;
        let mut decoder = LimitedStorageDecoder::new(self.cx, self.reader)?;
        let output = f(&mut decoder)?;

        if decoder.remaining != 0 {
            return Err(cx.message("Caller did not decode all available map entries"));
        }

        Ok(output)
    }

    #[inline]
    fn decode_tuple<F, O>(mut self, _: &SequenceHint, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeTuple) -> Result<O, C::Error>,
    {
        f(&mut self)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, C::Error>,
    {
        let cx = self.cx;
        let mut decoder = LimitedStorageDecoder::new(self.cx, self.reader)?;
        let output = f(&mut decoder)?;

        if decoder.remaining != 0 {
            return Err(cx.message("Caller did not decode all available map entries"));
        }

        Ok(output)
    }

    #[inline]
    fn decode_map_hint<F, O>(self, _: &MapHint, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeMapHint) -> Result<O, C::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_map_entries(self) -> Result<Self::DecodeMapEntries, C::Error> {
        LimitedStorageDecoder::new(self.cx, self.reader)
    }

    #[inline]
    fn decode_variant<F, O>(mut self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, C::Error>,
    {
        f(&mut self)
    }
}

impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> PackDecoder<'de>
    for StorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> TupleDecoder<'de>
    for StorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        PackDecoder::decode_next(self)
    }
}

impl<'a, 'de, R, const OPT: Options, C> LimitedStorageDecoder<'a, R, OPT, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    #[inline]
    fn new(cx: &'a C, mut reader: R) -> Result<Self, C::Error> {
        let remaining = musli_utils::int::decode_usize::<_, _, OPT>(cx, reader.borrow_mut())?;
        Ok(Self {
            cx,
            reader,
            remaining,
        })
    }

    #[inline]
    fn with_remaining(cx: &'a C, reader: R, remaining: usize) -> Self {
        Self {
            cx,
            reader,
            remaining,
        }
    }
}

impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> SequenceDecoder<'de>
    for LimitedStorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;

        Ok(Some(StorageDecoder::new(self.cx, self.reader.borrow_mut())))
    }
}

impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> MapDecoder<'de>
    for LimitedStorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeEntry<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this> = LimitedStorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, <Self::Cx as Context>::Error> {
        Ok(LimitedStorageDecoder::with_remaining(
            self.cx,
            self.reader.borrow_mut(),
            self.remaining,
        ))
    }
}

impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> MapEntryDecoder<'de>
    for StorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeMapKey<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;
    type DecodeMapValue = Self;

    #[inline]
    fn decode_map_key(&mut self) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_map_value(self) -> Result<Self::DecodeMapValue, C::Error> {
        Ok(self)
    }
}

impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> MapEntriesDecoder<'de>
    for LimitedStorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeMapEntryKey<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;
    type DecodeMapEntryValue<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn decode_map_entry_key(&mut self) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_map_entry_value(&mut self) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end_map_entries(self) -> Result<(), C::Error> {
        if self.remaining != 0 {
            return Err(self
                .cx
                .message("Caller did not decode all available map entries"));
        }

        Ok(())
    }
}

impl<'a, 'de, R, const OPT: Options, C: ?Sized + Context> VariantDecoder<'de>
    for StorageDecoder<'a, R, OPT, C>
where
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeTag<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;
    type DecodeValue<'this> = StorageDecoder<'a, R::Mut<'this>, OPT, C> where Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
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
