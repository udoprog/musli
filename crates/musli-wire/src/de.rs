use core::fmt;
use core::marker;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, SizeHint, ValueVisitor,
    VariantDecoder,
};
use musli::Context;
use musli_common::reader::{Limit, PosReader};
use musli_storage::de::StorageDecoder;
use musli_storage::int::{continuation as c, Variable};

use crate::integer_encoding::{WireIntegerEncoding, WireUsizeEncoding};
use crate::tag::Kind;
use crate::tag::Tag;

/// A very simple decoder.
pub struct WireDecoder<R, I, L>
where
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    reader: R,
    _marker: marker::PhantomData<(I, L)>,
}

impl<R, I, L> WireDecoder<R, I, L>
where
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
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
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any<'buf, C>(&mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = R::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Byte => {
                if tag.data().is_none() {
                    self.reader.skip(cx, 1)?;
                }
            }
            Kind::Prefix => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    L::decode_usize(cx, self.reader.borrow_mut())?
                };

                self.reader.skip(cx, len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    L::decode_usize(cx, self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any(cx)?;
                }
            }
            Kind::Continuation => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, _, u128>(cx, self.reader.borrow_mut())?;
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn decode_sequence_len<'buf, C>(&mut self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Sequence => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                L::decode_usize(cx, self.reader.borrow_mut())?
            }),
            _ => Err(cx.message(Expected {
                expected: Kind::Sequence,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            })),
        }
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_pair_sequence<'buf, C>(
        mut self,
        cx: &mut C,
    ) -> Result<RemainingWireDecoder<R, I, L>, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
    {
        let len = self.decode_sequence_len(cx)?;
        Ok(RemainingWireDecoder::new(len / 2, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence<'buf, C>(
        mut self,
        cx: &mut C,
    ) -> Result<RemainingWireDecoder<R, I, L>, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
    {
        let len = self.decode_sequence_len(cx)?;
        Ok(RemainingWireDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix<'buf, C>(&mut self, cx: &mut C, pos: usize) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = R::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag.kind() != Kind::Prefix {
            return Err(cx.message(Expected {
                expected: Kind::Prefix,
                actual: tag,
                pos,
            }));
        }

        Ok(if let Some(len) = tag.data() {
            len as usize
        } else {
            L::decode_usize(cx, self.reader.borrow_mut())?
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
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    remaining: usize,
    decoder: WireDecoder<R, I, L>,
}

#[musli::decoder]
impl<'de, R, I, L> Decoder<'de> for WireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = R::Error;
    type Pack = WireDecoder<Limit<R>, I, L>;
    type Some = Self;
    type Sequence = RemainingWireDecoder<R, I, L>;
    type Tuple = TupleWireDecoder<R, I, L>;
    type Map = RemainingWireDecoder<R, I, L>;
    type Struct = RemainingWireDecoder<R, I, L>;
    type Variant = Self;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire decoder")
    }

    #[inline(always)]
    fn decode_unit<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(())
    }

    #[inline(always)]
    fn decode_pack<'buf, C>(mut self, cx: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, pos)?;
        Ok(WireDecoder::new(self.reader.limit(len)))
    }

    #[inline(always)]
    fn decode_array<'buf, C, const N: usize>(mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, pos)?;

        if len != N {
            return Err(cx.message(BadLength {
                actual: len,
                expected: N,
                pos,
            }));
        }

        self.reader.read_array(cx)
    }

    #[inline(always)]
    fn decode_bytes<'buf, C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, [u8]>,
    {
        let pos = self.reader.pos();
        let len = self.decode_prefix(cx, pos)?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline(always)]
    fn decode_string<'buf, C, V>(self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, str>,
    {
        struct Visitor<V>(V);

        impl<'de, 'buf, C, V> ValueVisitor<'de, 'buf, C, [u8]> for Visitor<V>
        where
            C: Context<'buf>,
            V: ValueVisitor<'de, 'buf, C, str>,
        {
            type Ok = V::Ok;

            #[inline(always)]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline(always)]
            fn visit_owned(self, cx: &mut C, bytes: Vec<u8>) -> Result<Self::Ok, C::Error> {
                let string =
                    musli_common::str::from_utf8_owned(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_owned(cx, string)
            }

            #[inline(always)]
            fn visit_borrowed(self, cx: &mut C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline(always)]
            fn visit_ref(self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(cx, Visitor(visitor))
    }

    #[inline(always)]
    fn decode_bool<'buf, C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        const FALSE: Tag = Tag::new(Kind::Byte, 0);
        const TRUE: Tag = Tag::new(Kind::Byte, 1);

        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(cx.message(BadBoolean {
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            })),
        }
    }

    #[inline(always)]
    fn decode_char<'buf, C>(self, cx: &mut C) -> Result<char, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let num = self.decode_u32(cx)?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.message(BadCharacter(num))),
        }
    }

    #[inline(always)]
    fn decode_u8<'buf, C>(mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag.kind() != Kind::Byte {
            return Err(cx.message(Expected {
                expected: Kind::Byte,
                actual: tag,
                pos: self.reader.pos().saturating_sub(1),
            }));
        }

        if let Some(b) = tag.data() {
            Ok(b)
        } else {
            self.reader.read_byte(cx)
        }
    }

    #[inline(always)]
    fn decode_u16<'buf, C>(self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u32<'buf, C>(self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u64<'buf, C>(self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u128<'buf, C>(self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i8<'buf, C>(self, cx: &mut C) -> Result<i8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(self.decode_u8(cx)? as i8)
    }

    #[inline(always)]
    fn decode_i16<'buf, C>(self, cx: &mut C) -> Result<i16, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i32<'buf, C>(self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i64<'buf, C>(self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i128<'buf, C>(self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        I::decode_typed_signed(cx, self.reader)
    }

    #[inline(always)]
    fn decode_usize<'buf, C>(self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        L::decode_typed_usize(cx, self.reader)
    }

    #[inline(always)]
    fn decode_isize<'buf, C>(self, cx: &mut C) -> Result<isize, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(self.decode_usize(cx)? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f32<'buf, C>(self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f64<'buf, C>(self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline(always)]
    fn decode_option<'buf, C>(mut self, cx: &mut C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::new(Kind::Sequence, 0);
        const SOME: Tag = Tag::new(Kind::Sequence, 1);

        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(cx.message(ExpectedOption {
                tag,
                pos: self.reader.pos().saturating_sub(1),
            })),
        }
    }

    #[inline]
    fn decode_sequence<'buf, C>(self, cx: &mut C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.shared_decode_sequence(cx)
    }

    #[inline]
    fn decode_tuple<'buf, C>(mut self, cx: &mut C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let actual = self.decode_sequence_len(cx)?;

        if len != actual {
            return Err(cx.message(format_args!(
                "tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(TupleWireDecoder::new(self.reader, len))
    }

    #[inline]
    fn decode_map<'buf, C>(self, cx: &mut C) -> Result<Self::Map, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.shared_decode_pair_sequence(cx)
    }

    #[inline]
    fn decode_struct<'buf, C>(self, cx: &mut C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.shared_decode_pair_sequence(cx)
    }

    #[inline]
    fn decode_variant<'buf, C>(mut self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag != Tag::new(Kind::Sequence, 2) {
            return Err(cx.message(Expected {
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
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = StorageDecoder<R::PosMut<'this>, Variable, Variable> where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, I, L> RemainingWireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    #[inline]
    fn new(remaining: usize, decoder: WireDecoder<R, I, L>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'de, R, I, L> SequenceDecoder<'de> for RemainingWireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = WireDecoder<R::PosMut<'this>, I, L> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        // Skip remaining elements.
        while let Some(mut item) = SequenceDecoder::next(&mut self, cx)? {
            item.skip_any(cx)?;
        }

        Ok(())
    }
}

impl<'de, R, I, L> PairDecoder<'de> for WireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = R::Error;
    type First<'this> = WireDecoder<R::PosMut<'this>, I, L> where Self: 'this;
    type Second = Self;

    #[inline]
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn second<'buf, C>(self, _: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn skip_second<'buf, C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(true)
    }
}

impl<'de, R, I, L> VariantDecoder<'de> for WireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = R::Error;
    type Tag<'this> = WireDecoder<R::PosMut<'this>, I, L> where Self: 'this;
    type Variant<'this> = WireDecoder<R::PosMut<'this>, I, L> where Self: 'this;

    #[inline]
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn variant<'buf, C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(WireDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn skip_variant<'buf, C>(&mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, I, L> PairsDecoder<'de> for RemainingWireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = R::Error;

    type Decoder<'this> = WireDecoder<R::PosMut<'this>, I, L>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader.pos_borrow_mut())))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        // Skip remaining elements.
        while let Some(mut item) = PairsDecoder::next(&mut self, cx)? {
            item.skip_any(cx)?;
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

/// A tuple wire decoder.
pub struct TupleWireDecoder<R, I, L>
where
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    reader: R,
    remaining: usize,
    _marker: marker::PhantomData<(I, L)>,
}

impl<R, I, L> TupleWireDecoder<R, I, L>
where
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: R, remaining: usize) -> Self {
        Self {
            reader,
            remaining,
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, R, I, L> PackDecoder<'de> for TupleWireDecoder<R, I, L>
where
    R: PosReader<'de>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = R::Error;
    type Decoder<'this> = WireDecoder<R::PosMut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, cx: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Err(cx.message(format_args!("No more tuple elements to decode")));
        }

        self.remaining -= 1;
        Ok(WireDecoder::new(self.reader.pos_borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        while self.remaining > 0 {
            WireDecoder::<_, I, L>::new(self.reader.pos_borrow_mut()).skip_any(cx)?;
            self.remaining -= 1;
        }

        Ok(())
    }
}
