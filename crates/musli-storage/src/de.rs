use core::fmt;
use core::marker;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, SizeHint, ValueVisitor,
    VariantDecoder,
};
use musli::Context;
use musli_common::int::{IntegerEncoding, UsizeEncoding};
use musli_common::reader::PosReader;

/// A very simple decoder suitable for storage decoding.
pub struct StorageDecoder<R, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    reader: R,
    _marker: marker::PhantomData<(I, L)>,
}

impl<R, I, L> StorageDecoder<R, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
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
pub struct LimitedStorageDecoder<R, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    remaining: usize,
    decoder: StorageDecoder<R, I, L>,
}

#[musli::decoder]
impl<'de, R, I, L> Decoder<'de> for StorageDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Pack = Self;
    type Some = Self;
    type Sequence = LimitedStorageDecoder<R, I, L>;
    type Tuple = Self;
    type Map = LimitedStorageDecoder<R, I, L>;
    type Struct = LimitedStorageDecoder<R, I, L>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage decoder")
    }

    #[inline(always)]
    fn decode_unit<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let count = L::decode_usize(cx, self.reader.pos_borrow_mut())?;

        if count != 0 {
            return Err(cx.message(ExpectedEmptySequence { actual: count, pos }));
        }

        Ok(())
    }

    #[inline(always)]
    fn decode_pack<C>(self, _: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline(always)]
    fn decode_array<C, const N: usize>(mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.reader.read_array(cx)
    }

    #[inline(always)]
    fn decode_bytes<V>(
        mut self,
        cx: &mut V::Context,
        visitor: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let len = L::decode_usize(cx, self.reader.pos_borrow_mut())?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline(always)]
    fn decode_string<V>(
        self,
        cx: &mut V::Context,
        visitor: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        struct Visitor<V>(V);

        impl<'de, V> ValueVisitor<'de> for Visitor<V>
        where
            V: ValueVisitor<'de, Target = str>,
        {
            type Context = V::Context;
            type Target = [u8];
            type Ok = V::Ok;
            type Error = V::Error;

            #[inline(always)]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline(always)]
            fn visit_owned(
                self,
                cx: &mut Self::Context,
                bytes: Vec<u8>,
            ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
                let string =
                    musli_common::str::from_utf8_owned(bytes).map_err(|error| cx.custom(error))?;
                self.0.visit_owned(cx, string)
            }

            #[inline(always)]
            fn visit_borrowed(
                self,
                cx: &mut Self::Context,
                bytes: &'de [u8],
            ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
                let string =
                    musli_common::str::from_utf8(bytes).map_err(|error| cx.custom(error))?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline(always)]
            fn visit_ref(
                self,
                cx: &mut Self::Context,
                bytes: &[u8],
            ) -> Result<Self::Ok, <Self::Context as Context>::Error> {
                let string =
                    musli_common::str::from_utf8(bytes).map_err(|error| cx.custom(error))?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(cx, Visitor(visitor))
    }

    #[inline(always)]
    fn decode_bool<C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let byte = self.reader.read_byte(cx)?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(cx.message(BadBoolean { actual: b, pos })),
        }
    }

    #[inline(always)]
    fn decode_char<C>(self, cx: &mut C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let num = self.decode_u32(cx)?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.message(BadCharacter { actual: num, pos })),
        }
    }

    #[inline(always)]
    fn decode_u8<C>(mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.reader.read_byte(cx)
    }

    #[inline(always)]
    fn decode_u16<C>(self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u32<C>(self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u64<C>(self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u128<C>(self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i8<C>(self, cx: &mut C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self.decode_u8(cx)? as i8)
    }

    #[inline(always)]
    fn decode_i16<C>(self, cx: &mut C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i32<C>(self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i64<C>(self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i128<C>(self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_usize<C>(self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::decode_usize(cx, self.reader)
    }

    #[inline(always)]
    fn decode_isize<C>(self, cx: &mut C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self.decode_usize(cx)? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f32<C>(self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f64<C>(self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option<C>(mut self, cx: &mut C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let b = self.reader.read_byte(cx)?;
        Ok(if b == 1 { Some(self) } else { None })
    }

    #[inline]
    fn decode_sequence<C>(self, cx: &mut C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_tuple<C>(self, _: &mut C, _: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn decode_map<C>(self, cx: &mut C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_struct<C>(self, cx: &mut C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        LimitedStorageDecoder::new(cx, self)
    }

    #[inline]
    fn decode_variant<C>(self, _: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }
}

impl<'de, R, I, L> PackDecoder<'de> for StorageDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, I, L> LimitedStorageDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    #[inline]
    fn new<C>(cx: &mut C, mut decoder: StorageDecoder<R, I, L>) -> Result<Self, C::Error>
    where
        C: Context<Input = R::Error>,
    {
        let remaining = L::decode_usize(cx, &mut decoder.reader)?;
        Ok(Self { remaining, decoder })
    }
}

impl<'de, R, I, L> SequenceDecoder<'de> for LimitedStorageDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, I, L> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(
            self.decoder.reader.pos_borrow_mut(),
        )))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, I, L> PairsDecoder<'de> for LimitedStorageDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;

    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, I, L>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(
            self.decoder.reader.pos_borrow_mut(),
        )))
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, I, L> PairDecoder<'de> for StorageDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type First<'this> = StorageDecoder<R::PosMut<'this>, I, L> where Self: 'this;
    type Second = Self;

    #[inline]
    fn first<C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn second<C>(self, _: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn skip_second<C>(self, _: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(false)
    }
}

impl<'de, R, I, L> VariantDecoder<'de> for StorageDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Tag<'this> = StorageDecoder<R::PosMut<'this>, I, L> where Self: 'this;
    type Variant<'this> = StorageDecoder<R::PosMut<'this>, I, L> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn skip_variant<C>(&mut self, _: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(false)
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

struct ExpectedEmptySequence {
    actual: usize,
    pos: usize,
}

impl fmt::Display for ExpectedEmptySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, pos } = *self;
        write!(f, "Expected empty sequence, but was {actual} (at {pos})",)
    }
}

struct BadBoolean {
    actual: u8,
    pos: usize,
}

impl fmt::Display for BadBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, pos } = *self;
        write!(f, "Bad boolean byte 0x{actual:02x} (at {pos})")
    }
}

struct BadCharacter {
    actual: u32,
    pos: usize,
}

impl fmt::Display for BadCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, pos } = *self;
        write!(f, "Bad character number {actual} (at {pos})")
    }
}
