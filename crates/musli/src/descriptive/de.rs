use core::fmt;
use core::marker::PhantomData;
use core::mem::take;

use crate::alloc::Vec;
use crate::de::{
    Decoder, EntriesDecoder, EntryDecoder, MapDecoder, SequenceDecoder, SizeHint, Skip,
    UnsizedVisitor, VariantDecoder, Visitor,
};
use crate::int::continuation as c;
use crate::options;
use crate::reader::Limit;
use crate::storage::de::StorageDecoder;
use crate::value::{IntoValueDecoder, Value};
use crate::Context;
use crate::{Options, Reader};

use super::integer_encoding::{decode_typed_signed, decode_typed_unsigned};
use super::tag::{Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, U128, U16, U32, U64, U8};

const BUFFER_OPTIONS: Options = options::new().build();

/// A very simple decoder.
pub struct SelfDecoder<const OPT: Options, R, C, M> {
    cx: C,
    reader: R,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, R, C, M> SelfDecoder<OPT, R, C, M> {
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

impl<'de, const OPT: Options, R, C, M> SelfDecoder<OPT, Limit<R>, C, M>
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

impl<'de, const OPT: Options, R, C, M> SelfDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any(mut self) -> Result<(), C::Error> {
        let mut remaining = 1;

        while remaining > 0 {
            let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

            match tag.kind() {
                Kind::Number => {
                    _ = c::decode::<_, _, u128>(self.cx, self.reader.borrow_mut())?;
                }
                Kind::Mark => match tag.mark() {
                    Mark::Variant => {
                        remaining += 2;
                    }
                    Mark::Some => {
                        remaining += 1;
                    }
                    Mark::Char => {
                        _ = c::decode::<_, _, u32>(self.cx, self.reader.borrow_mut())?;
                    }
                    _ => {}
                },
                Kind::Bytes | Kind::String => {
                    let len = if let Some(len) = tag.data() {
                        len as usize
                    } else {
                        crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?
                    };

                    self.reader.skip(self.cx, len)?;
                }
                Kind::Sequence => {
                    let len = self.decode_len(tag)?;
                    remaining += len;
                }
                Kind::Map => {
                    let len = self.decode_len(tag)?;
                    remaining += len * 2;
                }
                kind => {
                    return Err(self
                        .cx
                        .message(format_args!("Cannot skip over kind {kind:?}")));
                }
            }

            remaining -= 1;
        }

        Ok(())
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_map(mut self) -> Result<RemainingSelfDecoder<OPT, R, C, M>, C::Error> {
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Map, &pos)?;
        Ok(RemainingSelfDecoder::new(self.cx, self.reader, len))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(mut self) -> Result<RemainingSelfDecoder<OPT, R, C, M>, C::Error> {
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Sequence, &pos)?;
        Ok(RemainingSelfDecoder::new(self.cx, self.reader, len))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix(&mut self, kind: Kind, mark: &C::Mark) -> Result<usize, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag.kind() != kind {
            return Err(self.cx.message_at(
                mark,
                Expected {
                    expected: kind,
                    actual: tag,
                },
            ));
        }

        self.decode_len(tag)
    }

    #[inline]
    fn decode_len(&mut self, tag: Tag) -> Result<usize, C::Error> {
        if let Some(len) = tag.data() {
            Ok(len as usize)
        } else {
            crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())
        }
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_pack_length(&mut self, start: &C::Mark) -> Result<usize, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Bytes => self.decode_len(tag),
            _ => Err(self.cx.message_at(start, "Expected prefix or pack")),
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

#[crate::decoder(crate)]
impl<'de, const OPT: Options, R, C, M> Decoder<'de> for SelfDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = SelfDecoder<OPT, R::TryClone, C, M>;
    type DecodeBuffer = IntoValueDecoder<BUFFER_OPTIONS, C, C::Allocator, M>;
    type DecodePack = SelfDecoder<OPT, Limit<R>, C, M>;
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
        write!(f, "type supported by the descriptive decoder")
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(SelfDecoder::new(self.cx, self.reader.try_clone()?))
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
        let len = self.decode_pack_length(&pos)?;
        let mut decoder = SelfDecoder::new(self.cx, self.reader.limit(len));
        let output = f(&mut decoder)?;
        decoder.end()?;
        Ok(output)
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], Self::Error> {
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Bytes, &pos)?;

        if len != N {
            return Err(self.cx.message_at(
                &pos,
                format_args! {
                    "Bad length, got {len} but expect {N}"
                },
            ));
        }

        self.reader.read_array(self.cx)
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, [u8], Error = Self::Error, Allocator = Self::Allocator>,
    {
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Bytes, &pos)?;
        self.reader.read_bytes(self.cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        struct Visitor<V>(V);

        #[crate::de::unsized_visitor(crate)]
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
        let len = self.decode_prefix(Kind::String, &pos)?;
        self.reader.read_bytes(self.cx, len, Visitor(visitor))
    }

    #[inline]
    fn decode_number<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Number => match tag.data() {
                Some(U8) => {
                    let value = self.decode_u8()?;
                    visitor.visit_u8(cx, value)
                }
                Some(U16) => {
                    let value = self.decode_u16()?;
                    visitor.visit_u16(cx, value)
                }
                Some(U32) => {
                    let value = self.decode_u32()?;
                    visitor.visit_u32(cx, value)
                }
                Some(U64) => {
                    let value = self.decode_u64()?;
                    visitor.visit_u64(cx, value)
                }
                Some(U128) => {
                    let value = self.decode_u128()?;
                    visitor.visit_u128(cx, value)
                }
                Some(I8) => {
                    let value = self.decode_i8()?;
                    visitor.visit_i8(cx, value)
                }
                Some(I16) => {
                    let value = self.decode_i16()?;
                    visitor.visit_i16(cx, value)
                }
                Some(I32) => {
                    let value = self.decode_i32()?;
                    visitor.visit_i32(cx, value)
                }
                Some(I64) => {
                    let value = self.decode_i64()?;
                    visitor.visit_i64(cx, value)
                }
                Some(I128) => {
                    let value = self.decode_i128()?;
                    visitor.visit_i128(cx, value)
                }
                Some(F32) => {
                    let value = self.decode_f32()?;
                    visitor.visit_f32(cx, value)
                }
                Some(F64) => {
                    let value = self.decode_f64()?;
                    visitor.visit_f64(cx, value)
                }
                _ => Err(cx.message(format_args!("Unsupported number tag, got {tag:?}"))),
            },
            _ => Err(cx.message(format_args!("Expected number, but got {tag:?}"))),
        }
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, Self::Error> {
        const FALSE: Tag = Tag::from_mark(Mark::False);
        const TRUE: Tag = Tag::from_mark(Mark::True);

        let pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(self.cx.message_at(
                &pos,
                format_args! {
                    "Bad boolean, got {tag:?}"
                },
            )),
        }
    }

    #[inline]
    fn decode_char(mut self) -> Result<char, Self::Error> {
        const CHAR: Tag = Tag::from_mark(Mark::Char);

        let pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag != CHAR {
            return Err(self
                .cx
                .message_at(&pos, format_args!("Expected {CHAR:?}, got {tag:?}")));
        }

        let num = c::decode(self.cx, self.reader.borrow_mut())?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(self.cx.message_at(&pos, format_args!("Bad character"))),
        }
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
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::from_mark(Mark::None);
        const SOME: Tag = Tag::from_mark(Mark::Some);

        let pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(self.cx.message_at(
                &pos,
                format_args! {
                    "Expected option, was {tag:?}"
                },
            )),
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
    fn decode_variant<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);

        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag != VARIANT {
            return Err(self.cx.message(Expected {
                expected: Kind::Mark,
                actual: tag,
            }));
        }

        f(&mut self)
    }

    #[inline]
    fn decode_any<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;

        let Some(tag) = self.reader.peek().map(Tag::from_byte) else {
            return Err(cx.message("Expected tag in input"));
        };

        match tag.kind() {
            Kind::Number => {
                let Some(data) = tag.data() else {
                    return Err(cx.message("Expected number with data"));
                };

                match data {
                    U8 => {
                        let value = self.decode_u8()?;
                        visitor.visit_u8(cx, value)
                    }
                    U16 => {
                        let value = self.decode_u16()?;
                        visitor.visit_u16(cx, value)
                    }
                    U32 => {
                        let value = self.decode_u32()?;
                        visitor.visit_u32(cx, value)
                    }
                    U64 => {
                        let value = self.decode_u64()?;
                        visitor.visit_u64(cx, value)
                    }
                    U128 => {
                        let value = self.decode_u128()?;
                        visitor.visit_u128(cx, value)
                    }
                    I8 => {
                        let value = self.decode_i8()?;
                        visitor.visit_i8(cx, value)
                    }
                    I16 => {
                        let value = self.decode_i16()?;
                        visitor.visit_i16(cx, value)
                    }
                    I32 => {
                        let value = self.decode_i32()?;
                        visitor.visit_i32(cx, value)
                    }
                    I64 => {
                        let value = self.decode_i64()?;
                        visitor.visit_i64(cx, value)
                    }
                    I128 => {
                        let value = self.decode_i128()?;
                        visitor.visit_i128(cx, value)
                    }
                    F32 => {
                        let value = self.decode_f32()?;
                        visitor.visit_f32(cx, value)
                    }
                    F64 => {
                        let value = self.decode_f64()?;
                        visitor.visit_f64(cx, value)
                    }
                    data => Err(cx.message(format_args!("Unsupported number data {data:?}"))),
                }
            }
            Kind::Sequence => {
                let mut sequence = self.shared_decode_sequence()?;
                let output = visitor.visit_sequence(&mut sequence)?;
                sequence.skip_sequence_remaining()?;
                Ok(output)
            }
            Kind::Map => {
                let mut map = self.shared_decode_map()?;
                let output = visitor.visit_map(&mut map)?;
                map.skip_map_remaining()?;
                Ok(output)
            }
            Kind::Bytes => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_bytes(cx, hint)?;
                self.decode_bytes(visitor)
            }
            Kind::String => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_string(cx, hint)?;
                self.decode_string(visitor)
            }
            Kind::Mark => match tag.mark() {
                Mark::True | Mark::False => {
                    let value = self.decode_bool()?;
                    visitor.visit_bool(cx, value)
                }
                Mark::Variant => self.decode_variant(|decoder| visitor.visit_variant(decoder)),
                Mark::Some | Mark::None => match self.decode_option()? {
                    Some(decoder) => visitor.visit_some(decoder),
                    None => visitor.visit_none(cx),
                },
                Mark::Char => {
                    let value = self.decode_char()?;
                    visitor.visit_char(cx, value)
                }
                Mark::Unit => {
                    self.decode_empty()?;
                    visitor.visit_empty(cx)
                }
                mark => Err(cx.message(format_args!("Unsupported mark {mark:?}"))),
            },
            kind => Err(cx.message(format_args!("Unsupported kind {kind:?}"))),
        }
    }
}

impl<'de, const OPT: Options, R, C, M> SequenceDecoder<'de> for SelfDecoder<OPT, Limit<R>, C, M>
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
        = SelfDecoder<OPT, R::Mut<'this>, C, M>
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
        Ok(Some(SelfDecoder::new(self.cx, self.reader.borrow_mut())))
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
        = SelfDecoder<OPT, R::Mut<'this>, C, M>
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
        Ok(Some(SelfDecoder::new(self.cx, self.reader.borrow_mut())))
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
        = SelfDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = SelfDecoder<OPT, R::Mut<'this>, C, M>
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
        Ok(Some(SelfDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        self.skip_map_remaining()?;
        Ok(())
    }
}

impl<'de, const OPT: Options, R, C, M> EntryDecoder<'de> for SelfDecoder<OPT, R, C, M>
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
        = SelfDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, Self::Error> {
        Ok(self)
    }
}

impl<'de, const OPT: Options, R, C, M> VariantDecoder<'de> for SelfDecoder<OPT, R, C, M>
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
        = SelfDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = SelfDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

struct Expected {
    expected: Kind,
    actual: Tag,
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = *self;

        write!(f, "Expected {expected:?} but was {actual:?}",)
    }
}
