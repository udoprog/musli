use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, MapDecoder, MapEntryDecoder, MapPairsDecoder, PackDecoder, SequenceDecoder, SizeHint,
    StructDecoder, StructFieldDecoder, StructPairsDecoder, ValueVisitor, VariantDecoder,
};
use musli::Context;
use musli_common::reader::{Limit, Reader};
use musli_storage::de::StorageDecoder;
use musli_storage::int::continuation as c;

use crate::options::Options;
use crate::tag::Kind;
use crate::tag::Tag;

/// A very simple decoder.
pub struct WireDecoder<R, const F: Options> {
    reader: R,
}

impl<R, const F: Options> WireDecoder<R, F> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<'de, R, const F: Options> WireDecoder<R, F>
where
    R: Reader<'de>,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any<C>(&mut self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Pack => {
                let len = 2usize.pow(tag.data_raw() as u32);
                self.reader.skip(cx, len)?;
            }
            Kind::Prefix => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    crate::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?
                };

                self.reader.skip(cx, len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    crate::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?
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
    fn decode_sequence_len<C>(&mut self, cx: &C) -> Result<usize, C::Error>
    where
        C: ?Sized + Context,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Sequence => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                crate::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?
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
        cx: &C,
    ) -> Result<RemainingWireDecoder<R, F>, C::Error>
    where
        C: ?Sized + Context,
    {
        let len = self.decode_sequence_len(cx)?;
        Ok(RemainingWireDecoder::new(len / 2, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence<C>(mut self, cx: &C) -> Result<RemainingWireDecoder<R, F>, C::Error>
    where
        C: ?Sized + Context,
    {
        let len = self.decode_sequence_len(cx)?;
        Ok(RemainingWireDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_len<C>(&mut self, cx: &C, start: C::Mark) -> Result<usize, C::Error>
    where
        C: ?Sized + Context,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Prefix => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                crate::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?
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
pub struct RemainingWireDecoder<R, const F: Options> {
    remaining: usize,
    decoder: WireDecoder<R, F>,
}

#[musli::decoder]
impl<'de, C, R, const F: Options> Decoder<'de, C> for WireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Decoder<U> = Self where U: Context;
    type Pack = WireDecoder<Limit<R>, F>;
    type Some = Self;
    type Sequence = RemainingWireDecoder<R, F>;
    type Tuple = TupleWireDecoder<R, F>;
    type Map = RemainingWireDecoder<R, F>;
    type Struct = RemainingWireDecoder<R, F>;
    type Variant = Self;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::Decoder<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire decoder")
    }

    #[inline(always)]
    fn decode_unit(mut self, cx: &C) -> Result<(), C::Error> {
        self.skip_any(cx)?;
        Ok(())
    }

    #[inline(always)]
    fn decode_pack(mut self, cx: &C) -> Result<Self::Pack, C::Error> {
        let mark = cx.mark();
        let len = self.decode_len(cx, mark)?;
        Ok(WireDecoder::new(self.reader.limit(len)))
    }

    #[inline(always)]
    fn decode_array<const N: usize>(mut self, cx: &C) -> Result<[u8; N], C::Error> {
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

        self.reader.read_array(cx)
    }

    #[inline(always)]
    fn decode_bytes<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let start = cx.mark();
        let len = self.decode_len(cx, start)?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline(always)]
    fn decode_string<V>(self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
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

            #[inline(always)]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline(always)]
            fn visit_owned(self, cx: &C, bytes: Vec<u8>) -> Result<Self::Ok, C::Error> {
                let string =
                    musli_common::str::from_utf8_owned(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_owned(cx, string)
            }

            #[inline(always)]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline(always)]
            fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(cx, Visitor(visitor))
    }

    #[inline(always)]
    fn decode_bool(mut self, cx: &C) -> Result<bool, C::Error> {
        const FALSE: Tag = Tag::new(Kind::Continuation, 0);
        const TRUE: Tag = Tag::new(Kind::Continuation, 1);

        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(cx.message(BadBoolean { actual: tag })),
        }
    }

    #[inline(always)]
    fn decode_char(self, cx: &C) -> Result<char, C::Error> {
        let num = self.decode_u32(cx)?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.message(BadCharacter(num))),
        }
    }

    #[inline(always)]
    fn decode_u8(self, cx: &C) -> Result<u8, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u16(self, cx: &C) -> Result<u16, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u64(self, cx: &C) -> Result<u64, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_u128(self, cx: &C) -> Result<u128, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i8(self, cx: &C) -> Result<i8, C::Error> {
        Ok(self.decode_u8(cx)? as i8)
    }

    #[inline(always)]
    fn decode_i16(self, cx: &C) -> Result<i16, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i32(self, cx: &C) -> Result<i32, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i64(self, cx: &C) -> Result<i64, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_i128(self, cx: &C) -> Result<i128, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_usize(self, cx: &C) -> Result<usize, C::Error> {
        crate::wire_int::decode_length::<_, _, F>(cx, self.reader)
    }

    #[inline(always)]
    fn decode_isize(self, cx: &C) -> Result<isize, C::Error> {
        Ok(self.decode_usize(cx)? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f32(self, cx: &C) -> Result<f32, C::Error> {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline(always)]
    fn decode_f64(self, cx: &C) -> Result<f64, C::Error> {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline(always)]
    fn decode_option(mut self, cx: &C) -> Result<Option<Self::Some>, C::Error> {
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::new(Kind::Sequence, 0);
        const SOME: Tag = Tag::new(Kind::Sequence, 1);

        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(cx.message(ExpectedOption { tag })),
        }
    }

    #[inline]
    fn decode_sequence(self, cx: &C) -> Result<Self::Sequence, C::Error> {
        self.shared_decode_sequence(cx)
    }

    #[inline]
    fn decode_tuple(mut self, cx: &C, len: usize) -> Result<Self::Tuple, C::Error> {
        let actual = self.decode_sequence_len(cx)?;

        if len != actual {
            return Err(cx.message(format_args!(
                "tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(TupleWireDecoder::new(self.reader, len))
    }

    #[inline]
    fn decode_map(self, cx: &C) -> Result<Self::Map, C::Error> {
        self.shared_decode_pair_sequence(cx)
    }

    #[inline]
    fn decode_struct(self, cx: &C, _: Option<usize>) -> Result<Self::Struct, C::Error> {
        self.shared_decode_pair_sequence(cx)
    }

    #[inline]
    fn decode_variant(mut self, cx: &C) -> Result<Self::Variant, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag != Tag::new(Kind::Sequence, 2) {
            return Err(cx.message(Expected {
                expected: Kind::Sequence,
                actual: tag,
            }));
        }

        Ok(self)
    }
}

impl<'de, C, R, const F: Options> PackDecoder<'de, C> for WireDecoder<Limit<R>, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Decoder<'this> = StorageDecoder<<Limit<R> as Reader<'de>>::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn next(&mut self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        Ok(StorageDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        if self.reader.remaining() > 0 {
            self.reader.skip(cx, self.reader.remaining())?;
        }

        Ok(())
    }
}

impl<R, const F: Options> RemainingWireDecoder<R, F> {
    #[inline]
    fn new(remaining: usize, decoder: WireDecoder<R, F>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'de, C, R, const F: Options> SequenceDecoder<'de, C> for RemainingWireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Decoder<'this> = WireDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn next(&mut self, _: &C) -> Result<Option<Self::Decoder<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        // Skip remaining elements.
        while let Some(mut item) = SequenceDecoder::next(&mut self, cx)? {
            item.skip_any(cx)?;
        }

        Ok(())
    }
}

impl<'de, C, R, const F: Options> VariantDecoder<'de, C> for WireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Tag<'this> = WireDecoder<R::Mut<'this>, F> where Self: 'this;
    type Variant<'this> = WireDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn tag(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error> {
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error> {
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn skip_variant(&mut self, cx: &C) -> Result<bool, C::Error> {
        self.skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

#[musli::map_decoder]
impl<'de, C, R, const F: Options> MapDecoder<'de, C> for RemainingWireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Entry<'this> = WireDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type MapPairs = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn into_map_pairs(self, _: &C) -> Result<Self::MapPairs, C::Error> {
        Ok(self)
    }

    #[inline]
    fn entry(&mut self, _: &C) -> Result<Option<Self::Entry<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        // Skip remaining elements.
        while let Some(mut item) = MapDecoder::entry(&mut self, cx)? {
            item.skip_any(cx)?;
        }

        Ok(())
    }
}

impl<'de, C, R, const F: Options> MapEntryDecoder<'de, C> for WireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type MapKey<'this> = WireDecoder<R::Mut<'this>, F> where Self: 'this;
    type MapValue = Self;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn map_value(self, _: &C) -> Result<Self::MapValue, C::Error> {
        Ok(self)
    }

    #[inline]
    fn skip_map_value(mut self, cx: &C) -> Result<bool, C::Error> {
        self.skip_any(cx)?;
        Ok(true)
    }
}

#[musli::struct_decoder]
impl<'de, C, R, const F: Options> StructDecoder<'de, C> for RemainingWireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Field<'this> = WireDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type StructPairs = Self;

    #[inline]
    fn size_hint(&self, cx: &C) -> SizeHint {
        MapDecoder::size_hint(self, cx)
    }

    #[inline]
    fn into_struct_pairs(self, _: &C) -> Result<Self::StructPairs, C::Error> {
        Ok(self)
    }

    #[inline]
    fn field(&mut self, cx: &C) -> Result<Option<Self::Field<'_>>, C::Error> {
        MapDecoder::entry(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapDecoder::end(self, cx)
    }
}

impl<'de, C, R, const F: Options> StructFieldDecoder<'de, C> for WireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type FieldName<'this> = WireDecoder<R::Mut<'this>, F> where Self: 'this;
    type FieldValue = Self;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        MapEntryDecoder::map_key(self, cx)
    }

    #[inline]
    fn field_value(self, cx: &C) -> Result<Self::FieldValue, C::Error> {
        MapEntryDecoder::map_value(self, cx)
    }

    #[inline]
    fn skip_field_value(self, cx: &C) -> Result<bool, C::Error> {
        MapEntryDecoder::skip_map_value(self, cx)
    }
}

impl<'de, C, R, const F: Options> MapPairsDecoder<'de, C> for RemainingWireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type MapPairsKey<'this> = WireDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type MapPairsValue<'this> = WireDecoder<R::Mut<'this>, F>
    where
        Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        Ok(WireDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_map_pairs_value(&mut self, cx: &C) -> Result<bool, C::Error> {
        self.map_pairs_value(cx)?.skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        while self.remaining > 0 {
            self.remaining -= 1;
            WireDecoder::<_, F>::new(self.decoder.reader.borrow_mut()).skip_any(cx)?;
            self.map_pairs_value(cx)?.skip_any(cx)?;
        }

        Ok(())
    }
}

impl<'de, C, R, const F: Options> StructPairsDecoder<'de, C> for RemainingWireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type FieldName<'this> = WireDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type FieldValue<'this> = WireDecoder<R::Mut<'this>, F>
    where
        Self: 'this;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(cx.message("Ran out of struct fields to decode"));
        }

        self.remaining -= 1;
        Ok(WireDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn field_value(&mut self, _: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        Ok(WireDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn skip_field_value(&mut self, cx: &C) -> Result<bool, C::Error> {
        self.field_value(cx)?.skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        while self.remaining > 0 {
            self.remaining -= 1;
            self.field_name(cx)?.skip_any(cx)?;
            self.field_value(cx)?.skip_any(cx)?;
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
pub struct TupleWireDecoder<R, const F: Options> {
    reader: R,
    remaining: usize,
}

impl<R, const F: Options> TupleWireDecoder<R, F> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: R, remaining: usize) -> Self {
        Self { reader, remaining }
    }
}

impl<'de, C, R, const F: Options> PackDecoder<'de, C> for TupleWireDecoder<R, F>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Decoder<'this> = WireDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn next(&mut self, cx: &C) -> Result<Self::Decoder<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(cx.message(format_args!("No more tuple elements to decode")));
        }

        self.remaining -= 1;
        Ok(WireDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        while self.remaining > 0 {
            WireDecoder::<_, F>::new(self.reader.borrow_mut()).skip_any(cx)?;
            self.remaining -= 1;
        }

        Ok(())
    }
}
