use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decode, Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, PackDecoder, SequenceDecoder,
    SizeHint, StructDecoder, StructFieldDecoder, StructFieldsDecoder, ValueVisitor, VariantDecoder,
};
use musli::Context;
use musli_common::int::continuation as c;
use musli_storage::de::StorageDecoder;

use crate::options::Options;
use crate::reader::{Limit, Reader};
use crate::tag::Kind;
use crate::tag::Tag;

/// A very simple decoder.
pub struct WireDecoder<'a, R, const F: Options, C: ?Sized> {
    cx: &'a C,
    reader: R,
}

impl<'a, R, const F: Options, C: ?Sized> WireDecoder<'a, R, F, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, reader: R) -> Self {
        Self { cx, reader }
    }
}

impl<'a, 'de, R, const F: Options, C> WireDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any(&mut self) -> Result<(), C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Pack => {
                let len = 2usize.pow(tag.data_raw() as u32);
                self.reader.skip(self.cx, len)?;
            }
            Kind::Prefix => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    musli_common::int::decode_usize::<_, _, F>(self.cx, self.reader.borrow_mut())?
                };

                self.reader.skip(self.cx, len)?;
            }
            Kind::Sequence => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    musli_common::int::decode_usize::<_, _, F>(self.cx, self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any()?;
                }
            }
            Kind::Continuation => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, _, u128>(self.cx, self.reader.borrow_mut())?;
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn decode_sequence_len(&mut self) -> Result<usize, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Sequence => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                musli_common::int::decode_usize::<_, _, F>(self.cx, self.reader.borrow_mut())?
            }),
            _ => Err(self.cx.message(Expected {
                expected: Kind::Sequence,
                actual: tag,
            })),
        }
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_pair_sequence(
        mut self,
    ) -> Result<RemainingWireDecoder<'a, R, F, C>, C::Error> {
        let len = self.decode_sequence_len()?;
        Ok(RemainingWireDecoder::new(len / 2, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(mut self) -> Result<RemainingWireDecoder<'a, R, F, C>, C::Error> {
        let len = self.decode_sequence_len()?;
        Ok(RemainingWireDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_len(&mut self, start: C::Mark) -> Result<usize, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Prefix => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                musli_common::int::decode_usize::<_, _, F>(self.cx, self.reader.borrow_mut())?
            }),
            Kind::Pack => {
                let Some(len) = 2usize.checked_pow(tag.data_raw() as u32) else {
                    return Err(self.cx.message("Pack tag overflowed"));
                };

                Ok(len)
            }
            _ => Err(self.cx.marked_message(start, "Expected prefix or pack")),
        }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct RemainingWireDecoder<'a, R, const F: Options, C: ?Sized> {
    remaining: usize,
    decoder: WireDecoder<'a, R, F, C>,
}

#[musli::decoder]
impl<'a, 'de, R, const F: Options, C> Decoder<'de> for WireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<'this, U> = WireDecoder<'this, R, F, U> where U: 'this + Context;
    type DecodePack = WireDecoder<'a, Limit<R>, F, C>;
    type DecodeSome = Self;
    type DecodeSequence = RemainingWireDecoder<'a, R, F, C>;
    type DecodeTuple = TupleWireDecoder<'a, R, F, C>;
    type DecodeMap = RemainingWireDecoder<'a, R, F, C>;
    type DecodeStruct = RemainingWireDecoder<'a, R, F, C>;
    type DecodeVariant = Self;

    #[inline]
    fn cx(&self) -> &C {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(WireDecoder::new(cx, self.reader))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire decoder")
    }

    #[inline]
    fn decode<T>(self) -> Result<T, C::Error>
    where
        T: Decode<'de, Self::Mode>,
    {
        T::decode(self.cx, self)
    }

    #[inline]
    fn skip(mut self) -> Result<(), C::Error> {
        self.skip_any()
    }

    #[inline]
    fn decode_unit(mut self) -> Result<(), C::Error> {
        self.skip_any()?;
        Ok(())
    }

    #[inline]
    fn decode_pack(mut self) -> Result<Self::DecodePack, C::Error> {
        let mark = self.cx.mark();
        let len = self.decode_len(mark)?;
        Ok(WireDecoder::new(self.cx, self.reader.limit(len)))
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], C::Error> {
        let mark = self.cx.mark();
        let len = self.decode_len(mark)?;

        if len != N {
            return Err(self.cx.marked_message(
                mark,
                BadLength {
                    actual: len,
                    expected: N,
                },
            ));
        }

        self.reader.read_array(self.cx)
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let mark = self.cx.mark();
        let len = self.decode_len(mark)?;
        self.reader.read_bytes(self.cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, C::Error>
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

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline]
            fn visit_owned(self, cx: &C, bytes: Vec<u8>) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8_owned(bytes).map_err(cx.map())?;
                self.0.visit_owned(cx, string)
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline]
            fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(Visitor(visitor))
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, C::Error> {
        const FALSE: Tag = Tag::new(Kind::Continuation, 0);
        const TRUE: Tag = Tag::new(Kind::Continuation, 1);

        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(self.cx.message(BadBoolean { actual: tag })),
        }
    }

    #[inline]
    fn decode_char(self) -> Result<char, C::Error> {
        let cx = self.cx;
        let num = self.decode_u32()?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.message(BadCharacter(num))),
        }
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, C::Error> {
        crate::wire_int::decode_unsigned::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, C::Error> {
        Ok(self.decode_u8()? as i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, C::Error> {
        crate::wire_int::decode_signed::<_, _, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, C::Error> {
        crate::wire_int::decode_length::<_, _, F>(self.cx, self.reader)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, C::Error> {
        Ok(self.decode_usize()? as isize)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self) -> Result<f32, C::Error> {
        let bits = self.decode_u32()?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self) -> Result<f64, C::Error> {
        let bits = self.decode_u64()?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, C::Error> {
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::new(Kind::Sequence, 0);
        const SOME: Tag = Tag::new(Kind::Sequence, 1);

        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(self.cx.message(ExpectedOption { tag })),
        }
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::DecodeSequence, C::Error> {
        self.shared_decode_sequence()
    }

    #[inline]
    fn decode_tuple(mut self, len: usize) -> Result<Self::DecodeTuple, C::Error> {
        let actual = self.decode_sequence_len()?;

        if len != actual {
            return Err(self.cx.message(format_args!(
                "tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(TupleWireDecoder::new(self.cx, self.reader, len))
    }

    #[inline]
    fn decode_map(self) -> Result<Self::DecodeMap, C::Error> {
        self.shared_decode_pair_sequence()
    }

    #[inline]
    fn decode_struct(self, _: Option<usize>) -> Result<Self::DecodeStruct, C::Error> {
        self.shared_decode_pair_sequence()
    }

    #[inline]
    fn decode_variant(mut self) -> Result<Self::DecodeVariant, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag != Tag::new(Kind::Sequence, 2) {
            return Err(self.cx.message(Expected {
                expected: Kind::Sequence,
                actual: tag,
            }));
        }

        Ok(self)
    }
}

impl<'a, 'de, R, const F: Options, C> PackDecoder<'de> for WireDecoder<'a, Limit<R>, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = StorageDecoder<'a, <Limit<R> as Reader<'de>>::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end(mut self) -> Result<(), C::Error> {
        if self.reader.remaining() > 0 {
            self.reader.skip(self.cx, self.reader.remaining())?;
        }

        Ok(())
    }
}

impl<'a, R, const F: Options, C: ?Sized> RemainingWireDecoder<'a, R, F, C> {
    #[inline]
    fn new(remaining: usize, decoder: WireDecoder<'a, R, F, C>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'a, 'de, R, const F: Options, C> SequenceDecoder<'de> for RemainingWireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = WireDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        )))
    }
}

impl<'a, 'de, R, const F: Options, C> VariantDecoder<'de> for WireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeTag<'this> = WireDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;
    type DecodeVariant<'this> = WireDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeVariant<'_>, C::Error> {
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn skip_value(&mut self) -> Result<bool, C::Error> {
        self.skip_any()?;
        Ok(true)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

#[musli::map_decoder]
impl<'a, 'de, R, const F: Options, C> MapDecoder<'de> for RemainingWireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeEntry<'this> = WireDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;
    type IntoMapEntries = Self;

    #[inline]
    fn cx(&self) -> &C {
        self.decoder.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
    }

    #[inline]
    fn into_map_entries(self) -> Result<Self::IntoMapEntries, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        )))
    }
}

impl<'a, 'de, R, const F: Options, C> MapEntryDecoder<'de> for WireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeMapKey<'this> = WireDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;
    type DecodeMapValue = Self;

    #[inline]
    fn decode_map_key(&mut self) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_map_value(self) -> Result<Self::DecodeMapValue, C::Error> {
        Ok(self)
    }

    #[inline]
    fn skip_map_value(mut self) -> Result<bool, C::Error> {
        self.skip_any()?;
        Ok(true)
    }
}

#[musli::struct_decoder]
impl<'a, 'de, R, const F: Options, C> StructDecoder<'de> for RemainingWireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeField<'this> = WireDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;
    type IntoStructFields = Self;

    #[inline]
    fn cx(&self) -> &C {
        self.decoder.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        MapDecoder::size_hint(self)
    }

    #[inline]
    fn into_struct_fields(self) -> Result<Self::IntoStructFields, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_field(&mut self) -> Result<Option<Self::DecodeField<'_>>, C::Error> {
        MapDecoder::decode_entry(self)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        MapDecoder::end(self)
    }
}

impl<'a, 'de, R, const F: Options, C> StructFieldDecoder<'de> for WireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeFieldName<'this> = WireDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;
    type DecodeFieldValue = Self;

    #[inline]
    fn decode_field_name(&mut self) -> Result<Self::DecodeFieldName<'_>, C::Error> {
        MapEntryDecoder::decode_map_key(self)
    }

    #[inline]
    fn decode_field_value(self) -> Result<Self::DecodeFieldValue, C::Error> {
        MapEntryDecoder::decode_map_value(self)
    }

    #[inline]
    fn skip_field_value(self) -> Result<bool, C::Error> {
        MapEntryDecoder::skip_map_value(self)
    }
}

impl<'a, 'de, R, const F: Options, C> MapEntriesDecoder<'de> for RemainingWireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeMapEntryKey<'this> = WireDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;
    type DecodeMapEntryValue<'this> = WireDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;

    #[inline]
    fn decode_map_entry_key(&mut self) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(WireDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        )))
    }

    #[inline]
    fn decode_map_entry_value(&mut self) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        Ok(WireDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        ))
    }

    #[inline]
    fn skip_map_entry_value(&mut self) -> Result<bool, C::Error> {
        self.decode_map_entry_value()?.skip_any()?;
        Ok(true)
    }
}

impl<'a, 'de, R, const F: Options, C> StructFieldsDecoder<'de> for RemainingWireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeStructFieldName<'this> = WireDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;
    type DecodeStructFieldValue<'this> = WireDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;

    #[inline]
    fn decode_struct_field_name(&mut self) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(self
                .decoder
                .cx
                .message("Ran out of struct fields to decode"));
        }

        self.remaining -= 1;
        Ok(WireDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        ))
    }

    #[inline]
    fn decode_struct_field_value(&mut self) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        Ok(WireDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        ))
    }

    #[inline]
    fn skip_struct_field_value(&mut self) -> Result<bool, C::Error> {
        self.decode_struct_field_value()?.skip_any()?;
        Ok(true)
    }

    #[inline]
    fn end(mut self) -> Result<(), C::Error> {
        while self.remaining > 0 {
            self.remaining -= 1;
            self.decode_struct_field_name()?.skip_any()?;
            self.decode_struct_field_value()?.skip_any()?;
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
pub struct TupleWireDecoder<'a, R, const F: Options, C: ?Sized> {
    cx: &'a C,
    reader: R,
    remaining: usize,
}

impl<'a, R, const F: Options, C: ?Sized> TupleWireDecoder<'a, R, F, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, reader: R, remaining: usize) -> Self {
        Self {
            cx,
            reader,
            remaining,
        }
    }
}

impl<'a, 'de, R, const F: Options, C> PackDecoder<'de> for TupleWireDecoder<'a, R, F, C>
where
    C: ?Sized + Context,
    R: Reader<'de>,
{
    type Cx = C;
    type DecodeNext<'this> = WireDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(self
                .cx
                .message(format_args!("No more tuple elements to decode")));
        }

        self.remaining -= 1;
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end(mut self) -> Result<(), C::Error> {
        while self.remaining > 0 {
            WireDecoder::<_, F, _>::new(self.cx, self.reader.borrow_mut()).skip_any()?;
            self.remaining -= 1;
        }

        Ok(())
    }
}
