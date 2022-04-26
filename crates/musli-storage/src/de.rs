use core::fmt;
use core::marker;

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use musli::de::{
    Decoder, PackDecoder, PairDecoder, PairsDecoder, ReferenceVisitor, SequenceDecoder,
};
use musli::error::Error;
use musli_binary_common::reader::PositionedReader;

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
pub struct RemainingSimpleDecoder<R, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    remaining: usize,
    decoder: StorageDecoder<R, I, L>,
}

impl<'de, R, I, L> Decoder<'de> for StorageDecoder<R, I, L>
where
    R: PositionedReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Pack = Self;
    type Some = Self;
    type Sequence = RemainingSimpleDecoder<R, I, L>;
    type Map = RemainingSimpleDecoder<R, I, L>;
    type Struct = RemainingSimpleDecoder<R, I, L>;
    type Tuple = RemainingSimpleDecoder<R, I, L>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage decoder")
    }

    #[inline]
    fn decode_unit(mut self) -> Result<(), Self::Error> {
        let pos = self.reader.pos();
        let count = L::decode_usize(&mut self.reader)?;

        if count != 0 {
            return Err(Self::Error::message(ExpectedEmptySequence {
                actual: count,
                pos,
            }));
        }

        Ok(())
    }

    #[inline]
    fn decode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], Self::Error> {
        self.reader.read_array()
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let len = L::decode_usize(&mut self.reader)?;
        self.reader.read_bytes(len, visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = str, Error = Self::Error>,
    {
        return self.decode_bytes(Visitor(visitor));

        struct Visitor<V>(V);

        impl<'de, V> ReferenceVisitor<'de> for Visitor<V>
        where
            V: ReferenceVisitor<'de, Target = str>,
        {
            type Target = [u8];
            type Ok = V::Ok;
            type Error = V::Error;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[inline]
            fn visit_borrowed(self, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
                let string = core::str::from_utf8(bytes).map_err(Self::Error::custom)?;
                self.0.visit_borrowed(string)
            }

            #[inline]
            fn visit_any(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                let string = core::str::from_utf8(bytes).map_err(Self::Error::custom)?;
                self.0.visit_any(string)
            }
        }
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, Self::Error> {
        let pos = self.reader.pos();
        let byte = self.reader.read_byte()?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(Self::Error::message(BadBoolean { actual: b, pos })),
        }
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        let pos = self.reader.pos();
        let num = self.decode_u32()?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(Self::Error::message(BadCharacter { actual: num, pos })),
        }
    }

    #[inline]
    fn decode_u8(mut self) -> Result<u8, Self::Error> {
        self.reader.read_byte()
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        I::decode_unsigned(self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        I::decode_unsigned(self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        I::decode_unsigned(self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        I::decode_unsigned(self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        Ok(self.decode_u8()? as i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        I::decode_signed(self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        I::decode_signed(self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        I::decode_signed(self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        I::decode_signed(self.reader)
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        L::decode_usize(self.reader)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        Ok(self.decode_usize()? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        let bits = self.decode_u32()?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        let bits = self.decode_u64()?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::Some>, Self::Error> {
        let b = self.reader.read_byte()?;
        Ok(if b == 1 { Some(self) } else { None })
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_unit_struct(self) -> Result<(), Self::Error> {
        self.decode_unit()
    }

    #[inline]
    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
        Ok(self)
    }
}

impl<'de, R, I, L> PackDecoder<'de> for StorageDecoder<R, I, L>
where
    R: PositionedReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<&'this mut R, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        Ok(StorageDecoder::new(&mut self.reader))
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'de, R, I, L> RemainingSimpleDecoder<R, I, L>
where
    R: PositionedReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    #[inline]
    fn new(mut decoder: StorageDecoder<R, I, L>) -> Result<Self, R::Error> {
        let remaining = L::decode_usize(&mut decoder.reader)?;
        Ok(Self { remaining, decoder })
    }
}

impl<'de, R, I, L> SequenceDecoder<'de> for RemainingSimpleDecoder<R, I, L>
where
    R: PositionedReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<&'this mut R, I, L> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(&mut self.decoder.reader)))
    }
}

impl<'de, R, I, L> PairsDecoder<'de> for RemainingSimpleDecoder<R, I, L>
where
    R: PositionedReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;

    type Decoder<'this> = StorageDecoder<&'this mut R, I, L>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(&mut self.decoder.reader)))
    }
}

impl<'de, R, I, L> PairDecoder<'de> for StorageDecoder<R, I, L>
where
    R: PositionedReader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type First<'this> = StorageDecoder<&'this mut R, I, L> where Self: 'this;
    type Second = StorageDecoder<R, I, L>;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(StorageDecoder::new(&mut self.reader))
    }

    #[inline]
    fn second(self) -> Result<Self::Second, Self::Error> {
        Ok(StorageDecoder::new(self.reader))
    }

    #[inline]
    fn skip_second(self) -> Result<bool, Self::Error> {
        Ok(false)
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
