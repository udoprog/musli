use core::fmt;
use core::marker;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, SizeHint, ValueVisitor,
    VariantDecoder,
};
use musli::Context;
use musli_common::reader::{Limit, Reader};
use musli_storage::de::StorageDecoder;
use musli_storage::int::{continuation as c, Variable};

use crate::error::Error;
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
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any<C>(&mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx.adapt())?);

        match tag.kind() {
            Kind::Pack => {
                let len = 2usize.pow(tag.data_raw() as u32);
                self.reader.skip(cx.adapt(), len)?;
            }
            Kind::Prefix => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    L::decode_usize(cx.adapt(), self.reader.borrow_mut())?
                };

                self.reader.skip(cx.adapt(), len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    L::decode_usize(cx.adapt(), self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any(cx)?;
                }
            }
            Kind::Continuation => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, _, u128>(cx.adapt(), self.reader.borrow_mut())?;
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn decode_sequence_len<C>(&mut self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<Input = Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx.adapt())?);

        match tag.kind() {
            Kind::Sequence => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                L::decode_usize(cx.adapt(), self.reader.borrow_mut())?
            }),
            _ => Err(cx.message(Expected {
                expected: Kind::Sequence,
                actual: tag,
            })),
        }
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_pair_sequence<C>(
        mut self,
        cx: &mut C,
    ) -> Result<RemainingWireDecoder<R, I, L>, C::Error>
    where
        C: Context<Input = Error>,
    {
        let len = self.decode_sequence_len(cx)?;
        Ok(RemainingWireDecoder::new(len / 2, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence<C>(
        mut self,
        cx: &mut C,
    ) -> Result<RemainingWireDecoder<R, I, L>, C::Error>
    where
        C: Context<Input = Error>,
    {
        let len = self.decode_sequence_len(cx)?;
        Ok(RemainingWireDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_len<C>(&mut self, cx: &mut C, start: C::Mark) -> Result<usize, C::Error>
    where
        C: Context<Input = Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx.adapt())?);

        match tag.kind() {
            Kind::Prefix => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                L::decode_usize(cx.adapt(), self.reader.borrow_mut())?
            }),
            Kind::Pack => {
                let Some(len) = 2usize.checked_pow(tag.data_raw() as u32) else {
                    return Err(cx.message("Pack tag overflowed"));
                };

                Ok(len)
            }
            _ => Err(cx.marked_message(start, "Expected prefix or pack")),
        }
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
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = Error;
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
    fn decode_unit<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(())
    }

    #[inline(always)]
    fn decode_pack<C>(mut self, cx: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let mark = cx.mark();
        let len = self.decode_len(cx, mark)?;
        Ok(WireDecoder::new(self.reader.limit(len)))
    }

    #[inline(always)]
    fn decode_array<C, const N: usize>(mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let mark = cx.mark();
        let len = self.decode_len(cx, mark)?;

        if len != N {
            return Err(cx.marked_message(
                mark,
                BadLength {
                    actual: len,
                    expected: N,
                },
            ));
        }

        self.reader.read_array(cx.adapt())
    }

    #[inline(always)]
    fn decode_bytes<C, V>(mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, [u8]>,
    {
        let start = cx.mark();
        let len = self.decode_len(cx, start)?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline(always)]
    fn decode_string<C, V>(self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, str>,
    {
        struct Visitor<V>(V);

        impl<'de, C, V> ValueVisitor<'de, C, [u8]> for Visitor<V>
        where
            C: Context,
            V: ValueVisitor<'de, C, str>,
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
    fn decode_bool<C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        const FALSE: Tag = Tag::new(Kind::Continuation, 0);
        const TRUE: Tag = Tag::new(Kind::Continuation, 1);

        let tag = Tag::from_byte(self.reader.read_byte(cx.adapt())?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(cx.message(BadBoolean { actual: tag })),
        }
    }

    #[inline(always)]
    fn decode_char<C>(self, cx: &mut C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let num = self.decode_u32(cx)?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.message(BadCharacter(num))),
        }
    }

    #[inline(always)]
    fn decode_u8<C>(self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_u16<C>(self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_u32<C>(self, cx: &mut C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_u64<C>(self, cx: &mut C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_u128<C>(self, cx: &mut C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_unsigned(cx.adapt(), self.reader)
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
        I::decode_typed_signed(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_i32<C>(self, cx: &mut C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_signed(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_i64<C>(self, cx: &mut C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_signed(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_i128<C>(self, cx: &mut C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        I::decode_typed_signed(cx.adapt(), self.reader)
    }

    #[inline(always)]
    fn decode_usize<C>(self, cx: &mut C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        L::decode_typed_usize(cx.adapt(), self.reader)
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

    #[inline(always)]
    fn decode_option<C>(mut self, cx: &mut C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::new(Kind::Sequence, 0);
        const SOME: Tag = Tag::new(Kind::Sequence, 1);

        let tag = Tag::from_byte(self.reader.read_byte(cx.adapt())?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(cx.message(ExpectedOption { tag })),
        }
    }

    #[inline]
    fn decode_sequence<C>(self, cx: &mut C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.shared_decode_sequence(cx)
    }

    #[inline]
    fn decode_tuple<C>(mut self, cx: &mut C, len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
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
    fn decode_map<C>(self, cx: &mut C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.shared_decode_pair_sequence(cx)
    }

    #[inline]
    fn decode_struct<C>(self, cx: &mut C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.shared_decode_pair_sequence(cx)
    }

    #[inline]
    fn decode_variant<C>(mut self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx.adapt())?);

        if tag != Tag::new(Kind::Sequence, 2) {
            return Err(cx.message(Expected {
                expected: Kind::Sequence,
                actual: tag,
            }));
        }

        Ok(self)
    }
}

impl<'de, R, I, L> PackDecoder<'de> for WireDecoder<Limit<R>, I, L>
where
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = Error;
    type Decoder<'this> = StorageDecoder<<Limit<R> as Reader<'de>>::Mut<'this>, Variable, Variable, Error> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.reader.remaining() > 0 {
            self.reader.skip(cx.adapt(), self.reader.remaining())?;
        }

        Ok(())
    }
}

impl<'de, R, I, L> RemainingWireDecoder<R, I, L>
where
    R: Reader<'de>,
    Error: From<R::Error>,
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
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = Error;
    type Decoder<'this> = WireDecoder<R::Mut<'this>, I, L> where Self: 'this;

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
        Ok(Some(WireDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
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
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = Error;
    type First<'this> = WireDecoder<R::Mut<'this>, I, L> where Self: 'this;
    type Second = Self;

    #[inline]
    fn first<C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn second<C>(self, _: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn skip_second<C>(mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(true)
    }
}

impl<'de, R, I, L> VariantDecoder<'de> for WireDecoder<R, I, L>
where
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = Error;
    type Tag<'this> = WireDecoder<R::Mut<'this>, I, L> where Self: 'this;
    type Variant<'this> = WireDecoder<R::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn skip_variant<C>(&mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end<C>(self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, R, I, L> PairsDecoder<'de> for RemainingWireDecoder<R, I, L>
where
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = Error;

    type Decoder<'this> = WireDecoder<R::Mut<'this>, I, L>
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
        Ok(Some(WireDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
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
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = *self;

        write!(f, "Expected {expected:?} but was {actual:?}")
    }
}

struct BadBoolean {
    actual: Tag,
}

impl fmt::Display for BadBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Bad boolean tag {actual:?}")
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
}

impl fmt::Display for ExpectedOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { tag } = *self;

        write!(f, "Expected zero-to-single sequence, was {tag:?}",)
    }
}

struct BadLength {
    actual: usize,
    expected: usize,
}

impl fmt::Display for BadLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = *self;

        write!(f, "Bad length, got {actual} but expect {expected}")
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
    R: Reader<'de>,
    Error: From<R::Error>,
    I: WireIntegerEncoding,
    L: WireUsizeEncoding,
{
    type Error = Error;
    type Decoder<'this> = WireDecoder<R::Mut<'this>, I, L> where Self: 'this;

    #[inline]
    fn next<C>(&mut self, cx: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.remaining == 0 {
            return Err(cx.message(format_args!("No more tuple elements to decode")));
        }

        self.remaining -= 1;
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end<C>(mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        while self.remaining > 0 {
            WireDecoder::<_, I, L>::new(self.reader.borrow_mut()).skip_any(cx)?;
            self.remaining -= 1;
        }

        Ok(())
    }
}
