use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decoder, MapDecoder, MapEntryDecoder, MapPairsDecoder, NumberHint, NumberVisitor, PackDecoder,
    SequenceDecoder, SizeHint, StructDecoder, StructFieldDecoder, StructPairsDecoder, TypeHint,
    ValueVisitor, VariantDecoder, Visitor,
};
use musli::Context;
use musli_storage::de::StorageDecoder;

use crate::int::continuation as c;
use crate::integer_encoding::{decode_typed_signed, decode_typed_unsigned};
use crate::options::Options;
use crate::reader::{Limit, Reader};
use crate::tag::{Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, U128, U16, U32, U64, U8};

/// A very simple decoder.
pub struct SelfDecoder<R, const F: Options> {
    reader: R,
}

impl<R, const F: Options> SelfDecoder<R, F> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }
}

pub struct SelfTupleDecoder<R, const F: Options> {
    reader: R,
}

impl<R, const F: Options> SelfTupleDecoder<R, F> {
    #[inline]
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<'de, R, const F: Options> SelfDecoder<R, F>
where
    R: Reader<'de>,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any<C>(&mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Number => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, _, u128>(cx, self.reader.borrow_mut())?;
                }
            }
            Kind::Mark => {
                if let Mark::Variant = tag.mark() {
                    self.skip_any(cx)?;
                    self.skip_any(cx)?;
                }
            }
            Kind::Bytes => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    crate::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?
                };

                self.reader.skip(cx, len)?;
            }
            Kind::Pack => {
                let len = 2usize.pow(tag.data_raw() as u32);
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
            Kind::Map => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    crate::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any(cx)?;
                    self.skip_any(cx)?;
                }
            }
            kind => {
                return Err(cx.message(format_args!("Unsupported kind {kind:?}")));
            }
        }

        Ok(())
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_map<C>(mut self, cx: &C) -> Result<RemainingSelfDecoder<R, F>, C::Error>
    where
        C: Context,
    {
        let pos = cx.mark();
        let len = self.decode_prefix(cx, Kind::Map, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence<C>(mut self, cx: &C) -> Result<RemainingSelfDecoder<R, F>, C::Error>
    where
        C: Context,
    {
        let pos = cx.mark();
        let len = self.decode_prefix(cx, Kind::Sequence, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix<C>(&mut self, cx: &C, kind: Kind, mark: C::Mark) -> Result<usize, C::Error>
    where
        C: Context,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag.kind() != kind {
            return Err(cx.marked_message(
                mark,
                Expected {
                    expected: kind,
                    actual: tag,
                },
            ));
        }

        Ok(if let Some(len) = tag.data() {
            len as usize
        } else {
            crate::int::decode_usize::<_, _, F>(cx, self.reader.borrow_mut())?
        })
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_pack_length<C>(&mut self, cx: &C, start: C::Mark) -> Result<usize, C::Error>
    where
        C: Context,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Bytes => Ok(if let Some(len) = tag.data() {
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
pub struct RemainingSelfDecoder<R, const F: Options> {
    remaining: usize,
    decoder: SelfDecoder<R, F>,
}

#[musli::decoder]
impl<'de, C, R, const F: Options> Decoder<'de, C> for SelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Decoder<U> = Self where U: Context;
    type Pack = SelfDecoder<Limit<R>, F>;
    type Some = Self;
    type Sequence = RemainingSelfDecoder<R, F>;
    type Tuple = SelfTupleDecoder<R, F>;
    type Map = RemainingSelfDecoder<R, F>;
    type Struct = RemainingSelfDecoder<R, F>;
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
        write!(f, "type supported by the descriptive decoder")
    }

    #[inline]
    fn type_hint(&mut self, cx: &C) -> Result<TypeHint, C::Error> {
        let tag = match self.reader.peek(cx)? {
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
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Sequence(hint))
            }
            Kind::Map => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Map(hint))
            }
            Kind::Bytes => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                Ok(TypeHint::Bytes(hint))
            }
            Kind::String => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
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
    fn decode_unit(mut self, cx: &C) -> Result<(), C::Error> {
        self.skip_any(cx)?;
        Ok(())
    }

    #[inline]
    fn decode_pack(mut self, cx: &C) -> Result<Self::Pack, C::Error> {
        let pos = cx.mark();
        let len = self.decode_pack_length(cx, pos)?;
        Ok(SelfDecoder::new(self.reader.limit(len)))
    }

    #[inline]
    fn decode_array<const N: usize>(mut self, cx: &C) -> Result<[u8; N], C::Error> {
        let pos = cx.mark();
        let len = self.decode_prefix(cx, Kind::Bytes, pos)?;

        if len != N {
            return Err(cx.marked_message(
                pos,
                format_args! {
                    "bad length, got {len} but expect {N}"
                },
            ));
        }

        self.reader.read_array(cx)
    }

    #[inline]
    fn decode_bytes<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, [u8]>,
    {
        let pos = cx.mark();
        let len = self.decode_prefix(cx, Kind::Bytes, pos)?;
        self.reader.read_bytes(cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: ValueVisitor<'de, C, str>,
    {
        struct Visitor<V>(V);

        impl<'de, C, V> ValueVisitor<'de, C, [u8]> for Visitor<V>
        where
            C: Context,
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
                let string =
                    musli_common::str::from_utf8_owned(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_owned(cx, string)
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline]
            fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string = musli_common::str::from_utf8(bytes).map_err(|err| cx.custom(err))?;
                self.0.visit_ref(cx, string)
            }
        }

        let pos = cx.mark();
        let len = self.decode_prefix(cx, Kind::String, pos)?;
        self.reader.read_bytes(cx, len, Visitor(visitor))
    }

    #[inline]
    fn decode_bool(mut self, cx: &C) -> Result<bool, C::Error> {
        const FALSE: Tag = Tag::from_mark(Mark::False);
        const TRUE: Tag = Tag::from_mark(Mark::True);

        let pos = cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(cx.marked_message(
                pos,
                format_args! {
                    "bad boolean, got {tag:?}"
                },
            )),
        }
    }

    #[inline]
    fn decode_char(mut self, cx: &C) -> Result<char, C::Error> {
        const CHAR: Tag = Tag::from_mark(Mark::Char);

        let pos = cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag != CHAR {
            return Err(cx.marked_message(pos, format_args!("Expected {CHAR:?}, got {tag:?}")));
        }

        let num = c::decode(cx, self.reader.borrow_mut())?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.marked_message(pos, format_args!("Bad character"))),
        }
    }

    #[inline]
    fn decode_number<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: NumberVisitor<'de, C>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag.kind() {
            Kind::Number => match tag.data() {
                Some(U8) => {
                    let value = self.decode_u8(cx)?;
                    visitor.visit_u8(cx, value)
                }
                Some(U16) => {
                    let value = self.decode_u16(cx)?;
                    visitor.visit_u16(cx, value)
                }
                Some(U32) => {
                    let value = self.decode_u32(cx)?;
                    visitor.visit_u32(cx, value)
                }
                Some(U64) => {
                    let value = self.decode_u64(cx)?;
                    visitor.visit_u64(cx, value)
                }
                Some(U128) => {
                    let value = self.decode_u128(cx)?;
                    visitor.visit_u128(cx, value)
                }
                Some(I8) => {
                    let value = self.decode_i8(cx)?;
                    visitor.visit_i8(cx, value)
                }
                Some(I16) => {
                    let value = self.decode_i16(cx)?;
                    visitor.visit_i16(cx, value)
                }
                Some(I32) => {
                    let value = self.decode_i32(cx)?;
                    visitor.visit_i32(cx, value)
                }
                Some(I64) => {
                    let value = self.decode_i64(cx)?;
                    visitor.visit_i64(cx, value)
                }
                Some(I128) => {
                    let value = self.decode_i128(cx)?;
                    visitor.visit_i128(cx, value)
                }
                Some(F32) => {
                    let value = self.decode_f32(cx)?;
                    visitor.visit_f32(cx, value)
                }
                Some(F64) => {
                    let value = self.decode_f64(cx)?;
                    visitor.visit_f64(cx, value)
                }
                _ => Err(cx.message(format_args!("Unsupported number tag, got {tag:?}"))),
            },
            _ => Err(cx.message(format_args!("Expected number, but got {tag:?}"))),
        }
    }

    #[inline]
    fn decode_u8(self, cx: &C) -> Result<u8, C::Error> {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u16(self, cx: &C) -> Result<u16, C::Error> {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u64(self, cx: &C) -> Result<u64, C::Error> {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_u128(self, cx: &C) -> Result<u128, C::Error> {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_i8(self, cx: &C) -> Result<i8, C::Error> {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i16(self, cx: &C) -> Result<i16, C::Error> {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i32(self, cx: &C) -> Result<i32, C::Error> {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i64(self, cx: &C) -> Result<i64, C::Error> {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_i128(self, cx: &C) -> Result<i128, C::Error> {
        decode_typed_signed(cx, self.reader)
    }

    #[inline]
    fn decode_usize(self, cx: &C) -> Result<usize, C::Error> {
        decode_typed_unsigned(cx, self.reader)
    }

    #[inline]
    fn decode_isize(self, cx: &C) -> Result<isize, C::Error> {
        decode_typed_signed(cx, self.reader)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self, cx: &C) -> Result<f32, C::Error> {
        let bits = self.decode_u32(cx)?;
        Ok(f32::from_bits(bits))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self, cx: &C) -> Result<f64, C::Error> {
        let bits = self.decode_u64(cx)?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_option(mut self, cx: &C) -> Result<Option<Self::Some>, C::Error> {
        // Options are encoded as empty or sequences with a single element.
        const NONE: Tag = Tag::from_mark(Mark::None);
        const SOME: Tag = Tag::from_mark(Mark::Some);

        let pos = cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(cx.marked_message(
                pos,
                format_args! {
                    "Expected option, was {tag:?}"
                },
            )),
        }
    }

    #[inline]
    fn decode_sequence(self, cx: &C) -> Result<Self::Sequence, C::Error> {
        self.shared_decode_sequence(cx)
    }

    #[inline]
    fn decode_tuple(mut self, cx: &C, len: usize) -> Result<Self::Tuple, C::Error> {
        let pos = cx.mark();
        let actual = self.decode_prefix(cx, Kind::Sequence, pos)?;

        if len != actual {
            return Err(cx.message(format_args!(
                "tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(SelfTupleDecoder::new(self.reader))
    }

    #[inline]
    fn decode_map(self, cx: &C) -> Result<Self::Map, C::Error> {
        self.shared_decode_map(cx)
    }

    #[inline]
    fn decode_struct(self, cx: &C, _: Option<usize>) -> Result<Self::Struct, C::Error> {
        self.shared_decode_map(cx)
    }

    #[inline]
    fn decode_variant(mut self, cx: &C) -> Result<Self::Variant, C::Error> {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);

        let tag = Tag::from_byte(self.reader.read_byte(cx)?);

        if tag != VARIANT {
            return Err(cx.message(Expected {
                expected: Kind::Mark,
                actual: tag,
            }));
        }

        Ok(self)
    }

    #[inline]
    fn decode_any<V>(mut self, cx: &C, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, C>,
    {
        let tag = match self.reader.peek(cx)? {
            Some(b) => Tag::from_byte(b),
            None => return visitor.visit_any(cx, self, TypeHint::Any),
        };

        match tag.kind() {
            Kind::Number => match tag.data() {
                Some(U8) => {
                    let value = self.decode_u8(cx)?;
                    visitor.visit_u8(cx, value)
                }
                Some(U16) => {
                    let value = self.decode_u16(cx)?;
                    visitor.visit_u16(cx, value)
                }
                Some(U32) => {
                    let value = self.decode_u32(cx)?;
                    visitor.visit_u32(cx, value)
                }
                Some(U64) => {
                    let value = self.decode_u64(cx)?;
                    visitor.visit_u64(cx, value)
                }
                Some(U128) => {
                    let value = self.decode_u128(cx)?;
                    visitor.visit_u128(cx, value)
                }
                Some(I8) => {
                    let value = self.decode_i8(cx)?;
                    visitor.visit_i8(cx, value)
                }
                Some(I16) => {
                    let value = self.decode_i16(cx)?;
                    visitor.visit_i16(cx, value)
                }
                Some(I32) => {
                    let value = self.decode_i32(cx)?;
                    visitor.visit_i32(cx, value)
                }
                Some(I64) => {
                    let value = self.decode_i64(cx)?;
                    visitor.visit_i64(cx, value)
                }
                Some(I128) => {
                    let value = self.decode_i128(cx)?;
                    visitor.visit_i128(cx, value)
                }
                Some(F32) => {
                    let value = self.decode_f32(cx)?;
                    visitor.visit_f32(cx, value)
                }
                Some(F64) => {
                    let value = self.decode_f64(cx)?;
                    visitor.visit_f64(cx, value)
                }
                _ => {
                    let visitor = visitor.visit_number(cx, NumberHint::Any)?;
                    visitor.visit_any(cx, self, TypeHint::Number(NumberHint::Any))
                }
            },
            Kind::Sequence => {
                let sequence = self.shared_decode_sequence(cx)?;
                visitor.visit_sequence(cx, sequence)
            }
            Kind::Map => {
                let map = self.shared_decode_map(cx)?;
                visitor.visit_map(cx, map)
            }
            Kind::Bytes => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_bytes(cx, hint)?;
                self.decode_bytes(cx, visitor)
            }
            Kind::String => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_string(cx, hint)?;
                self.decode_string(cx, visitor)
            }
            Kind::Mark => match tag.mark() {
                Mark::True | Mark::False => {
                    let value = self.decode_bool(cx)?;
                    visitor.visit_bool(cx, value)
                }
                Mark::Variant => {
                    let value = self.decode_variant(cx)?;
                    visitor.visit_variant(cx, value)
                }
                Mark::Some | Mark::None => {
                    let value = self.decode_option(cx)?;
                    visitor.visit_option(cx, value)
                }
                Mark::Char => {
                    let value = self.decode_char(cx)?;
                    visitor.visit_char(cx, value)
                }
                Mark::Unit => {
                    self.decode_unit(cx)?;
                    visitor.visit_unit(cx)
                }
                _ => visitor.visit_any(cx, self, TypeHint::Any),
            },
            _ => visitor.visit_any(cx, self, TypeHint::Any),
        }
    }
}

impl<'de, C, R, const F: Options> PackDecoder<'de, C> for SelfDecoder<Limit<R>, F>
where
    C: Context,
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

impl<'de, C, R, const F: Options> PackDecoder<'de, C> for SelfTupleDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Decoder<'this> = SelfDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn next(&mut self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        Ok(SelfDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, R, const F: Options> RemainingSelfDecoder<R, F> {
    #[inline]
    fn new(remaining: usize, decoder: SelfDecoder<R, F>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'de, C, R, const F: Options> SequenceDecoder<'de, C> for RemainingSelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Decoder<'this> = SelfDecoder<R::Mut<'this>, F> where Self: 'this;

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
        Ok(Some(SelfDecoder::new(self.decoder.reader.borrow_mut())))
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

#[musli::map_decoder]
impl<'de, C, R, const F: Options> MapDecoder<'de, C> for RemainingSelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Entry<'this> = SelfDecoder<R::Mut<'this>, F>
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
        Ok(Some(SelfDecoder::new(self.decoder.reader.borrow_mut())))
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

impl<'de, C, R, const F: Options> MapPairsDecoder<'de, C> for RemainingSelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type MapPairsKey<'this> = SelfDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type MapPairsValue<'this> = SelfDecoder<R::Mut<'this>, F>
    where
        Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(SelfDecoder::new(self.decoder.reader.borrow_mut())))
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        Ok(SelfDecoder::new(self.decoder.reader.borrow_mut()))
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
            SelfDecoder::<_, F>::new(self.decoder.reader.borrow_mut()).skip_any(cx)?;
            SelfDecoder::<_, F>::new(self.decoder.reader.borrow_mut()).skip_any(cx)?;
        }

        Ok(())
    }
}

impl<'de, C, R, const F: Options> StructPairsDecoder<'de, C> for RemainingSelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type FieldName<'this> = SelfDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type FieldValue<'this> = SelfDecoder<R::Mut<'this>, F>
    where
        Self: 'this;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(cx.message("Ran out of fields"));
        }

        self.remaining -= 1;
        Ok(SelfDecoder::new(self.decoder.reader.borrow_mut()))
    }

    #[inline]
    fn field_value(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        MapPairsDecoder::map_pairs_value(self, cx)
    }

    #[inline]
    fn skip_field_value(&mut self, cx: &C) -> Result<bool, C::Error> {
        MapPairsDecoder::skip_map_pairs_value(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapPairsDecoder::end(self, cx)
    }
}

impl<'de, C, R, const F: Options> MapEntryDecoder<'de, C> for SelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type MapKey<'this> = SelfDecoder<R::Mut<'this>, F> where Self: 'this;
    type MapValue = Self;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        Ok(SelfDecoder::new(self.reader.borrow_mut()))
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
impl<'de, C, R, const F: Options> StructDecoder<'de, C> for RemainingSelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Field<'this> = SelfDecoder<R::Mut<'this>, F>
    where
        Self: 'this;
    type StructPairs = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::Exact(self.remaining)
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

impl<'de, C, R, const F: Options> StructFieldDecoder<'de, C> for SelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type FieldName<'this> = SelfDecoder<R::Mut<'this>, F> where Self: 'this;
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

impl<'de, C, R, const F: Options> VariantDecoder<'de, C> for SelfDecoder<R, F>
where
    C: Context,
    R: Reader<'de>,
{
    type Tag<'this> = SelfDecoder<R::Mut<'this>, F> where Self: 'this;
    type Variant<'this> = SelfDecoder<R::Mut<'this>, F> where Self: 'this;

    #[inline]
    fn tag(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error> {
        Ok(SelfDecoder::new(self.reader.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error> {
        Ok(SelfDecoder::new(self.reader.borrow_mut()))
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
