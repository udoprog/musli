use core::fmt;
use core::marker;

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use crate::tag::Kind;
use crate::tag::Tag;
use musli::de::{
    Decoder, MapDecoder, MapEntryDecoder, PackDecoder, PairDecoder, ReferenceVisitor,
    SequenceDecoder, StructDecoder,
};
use musli::error::Error;
use musli_binary_common::int::continuation as c;
use musli_binary_common::reader::{Reader, WithPosition};

/// A very simple decoder.
pub struct WireDecoder<'de, R, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    reader: &'de mut WithPosition<R>,
    _marker: marker::PhantomData<(I, L)>,
}

impl<'de, R, I, L> WireDecoder<'de, R, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: &'de mut WithPosition<R>) -> Self {
        Self {
            reader,
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, 'a, R, I, L> WireDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any(&mut self) -> Result<(), R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Byte => {
                if tag.data().is_none() {
                    self.reader.skip(1)?;
                }
            }
            Kind::Prefix => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    L::decode_usize(&mut *self.reader)?
                };

                self.reader.skip(len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    L::decode_usize(&mut *self.reader)?
                };

                for _ in 0..len {
                    self.skip_any()?;
                }
            }
            Kind::Continuation => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, u128>(&mut *self.reader)?;
                }
            }
        }

        Ok(())
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(self) -> Result<RemainingSimpleDecoder<'a, R, I, L>, R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Sequence => {
                if let Some(len) = tag.data() {
                    RemainingSimpleDecoder::with_len(self, len as usize)
                } else {
                    RemainingSimpleDecoder::sequence(self)
                }
            }
            _ => Err(R::Error::collect_from_display(Expected(
                Kind::Sequence,
                self.reader.pos(),
            ))),
        }
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_pair_sequence(self) -> Result<RemainingSimpleDecoder<'a, R, I, L>, R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Sequence => {
                if let Some(len) = tag.data() {
                    RemainingSimpleDecoder::with_len(self, (len / 2) as usize)
                } else {
                    RemainingSimpleDecoder::pairs(self)
                }
            }
            _ => Err(R::Error::collect_from_display(Expected(
                Kind::Sequence,
                self.reader.pos(),
            ))),
        }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct RemainingSimpleDecoder<'a, R, I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    remaining: usize,
    decoder: WireDecoder<'a, R, I, L>,
}

impl<'de, 'a, R, I, L> Decoder<'de> for WireDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Pack = Self;
    type Some = Self;
    type Sequence = RemainingSimpleDecoder<'a, R, I, L>;
    type Map = RemainingSimpleDecoder<'a, R, I, L>;
    type Struct = RemainingSimpleDecoder<'a, R, I, L>;
    type Tuple = RemainingSimpleDecoder<'a, R, I, L>;
    type Variant = Self;

    #[inline]
    fn decode_unit(mut self) -> Result<(), Self::Error> {
        self.skip_any()?;
        Ok(())
    }

    #[inline]
    fn decode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        self.reader.read_array()
    }

    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag.kind() != Kind::Prefix {
            return Err(Self::Error::collect_from_display(Expected(
                Kind::Prefix,
                self.reader.pos(),
            )));
        }

        let len = if let Some(len) = tag.data() {
            len as usize
        } else {
            L::decode_usize(&mut *self.reader)?
        };

        let bytes = self.reader.read_bytes(len)?;
        visitor.visit_ref(bytes)
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
            fn expected(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expected(f)
            }

            #[inline]
            fn visit_ref(self, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
                let string = core::str::from_utf8(bytes).map_err(Self::Error::custom)?;
                self.0.visit_ref(string)
            }

            #[inline]
            fn visit(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                let string = core::str::from_utf8(bytes).map_err(Self::Error::custom)?;
                self.0.visit(string)
            }
        }
    }

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        match self.decode_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(Self::Error::custom(format!(
                "bad boolean, expected byte 1 or 0 but was {}",
                b
            ))),
        }
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        let num = self.decode_u32()?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(Self::Error::custom("bad character")),
        }
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        let b = Tag::from_byte(self.reader.read_byte()?);

        if b.kind() != Kind::Byte {
            return Err(Self::Error::collect_from_display(Expected(
                Kind::Byte,
                self.reader.pos(),
            )));
        }

        if let Some(b) = b.data() {
            Ok(b)
        } else {
            self.reader.read_byte()
        }
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
        L::decode_typed_usize(self.reader)
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
    fn decode_option(self) -> Result<Option<Self::Some>, Self::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Sequence => match tag.data_raw() {
                0 => Ok(None),
                1 => Ok(Some(self)),
                _ => Err(Self::Error::collect_from_display(Expected(
                    Kind::Sequence,
                    self.reader.pos(),
                ))),
            },
            _ => Err(Self::Error::collect_from_display(Expected(
                Kind::Sequence,
                self.reader.pos(),
            ))),
        }
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        self.shared_decode_sequence()
    }

    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        self.shared_decode_pair_sequence()
    }

    #[inline]
    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        self.shared_decode_pair_sequence()
    }

    #[inline]
    fn decode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        self.shared_decode_pair_sequence()
    }

    #[inline]
    fn decode_unit_struct(mut self) -> Result<(), Self::Error> {
        self.skip_any()?;
        Ok(())
    }

    #[inline]
    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
        if Tag::from_byte(self.reader.read_byte()?) != Tag::new(Kind::Sequence, 2) {
            return Err(Self::Error::collect_from_display(Expected(
                Kind::Sequence,
                self.reader.pos(),
            )));
        }

        Ok(self)
    }
}

impl<'de, R, I, L> PackDecoder<'de> for WireDecoder<'_, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = WireDecoder<'this, R, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        Ok(WireDecoder::new(self.reader))
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'de, 'a, R, I, L> RemainingSimpleDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    #[inline]
    fn sequence(decoder: WireDecoder<'a, R, I, L>) -> Result<Self, R::Error> {
        let remaining = L::decode_usize(&mut *decoder.reader)?;
        Ok(Self { remaining, decoder })
    }

    #[inline]
    fn pairs(decoder: WireDecoder<'a, R, I, L>) -> Result<Self, R::Error> {
        let remaining = L::decode_usize(&mut *decoder.reader)?;
        Ok(Self {
            remaining: remaining / 2,
            decoder,
        })
    }

    #[inline]
    fn with_len(decoder: WireDecoder<'a, R, I, L>, remaining: usize) -> Result<Self, R::Error> {
        Ok(Self { remaining, decoder })
    }
}

impl<'a, 'de, R, I, L> SequenceDecoder<'de> for RemainingSimpleDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Next<'this> = WireDecoder<'this, R, I, L> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Option<Self::Next<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader)))
    }
}

impl<'a, 'de, R, I, L> MapDecoder<'de> for RemainingSimpleDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;

    type Entry<'this> = WireDecoder<'this, R, I, L>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::Entry<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader)))
    }
}

impl<'a, 'de, R, I, L> MapEntryDecoder<'de> for WireDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type Key<'this> = WireDecoder<'this, R, I, L> where Self: 'this;
    type Value<'this> = WireDecoder<'this, R, I, L> where Self: 'this;

    #[inline]
    fn decode_key(&mut self) -> Result<Self::Key<'_>, Self::Error> {
        Ok(WireDecoder::new(self.reader))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::Value<'_>, Self::Error> {
        Ok(WireDecoder::new(self.reader))
    }
}

impl<'a, 'de, R, I, L> PairDecoder<'de> for WireDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;
    type First<'this> = WireDecoder<'this, R, I, L> where Self: 'this;
    type Second = Self;

    #[inline]
    fn decode_first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(WireDecoder::new(self.reader))
    }

    #[inline]
    fn decode_second(self) -> Result<Self::Second, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn skip_second(mut self) -> Result<bool, Self::Error> {
        self.skip_any()?;
        Ok(true)
    }
}

impl<'a, 'de, R, I, L> StructDecoder<'de> for RemainingSimpleDecoder<'a, R, I, L>
where
    R: Reader<'de>,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    type Error = R::Error;

    type Field<'this> = WireDecoder<'this, R, I, L>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }

    #[inline]
    fn decode_field(&mut self) -> Result<Option<Self::Field<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader)))
    }
}

struct Expected(Kind, Option<usize>);

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pos) = self.1 {
            write!(f, "Expected {:?} (at {})", self.0, pos)
        } else {
            write!(f, "Expected {:?}", self.0)
        }
    }
}

struct UnsupportedMark(Option<u8>, Option<usize>);

impl fmt::Display for UnsupportedMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pos) = self.1 {
            write!(f, "Unsupported mark {:?} (at {})", self.0, pos)
        } else {
            write!(f, "Unsupported mark {:?}", self.0)
        }
    }
}
