use core::fmt;
use core::marker::PhantomData;
use core::mem::take;

use crate::Context;
use crate::alloc::Vec;
use crate::de::{
    Decoder, EntriesDecoder, EntryDecoder, MapDecoder, SequenceDecoder, SizeHint, Skip,
    UnsizedVisitor, VariantDecoder, Visitor,
};
use crate::options;
use crate::reader::Limit;
use crate::storage::de::StorageDecoder;
use crate::value::{IntoValueDecoder, Value};
use crate::{Options, Reader};

use super::integer_encoding::{decode_typed_signed, decode_typed_unsigned};
use super::tag::{Kind, Tag};

const BUFFER_OPTIONS: Options = options::new().build();

/// A very simple decoder.
pub struct UntypedDecoder<const OPT: Options, R, C, M> {
    cx: C,
    reader: R,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, R, C, M> UntypedDecoder<OPT, R, C, M> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: C, reader: R) -> Self {
        Self {
            cx,
            reader,
            _marker: PhantomData,
        }
    }
}

impl<'de, const OPT: Options, R, C, M> UntypedDecoder<OPT, Limit<R>, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    fn end(mut self) -> Result<(), C::Error> {
        if self.reader.remaining() > 0 {
            self.reader.skip(self.cx, self.reader.remaining())?;
        }

        Ok(())
    }
}

impl<'de, const OPT: Options, R, C, M> UntypedDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any(self) -> Result<(), C::Error> {
        Err(self.cx.message("Cannot skip over value"))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_map(self) -> Result<RemainingSelfDecoder<OPT, R, C, M>, C::Error> {
        Err(self.cx.message("Maps are not supported"))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(self) -> Result<RemainingSelfDecoder<OPT, R, C, M>, C::Error> {
        Err(self.cx.message("Sequences are not supported"))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_text_prefix(&mut self, mark: &C::Mark) -> Result<(Kind, usize), C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);
        let kind = tag.kind();

        if !matches!(kind, Kind::Text | Kind::TextJ | Kind::Text5) {
            return Err(self.cx.message_at(
                mark,
                Expected {
                    expected: Kind::Text,
                    actual: tag,
                },
            ));
        }

        Ok((kind, self.decode_len(tag)?))
    }

    #[inline]
    fn decode_len(&mut self, tag: Tag) -> Result<usize, C::Error> {
        fn decode_big_endian(bytes: &[u8]) -> Option<usize> {
            let mut value = 0usize;

            for &byte in bytes {
                value = value.checked_shl(8)?.checked_add(byte as usize)?;
            }

            Some(value)
        }

        let len = match tag.size() {
            size @ 0..=11 => return Ok(size as usize),
            12 => 1,
            13 => 2,
            14 => 4,
            _ => 8,
        };

        let mut buf = [0; 8];
        self.reader.read(self.cx, &mut buf[..len])?;

        let Some(len) = decode_big_endian(&buf[..len]) else {
            return Err(self.cx.message(format_args!("Size overflow for {tag:?}")));
        };

        Ok(len)
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_pack_length(&mut self, start: &C::Mark) -> Result<(Kind, usize), C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);
        let kind = tag.kind();

        match kind {
            Kind::Text | Kind::TextJ | Kind::Text5 => Ok((kind, self.decode_len(tag)?)),
            kind => Err(self.cx.message_at(
                start,
                format_args!("Expected bytes for pack but got {kind:?}"),
            )),
        }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
pub struct RemainingSelfDecoder<const OPT: Options, R, C, M> {
    cx: C,
    reader: R,
    remaining: usize,
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, R, C, M> RemainingSelfDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, reader: R, remaining: usize) -> Self {
        Self {
            cx,
            reader,
            remaining,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn skip_sequence_remaining(mut self) -> Result<(), C::Error> {
        if let Some(item) = self.try_decode_next()? {
            item.skip()?;
        }

        Ok(())
    }

    #[inline]
    fn skip_map_remaining(mut self) -> Result<(), C::Error> {
        loop {
            let Some(key) = self.decode_entry_key()? else {
                break;
            };

            key.skip()?;
            self.decode_entry_value()?.skip()?;
        }

        Ok(())
    }
}

#[crate::trait_defaults(crate)]
impl<'de, const OPT: Options, R, C, M> Decoder<'de> for UntypedDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = UntypedDecoder<OPT, R::TryClone, C, M>;
    type DecodeBuffer = IntoValueDecoder<BUFFER_OPTIONS, C, C::Allocator, M>;
    type DecodePack = UntypedDecoder<OPT, Limit<R>, C, M>;
    type DecodeSome = Self;
    type DecodeSequence = RemainingSelfDecoder<OPT, R, C, M>;
    type DecodeMap = RemainingSelfDecoder<OPT, R, C, M>;
    type DecodeMapEntries = RemainingSelfDecoder<OPT, R, C, M>;
    type DecodeVariant = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the jsonb decoder")
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(UntypedDecoder::new(self.cx, self.reader.try_clone()?))
    }

    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        self.skip_any()
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        self.skip()?;
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, Self::Error> {
        let cx = self.cx;
        let value = self.decode::<Value<Self::Allocator>>()?;
        Ok(value.into_decoder(cx))
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        self.skip()
    }

    #[inline]
    fn decode_pack<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        let pos = self.cx.mark();
        let (_kind, len) = self.decode_pack_length(&pos)?;
        let mut decoder = UntypedDecoder::new(self.cx, self.reader.limit(len));
        let output = f(&mut decoder)?;
        decoder.end()?;
        Ok(output)
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        Err(self.cx.message("Fixed-size arrays are not supported"))
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, [u8], Error = Self::Error, Allocator = Self::Allocator>,
    {
        let pos = self.cx.mark();
        let (_kind, len) = self.decode_text_prefix(&pos)?;
        self.reader.read_bytes(self.cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        struct Visitor<V>(V);

        #[crate::trait_defaults(crate)]
        impl<'de, C, V> UnsizedVisitor<'de, C, [u8]> for Visitor<V>
        where
            C: Context,
            V: UnsizedVisitor<'de, C, str, Error = C::Error, Allocator = C::Allocator>,
        {
            type Ok = V::Ok;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[inline]
            fn visit_owned(
                self,
                cx: C,
                bytes: Vec<u8, Self::Allocator>,
            ) -> Result<Self::Ok, Self::Error> {
                let string = crate::str::from_utf8_owned(bytes).map_err(cx.map())?;
                self.0.visit_owned(cx, string)
            }

            #[inline]
            fn visit_borrowed(self, cx: C, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline]
            fn visit_ref(self, cx: C, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_ref(cx, string)
            }
        }

        let pos = self.cx.mark();
        let (_kind, len) = self.decode_text_prefix(&pos)?;
        self.reader.read_bytes(self.cx, len, Visitor(visitor))
    }

    #[inline]
    fn decode_number<V>(self, _visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        Err(self.cx.message("Numbers are not supported"))
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, Self::Error> {
        let pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::False => Ok(false),
            Kind::True => Ok(true),
            tag => Err(self.cx.message_at(
                &pos,
                format_args! {
                    "Bad boolean, got {tag:?}"
                },
            )),
        }
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        Err(self.cx.message("Characters are not supported"))
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        decode_typed_signed(self.cx, self.reader)
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
    fn decode_usize(mut self) -> Result<usize, Self::Error> {
        decode_typed_unsigned(self.cx, self.reader.borrow_mut())
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        let _pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        let mut decoder = self.shared_decode_sequence()?;
        let output = f(&mut decoder)?;
        decoder.skip_sequence_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        let mut decoder = self.shared_decode_map()?;
        let output = f(&mut decoder)?;
        decoder.skip_map_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        let mut decoder = self.shared_decode_map()?;
        let output = f(&mut decoder)?;
        decoder.skip_map_remaining()?;
        Ok(output)
    }

    #[inline]
    fn decode_variant<F, O>(self, _f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        Err(self.cx.message("Variants are not supported"))
    }

    #[inline]
    fn decode_any<V>(mut self, _visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;

        let Some(tag) = self.reader.peek().map(Tag::from_byte) else {
            return Err(cx.message("Expected tag in input"));
        };

        match tag.kind() {
            kind => Err(cx.message(format_args!("Unsupported kind {kind:?}"))),
        }
    }
}

impl<'de, const OPT: Options, R, C, M> SequenceDecoder<'de> for UntypedDecoder<OPT, Limit<R>, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeNext<'this>
        = StorageDecoder<OPT, true, <Limit<R> as Reader<'de>>::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, Self::Error> {
        Ok(Some(StorageDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

impl<'de, const OPT: Options, R, C, M> SequenceDecoder<'de> for RemainingSelfDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeNext<'this>
        = UntypedDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::exact(self.remaining)
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(UntypedDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        let cx = self.cx;

        let Some(decoder) = self.try_decode_next()? else {
            return Err(cx.message("No remaining elements"));
        };

        Ok(decoder)
    }
}

impl<'de, const OPT: Options, R, C, M> MapDecoder<'de> for RemainingSelfDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntry<'this>
        = UntypedDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = RemainingSelfDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::exact(self.remaining)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(UntypedDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, Self::Error> {
        Ok(RemainingSelfDecoder::new(
            self.cx,
            self.reader.borrow_mut(),
            take(&mut self.remaining),
        ))
    }
}

impl<'de, const OPT: Options, R, C, M> EntriesDecoder<'de> for RemainingSelfDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntryKey<'this>
        = UntypedDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = UntypedDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(UntypedDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        Ok(UntypedDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        self.skip_map_remaining()?;
        Ok(())
    }
}

impl<'de, const OPT: Options, R, C, M> EntryDecoder<'de> for UntypedDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeKey<'this>
        = UntypedDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, Self::Error> {
        Ok(UntypedDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, Self::Error> {
        Ok(self)
    }
}

impl<'de, const OPT: Options, R, C, M> VariantDecoder<'de> for UntypedDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeTag<'this>
        = UntypedDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = UntypedDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error> {
        Ok(UntypedDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error> {
        Ok(UntypedDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

struct Expected {
    expected: Kind,
    actual: Tag,
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = *self;
        write!(f, "Expected {expected:?}, but was {actual:?}",)
    }
}

struct ExpectedVariant {
    expected: Kind,
    actual: Tag,
}

impl fmt::Display for ExpectedVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = *self;
        write!(f, "Expected {expected:?} for variant, but was {actual:?}",)
    }
}
