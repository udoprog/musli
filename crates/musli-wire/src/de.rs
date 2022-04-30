use core::fmt;
use core::marker;

use crate::integer_encoding::{TypedIntegerEncoding, TypedUsizeEncoding};
use crate::tag::Kind;
use crate::tag::Tag;
use musli::de::{Decoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, ValueVisitor};
use musli::error::Error;
use musli_common::int::continuation as c;
use musli_common::reader::{Limit, PosReader};
use musli_storage::de::StorageDecoder;

/// A very simple decoder.
pub struct WireDecoder<R, I, L>
where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    reader: R,
    _marker: marker::PhantomData<(I, L)>,
}

impl<R, I, L> WireDecoder<R, I, L>
where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, R, I, L> WireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
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
                    L::decode_usize(self.reader.borrow_mut())?
                };

                self.reader.skip(len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    L::decode_usize(self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any()?;
                }
            }
            Kind::Continuation => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, u128>(self.reader.borrow_mut())?;
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn decode_sequence_len(&mut self) -> Result<usize, R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Sequence => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                L::decode_usize(self.reader.borrow_mut())?
            }),
            _ => Err(R::Error::message(Expected {
                expected: Kind::Sequence,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            })),
        }
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_pair_sequence(mut self) -> Result<RemainingWireDecoder<R, I, L>, R::Error> {
        let len = self.decode_sequence_len()?;
        Ok(RemainingWireDecoder::new(len / 2, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(mut self) -> Result<RemainingWireDecoder<R, I, L>, R::Error> {
        let len = self.decode_sequence_len()?;
        Ok(RemainingWireDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix(&mut self, pos: usize) -> Result<usize, R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag.kind() != Kind::Prefix {
            return Err(R::Error::message(Expected {
                expected: Kind::Prefix,
                actual: tag,
                pos,
            }));
        }

        Ok(if let Some(len) = tag.data() {
            len as usize
        } else {
            L::decode_usize(self.reader.borrow_mut())?
        })
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct RemainingWireDecoder<R, I, L>
where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    remaining: usize,
    decoder: WireDecoder<R, I, L>,
}

impl<'de, R, I, L> Decoder<'de> for WireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Error = R::Error;
    type Pack = WireDecoder<Limit<R>, I, L>;
    type Some = Self;
    type Sequence = RemainingWireDecoder<R, I, L>;
    type Tuple = Self;
    type Map = RemainingWireDecoder<R, I, L>;
    type Struct = RemainingWireDecoder<R, I, L>;
    type TupleStruct = RemainingWireDecoder<R, I, L>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire decoder")
    }

    #[inline]
    fn decode_unit(mut self) -> Result<(), Self::Error> {
        self.skip_any()?;
        Ok(())
    }

    #[inline]
    fn decode_pack(mut self) -> Result<Self::Pack, Self::Error> {
        let pos = self.reader.pos();
        let len = self.decode_prefix(pos)?;
        Ok(WireDecoder::new(self.reader.limit(len)))
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], Self::Error> {
        let pos = self.reader.pos();
        let len = self.decode_prefix(pos)?;

        if len != N {
            return Err(Self::Error::message(BadLength {
                actual: len,
                expected: N,
                pos,
            }));
        }

        self.reader.read_array()
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag.kind() != Kind::Prefix {
            return Err(Self::Error::message(Expected {
                expected: Kind::Prefix,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        let len = if let Some(len) = tag.data() {
            len as usize
        } else {
            L::decode_usize(self.reader.borrow_mut())?
        };

        self.reader.read_bytes(len, visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        return self.decode_bytes(Visitor(visitor));

        struct Visitor<V>(V);

        impl<'de, V> ValueVisitor<'de> for Visitor<V>
        where
            V: ValueVisitor<'de, Target = str>,
        {
            type Target = [u8];
            type Ok = V::Ok;
            type Error = V::Error;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[inline]
            fn visit_owned(self, bytes: Vec<u8>) -> Result<Self::Ok, Self::Error> {
                let string = String::from_utf8(bytes).map_err(Self::Error::custom)?;
                self.0.visit_owned(string)
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
        const FALSE: Tag = Tag::new(Kind::Byte, 0);
        const TRUE: Tag = Tag::new(Kind::Byte, 1);

        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(Self::Error::message(BadBoolean {
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            })),
        }
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        let num = self.decode_u32()?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(Self::Error::message(BadCharacter(num))),
        }
    }

    #[inline]
    fn decode_u8(mut self) -> Result<u8, Self::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag.kind() != Kind::Byte {
            return Err(Self::Error::message(Expected {
                expected: Kind::Byte,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        if let Some(b) = tag.data() {
            Ok(b)
        } else {
            self.reader.read_byte()
        }
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        I::decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        I::decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        I::decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        I::decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        Ok(self.decode_u8()? as i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        I::decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        I::decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        I::decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        I::decode_typed_signed(self.reader)
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
    fn decode_option(mut self) -> Result<Option<Self::Some>, Self::Error> {
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::new(Kind::Sequence, 0);
        const SOME: Tag = Tag::new(Kind::Sequence, 1);

        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(Self::Error::message(ExpectedOption {
                tag,
                pos: self.reader.pos().saturating_sub(1),
            })),
        }
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        self.shared_decode_sequence()
    }

    #[inline]
    fn decode_tuple(mut self, len: usize) -> Result<Self::Tuple, Self::Error> {
        let actual = self.decode_sequence_len()?;

        if len != actual {
            return Err(Self::Error::message(format_args!(
                "tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(self)
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
    fn decode_tuple_struct(self, _: usize) -> Result<Self::TupleStruct, Self::Error> {
        self.shared_decode_pair_sequence()
    }

    #[inline]
    fn decode_unit_struct(mut self) -> Result<(), Self::Error> {
        self.skip_any()?;
        Ok(())
    }

    #[inline]
    fn decode_variant(mut self) -> Result<Self::Variant, Self::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag != Tag::new(Kind::Sequence, 2) {
            return Err(Self::Error::message(Expected {
                expected: Kind::Sequence,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        Ok(self)
    }
}

impl<'de, R, I, L> PackDecoder<'de> for WireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }
}

impl<'de, R, I, L> RemainingWireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    #[inline]
    fn new(remaining: usize, decoder: WireDecoder<R, I, L>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'de, R, I, L> SequenceDecoder<'de> for RemainingWireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = WireDecoder<R::PosMut<'this>, I, L> where Self: 'this;

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
        Ok(Some(WireDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }
}

impl<'a, 'de, R, I, L> PairDecoder<'de> for WireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Error = R::Error;
    type First<'this> = WireDecoder<R::PosMut<'this>, I, L> where Self: 'this;
    type Second = WireDecoder<R, I, L>;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(WireDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn second(self) -> Result<Self::Second, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn skip_second(mut self) -> Result<bool, Self::Error> {
        self.skip_any()?;
        Ok(true)
    }
}

impl<'de, R, I, L> PairsDecoder<'de> for RemainingWireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    type Error = R::Error;

    type Decoder<'this> = WireDecoder<R::PosMut<'this>, I, L>
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
        Ok(Some(WireDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }
}

struct Expected {
    expected: Kind,
    actual: Tag,
    pos: usize,
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            expected,
            actual,
            pos,
        } = *self;

        write!(f, "Expected {expected:?} but was {actual:?} (at {pos})",)
    }
}

struct BadBoolean {
    actual: Tag,
    pos: usize,
}

impl fmt::Display for BadBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, pos } = *self;
        write!(f, "Bad boolean tag {actual:?} (at {pos})")
    }
}

struct BadCharacter(u32);

impl fmt::Display for BadCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bad character number 0x{:02x}", self.0)
    }
}

struct ExpectedOption {
    tag: Tag,
    pos: usize,
}

impl fmt::Display for ExpectedOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { tag, pos } = *self;

        write!(
            f,
            "Expected zero-to-single sequence, was {tag:?} (at {pos})",
        )
    }
}

struct BadLength {
    actual: usize,
    expected: usize,
    pos: usize,
}

impl fmt::Display for BadLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            actual,
            expected,
            pos,
        } = *self;

        write!(
            f,
            "Bad length, got {actual} but expect {expected} (at {pos})"
        )
    }
}
