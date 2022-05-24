use core::fmt;

use crate::integer_encoding::{decode_typed_signed, decode_typed_unsigned};
use crate::tag::VARIANT;
use crate::tag::{Kind, F32, F64, I128, I16, I32, I64, I8, U128, U16, U32, U64, U8};
use crate::tag::{Tag, ABSENT, FALSE, PRESENT, TRUE};
use musli::de::NumberHint;
use musli::de::{
    Decoder, LengthHint, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, TypeHint,
    ValueVisitor, VariantDecoder,
};
use musli::error::Error;
use musli::never::Never;
use musli_common::encoding::Variable;
use musli_common::int::continuation as c;
use musli_common::reader::{Limit, PosReader};
use musli_storage::de::StorageDecoder;
use musli_storage::integer_encoding::UsizeEncoding;

/// A very simple decoder.
pub struct SelfDecoder<R> {
    reader: R,
}

impl<R> SelfDecoder<R> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<'de, R> SelfDecoder<R>
where
    R: PosReader<'de>,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any(&mut self) -> Result<(), R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Number => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, u128>(self.reader.borrow_mut())?;
                }
            }
            Kind::Marker => (),
            Kind::Bytes => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    Variable::decode_usize(self.reader.borrow_mut())?
                };

                self.reader.skip(len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    Variable::decode_usize(self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any()?;
                }
            }
            Kind::Map => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    Variable::decode_usize(self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any()?;
                    self.skip_any()?;
                }
            }
            Kind::Variant => {
                self.skip_any()?;
                self.skip_any()?;
            }
            kind => {
                return Err(R::Error::message(format_args!("unsupported kind {kind:?}")));
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
                Variable::decode_usize(self.reader.borrow_mut())?
            }),
            _ => Err(R::Error::message(Expected {
                expected: Kind::Sequence,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            })),
        }
    }

    #[inline]
    fn decode_map_len(&mut self) -> Result<usize, R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Map => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                Variable::decode_usize(self.reader.borrow_mut())?
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
    fn shared_decode_map(mut self) -> Result<RemainingSelfDecoder<R>, R::Error> {
        let len = self.decode_map_len()?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(mut self) -> Result<RemainingSelfDecoder<R>, R::Error> {
        let len = self.decode_sequence_len()?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix(&mut self, pos: usize) -> Result<usize, R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag.kind() != Kind::Bytes {
            return Err(R::Error::message(Expected {
                expected: Kind::Bytes,
                actual: tag,
                pos,
            }));
        }

        Ok(if let Some(len) = tag.data() {
            len as usize
        } else {
            Variable::decode_usize(self.reader.borrow_mut())?
        })
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct RemainingSelfDecoder<R> {
    remaining: usize,
    decoder: SelfDecoder<R>,
}

impl<'de, R> Decoder<'de> for SelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Buffer = Never<R::Error>;
    type Pack = SelfDecoder<Limit<R>>;
    type Some = Self;
    type Sequence = RemainingSelfDecoder<R>;
    type Tuple = Self;
    type Map = RemainingSelfDecoder<R>;
    type Struct = RemainingSelfDecoder<R>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire decoder")
    }

    #[inline]
    fn type_hint(&mut self) -> Result<TypeHint, Self::Error> {
        let tag = match self.reader.peek()? {
            Some(b) => Tag::from_byte(b),
            None => return Ok(TypeHint::Any),
        };

        match tag.kind() {
            Kind::Number => Ok(TypeHint::Number(match tag.data() {
                Some(U8) => NumberHint::U8,
                Some(U16) => NumberHint::U16,
                Some(U32) => NumberHint::U32,
                Some(U64) => NumberHint::U64,
                Some(U128) => NumberHint::U128,
                Some(I8) => NumberHint::I8,
                Some(I16) => NumberHint::I16,
                Some(I32) => NumberHint::I32,
                Some(I64) => NumberHint::I64,
                Some(I128) => NumberHint::I128,
                Some(F32) => NumberHint::F32,
                Some(F64) => NumberHint::F64,
                _ => NumberHint::Any,
            })),
            Kind::Sequence => {
                let hint = tag
                    .data()
                    .map(|d| LengthHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Sequence(hint))
            }
            Kind::Map => {
                let hint = tag
                    .data()
                    .map(|d| LengthHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Map(hint))
            }
            Kind::Bytes => {
                let hint = tag
                    .data()
                    .map(|d| LengthHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Bytes(hint))
            }
            Kind::Variant => Ok(TypeHint::Variant),
            Kind::Marker => Ok(match tag.data() {
                Some(TRUE | FALSE) => TypeHint::Bool,
                _ => TypeHint::Any,
            }),
            _ => Ok(TypeHint::Any),
        }
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
        Ok(SelfDecoder::new(self.reader.limit(len)))
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

        if tag.kind() != Kind::Bytes {
            return Err(Self::Error::message(Expected {
                expected: Kind::Bytes,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        let len = if let Some(len) = tag.data() {
            len as usize
        } else {
            Variable::decode_usize(self.reader.borrow_mut())?
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

            #[cfg(feature = "std")]
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
        const FALSE_TAG: Tag = Tag::new(Kind::Marker, FALSE);
        const TRUE_TAG: Tag = Tag::new(Kind::Marker, TRUE);

        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag {
            FALSE_TAG => Ok(false),
            TRUE_TAG => Ok(true),
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
    fn decode_u8(self) -> Result<u8, Self::Error> {
        decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        decode_typed_signed(self.reader)
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        decode_typed_unsigned(self.reader)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        decode_typed_signed(self.reader)
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
        const NONE: Tag = Tag::new(Kind::Marker, ABSENT);
        const SOME: Tag = Tag::new(Kind::Marker, PRESENT);

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
        self.shared_decode_map()
    }

    #[inline]
    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        self.shared_decode_map()
    }

    #[inline]
    fn decode_variant(mut self) -> Result<Self::Variant, Self::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag != Tag::new(Kind::Marker, VARIANT) {
            return Err(Self::Error::message(Expected {
                expected: Kind::Marker,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        Ok(self)
    }
}

impl<'de, R> PackDecoder<'de> for SelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, Variable, Variable> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'de, R> RemainingSelfDecoder<R>
where
    R: PosReader<'de>,
{
    #[inline]
    fn new(remaining: usize, decoder: SelfDecoder<R>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'de, R> SequenceDecoder<'de> for RemainingSelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Decoder<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;

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
        Ok(Some(SelfDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }

    #[inline]
    fn end(mut self) -> Result<(), Self::Error> {
        // Skip remaining elements.
        while let Some(mut item) = SequenceDecoder::next(&mut self)? {
            item.skip_any()?;
        }

        Ok(())
    }
}

impl<'a, 'de, R> PairDecoder<'de> for SelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type First<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;
    type Second = Self;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
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

impl<'a, 'de, R> VariantDecoder<'de> for SelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Tag<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;
    type Variant<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn skip_variant(&mut self) -> Result<bool, Self::Error> {
        self.skip_any()?;
        Ok(true)
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'de, R> PairsDecoder<'de> for RemainingSelfDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;

    type Decoder<'this> = SelfDecoder<R::PosMut<'this>>
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
        Ok(Some(SelfDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }

    #[inline]
    fn end(mut self) -> Result<(), Self::Error> {
        // Skip remaining elements.
        while let Some(mut item) = PairsDecoder::next(&mut self)? {
            item.skip_any()?;
        }

        Ok(())
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

        write!(f, "expected option marker, was {tag:?} (at {pos})",)
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
