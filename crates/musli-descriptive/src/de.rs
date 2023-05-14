use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, LengthHint, NumberHint, NumberVisitor, PackDecoder, PairDecoder, PairsDecoder,
    SequenceDecoder, TypeHint, ValueVisitor, VariantDecoder,
};
use musli::error::Error;
use musli::never::Never;
use musli_common::int::{continuation as c, UsizeEncoding, Variable};
use musli_common::reader::PosReader;
use musli_storage::de::StorageDecoder;

use crate::integer_encoding::{decode_typed_signed, decode_typed_unsigned};
use crate::tag::{Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, U128, U16, U32, U64, U8};

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

pub struct SelfPackDecoder<R> {
    reader: R,
    remaining: usize,
}

impl<R> SelfPackDecoder<R> {
    #[inline]
    pub(crate) fn new(reader: R, end: usize) -> Self {
        Self {
            reader,
            remaining: end,
        }
    }
}

pub struct SelfTupleDecoder<R> {
    reader: R,
}

impl<R> SelfTupleDecoder<R> {
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
            Kind::Mark => {
                if let Mark::Variant = tag.mark() {
                    self.skip_any()?;
                    self.skip_any()?;
                }
            }
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
            kind => {
                return Err(R::Error::message(format_args!("unsupported kind {kind:?}")));
            }
        }

        Ok(())
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_map(mut self) -> Result<RemainingSelfDecoder<R>, R::Error> {
        let pos = self.reader.pos();
        let len = self.decode_prefix(Kind::Map, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(mut self) -> Result<RemainingSelfDecoder<R>, R::Error> {
        let pos = self.reader.pos();
        let len = self.decode_prefix(Kind::Sequence, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix(&mut self, kind: Kind, pos: usize) -> Result<usize, R::Error> {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag.kind() != kind {
            return Err(R::Error::message(Expected {
                expected: kind,
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
    type Pack = SelfPackDecoder<R>;
    type Some = Self;
    type Sequence = RemainingSelfDecoder<R>;
    type Tuple = SelfTupleDecoder<R>;
    type Map = RemainingSelfDecoder<R>;
    type Struct = RemainingSelfDecoder<R>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the descriptive decoder")
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
            Kind::String => {
                let hint = tag
                    .data()
                    .map(|d| LengthHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::String(hint))
            }
            Kind::Mark => Ok(match tag.mark() {
                Mark::True | Mark::False => TypeHint::Bool,
                Mark::Variant => TypeHint::Variant,
                Mark::Some | Mark::None => TypeHint::Option,
                Mark::Char => TypeHint::Char,
                Mark::Unit => TypeHint::Unit,
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
        let len = self.decode_prefix(Kind::Bytes, pos)?;
        Ok(SelfPackDecoder::new(self.reader, len))
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], Self::Error> {
        let pos = self.reader.pos();
        let len = self.decode_prefix(Kind::Bytes, pos)?;

        if len != N {
            return Err(Self::Error::message(format_args! {
                "bad length, got {len} but expect {N} (at {pos})"
            }));
        }

        self.reader.read_array()
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(Kind::Bytes, pos)?;
        self.reader.read_bytes(len, visitor)
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(Kind::String, pos)?;
        return self.reader.read_bytes(len, Visitor(visitor));

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

            #[cfg(feature = "alloc")]
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
        const FALSE: Tag = Tag::from_mark(Mark::False);
        const TRUE: Tag = Tag::from_mark(Mark::True);

        let pos = self.reader.pos();
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(Self::Error::message(format_args! {
                "bad boolean, got {tag:?} (at {pos})"
            })),
        }
    }

    #[inline]
    fn decode_char(mut self) -> Result<char, Self::Error> {
        const CHAR: Tag = Tag::from_mark(Mark::Char);

        let pos = self.reader.pos();
        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag != CHAR {
            return Err(R::Error::message(format_args!(
                "expected {CHAR:?}, got {tag:?} (at {pos})"
            )));
        }

        let num = c::decode(self.reader.borrow_mut())?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(Self::Error::message(format_args!(
                "bad character (at {pos}"
            ))),
        }
    }

    #[inline]
    fn decode_number<V>(mut self, visitor: V) -> Result<V::Ok, Self::Error>
    where
        V: NumberVisitor<Error = Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag.kind() {
            Kind::Number => match tag.data() {
                Some(U8) => visitor.visit_u8(self.decode_u8()?),
                Some(U16) => visitor.visit_u16(self.decode_u16()?),
                Some(U32) => visitor.visit_u32(self.decode_u32()?),
                Some(U64) => visitor.visit_u64(self.decode_u64()?),
                Some(U128) => visitor.visit_u128(self.decode_u128()?),
                Some(I8) => visitor.visit_i8(self.decode_i8()?),
                Some(I16) => visitor.visit_i16(self.decode_i16()?),
                Some(I32) => visitor.visit_i32(self.decode_i32()?),
                Some(I64) => visitor.visit_i64(self.decode_i64()?),
                Some(I128) => visitor.visit_i128(self.decode_i128()?),
                Some(F32) => visitor.visit_f32(self.decode_f32()?),
                Some(F64) => visitor.visit_f64(self.decode_f64()?),
                _ => Err(Self::Error::message(format_args!(
                    "unsupported number tag, got {tag:?}"
                ))),
            },
            _ => Err(Self::Error::message(format_args!(
                "expected number, but got {tag:?}"
            ))),
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
        const NONE: Tag = Tag::from_mark(Mark::None);
        const SOME: Tag = Tag::from_mark(Mark::Some);

        let pos = self.reader.pos();
        let tag = Tag::from_byte(self.reader.read_byte()?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(Self::Error::message(format_args! {
                "expected option, was {tag:?} (at {pos})"
            })),
        }
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        self.shared_decode_sequence()
    }

    #[inline]
    fn decode_tuple(mut self, len: usize) -> Result<Self::Tuple, Self::Error> {
        let pos = self.reader.pos();
        let actual = self.decode_prefix(Kind::Sequence, pos)?;

        if len != actual {
            return Err(Self::Error::message(format_args!(
                "tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(SelfTupleDecoder::new(self.reader))
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
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);

        let tag = Tag::from_byte(self.reader.read_byte()?);

        if tag != VARIANT {
            return Err(Self::Error::message(Expected {
                expected: Kind::Mark,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        Ok(self)
    }
}

impl<'de, R> PackDecoder<'de> for SelfPackDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, Variable, Variable> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        self.remaining = match self.remaining.checked_sub(1) {
            Some(remaining) => remaining,
            None => return Err(Self::Error::message("tried to decode past the pack")),
        };

        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'de, R> PackDecoder<'de> for SelfTupleDecoder<R>
where
    R: PosReader<'de>,
{
    type Error = R::Error;
    type Decoder<'this> = SelfDecoder<R::PosMut<'this>> where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.reader.pos_borrow_mut()))
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

impl<'de, R> PairDecoder<'de> for SelfDecoder<R>
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

impl<'de, R> VariantDecoder<'de> for SelfDecoder<R>
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
