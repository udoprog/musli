use core::{fmt, marker};

use crate::integer_encoding::{IntegerEncoding, UsizeEncoding};
use crate::types::TypeTag;
use musli::de::{
    Decoder, MapDecoder, MapEntryDecoder, PackDecoder, PairDecoder, SequenceDecoder, StructDecoder,
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
        let b = self.reader.read_byte()?;

        // Special case: MSB is set indicating that the rest of the bits are the
        // payload.
        if b & TypeTag::Fixed8 as u8 == TypeTag::Fixed8 as u8 {
            if b == TypeTag::Fixed8Next as u8 {
                self.reader.skip(1)?;
            }

            return Ok(());
        }

        match b {
            TypeTag::CONTINUATION_BYTE => {
                let _ = c::decode::<_, u128>(&mut *self.reader)?;
            }
            TypeTag::SEQUENCE_BYTE => {
                let len = L::decode_usize(&mut *self.reader)?;

                // Skip over all values in the sequence.
                for _ in 0..len {
                    self.skip_any()?;
                }
            }
            TypeTag::PAIR_SEQUENCE_BYTE => {
                let len = L::decode_usize(&mut *self.reader)?;

                for _ in 0..len {
                    // Skip field.
                    self.skip_any()?;
                    // Skip field value.
                    self.skip_any()?;
                }
            }
            TypeTag::PAIR_BYTE => {
                self.skip_any()?;
                self.skip_any()?;
            }
            TypeTag::PREFIXED_BYTE => {
                let len = L::decode_usize(&mut *self.reader)?;
                self.reader.skip(len)?;
            }
            TypeTag::FIXED16_BYTE => {
                self.reader.skip(2)?;
            }
            TypeTag::FIXED32_BYTE => {
                self.reader.skip(4)?;
            }
            TypeTag::FIXED64_BYTE => {
                self.reader.skip(8)?;
            }
            TypeTag::FIXED128_BYTE => {
                self.reader.skip(16)?;
            }
            TypeTag::OPTION_NONE_BYTE => {
                // Nothing follows this tag.
            }
            TypeTag::OPTION_SOME_BYTE => {
                self.skip_any()?;
            }
            other => {
                return Err(R::Error::custom(format!(
                    "unexpected type tag {:08b}",
                    other
                )));
            }
        }

        Ok(())
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
    fn decode_bytes(self) -> Result<&'de [u8], Self::Error> {
        if self.reader.read_byte()? != TypeTag::Prefixed as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::Prefixed,
                self.reader.pos(),
            )));
        }

        let len = L::decode_usize(&mut *self.reader)?;
        self.reader.read_bytes(len)
    }

    #[inline]
    fn decode_str(self) -> Result<&'de str, Self::Error> {
        let bytes = self.decode_bytes()?;
        core::str::from_utf8(bytes).map_err(Self::Error::custom)
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
        let b = self.reader.read_byte()?;

        Ok(if b == TypeTag::Fixed8Next as u8 {
            self.reader.read_byte()?
        } else {
            b & !(TypeTag::Fixed8 as u8)
        })
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
        let b = self.reader.read_byte()?;

        if b & TypeTag::OptionNone as u8 != TypeTag::OptionNone as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::OptionSome,
                self.reader.pos(),
            )));
        }

        Ok(if b == TypeTag::OptionSome as u8 {
            Some(self)
        } else {
            None
        })
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        if self.reader.read_byte()? != TypeTag::Sequence as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::Sequence,
                self.reader.pos(),
            )));
        }

        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        if self.reader.read_byte()? != TypeTag::PairSequence as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::PairSequence,
                self.reader.pos(),
            )));
        }

        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        if self.reader.read_byte()? != TypeTag::PairSequence as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::PairSequence,
                self.reader.pos(),
            )));
        }

        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        if self.reader.read_byte()? != TypeTag::PairSequence as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::PairSequence,
                self.reader.pos(),
            )));
        }

        RemainingSimpleDecoder::new(self)
    }

    #[inline]
    fn decode_unit_struct(mut self) -> Result<(), Self::Error> {
        if self.reader.read_byte()? != TypeTag::PairSequence as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::PairSequence,
                self.reader.pos(),
            )));
        }

        let len = L::decode_usize(&mut *self.reader)?;

        // Skip over fields.
        for _ in 0..len {
            self.skip_any()?;
            self.skip_any()?;
        }

        Ok(())
    }

    #[inline]
    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
        if self.reader.read_byte()? != TypeTag::Pair as u8 {
            return Err(Self::Error::collect_from_display(Expected(
                TypeTag::Pair,
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

    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        Ok(WireDecoder::new(self.reader))
    }

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
    fn new(decoder: WireDecoder<'a, R, I, L>) -> Result<Self, R::Error> {
        let remaining = L::decode_usize(&mut *decoder.reader)?;
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

struct Expected(TypeTag, Option<usize>);

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pos) = self.1 {
            write!(f, "Expected {:?} (at {})", self.0, pos)
        } else {
            write!(f, "Expected {:?}", self.0)
        }
    }
}
