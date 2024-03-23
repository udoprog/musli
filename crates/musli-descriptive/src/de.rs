use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::de::{
    Decode, Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, NumberHint, NumberVisitor,
    PackDecoder, SequenceDecoder, SizeHint, StructDecoder, StructFieldDecoder, StructFieldsDecoder,
    TypeHint, ValueVisitor, VariantDecoder, Visitor,
};
use musli::Context;
use musli_common::int::continuation as c;
use musli_storage::de::StorageDecoder;

use crate::integer_encoding::{decode_typed_signed, decode_typed_unsigned};
use crate::options::Options;
use crate::reader::{Limit, Reader};
use crate::tag::{Kind, Mark, Tag, F32, F64, I128, I16, I32, I64, I8, U128, U16, U32, U64, U8};

#[cfg(feature = "musli-value")]
const BUFFER_OPTIONS: crate::options::Options = crate::options::new().build();

/// A very simple decoder.
pub struct SelfDecoder<'a, R, const F: Options, C: ?Sized> {
    cx: &'a C,
    reader: R,
}

impl<'a, R, const F: Options, C: ?Sized> SelfDecoder<'a, R, F, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(cx: &'a C, reader: R) -> Self {
        Self { cx, reader }
    }
}

pub struct SelfTupleDecoder<'a, R, const F: Options, C: ?Sized> {
    cx: &'a C,
    reader: R,
}

impl<'a, R, const F: Options, C: ?Sized> SelfTupleDecoder<'a, R, F, C> {
    #[inline]
    pub(crate) fn new(cx: &'a C, reader: R) -> Self {
        Self { cx, reader }
    }
}

impl<'a, 'de, R, const F: Options, C> SelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any(&mut self) -> Result<(), C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Number => {
                if tag.data().is_none() {
                    let _ = c::decode::<_, _, u128>(self.cx, self.reader.borrow_mut())?;
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
                    musli_common::int::decode_usize::<_, _, F>(self.cx, self.reader.borrow_mut())?
                };

                self.reader.skip(self.cx, len)?;
            }
            Kind::Pack => {
                let len = 2usize.pow(tag.data_raw() as u32);
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
            Kind::Map => {
                let len = if let Some(len) = tag.data() {
                    len as usize
                } else {
                    musli_common::int::decode_usize::<_, _, F>(self.cx, self.reader.borrow_mut())?
                };

                for _ in 0..len {
                    self.skip_any()?;
                    self.skip_any()?;
                }
            }
            kind => {
                return Err(self.cx.message(format_args!("Unsupported kind {kind:?}")));
            }
        }

        Ok(())
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_map(mut self) -> Result<RemainingSelfDecoder<'a, R, F, C>, C::Error> {
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Map, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(mut self) -> Result<RemainingSelfDecoder<'a, R, F, C>, C::Error> {
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Sequence, pos)?;
        Ok(RemainingSelfDecoder::new(len, self))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_prefix(&mut self, kind: Kind, mark: C::Mark) -> Result<usize, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag.kind() != kind {
            return Err(self.cx.marked_message(
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
            musli_common::int::decode_usize::<_, _, F>(self.cx, self.reader.borrow_mut())?
        })
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_pack_length(&mut self, start: C::Mark) -> Result<usize, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Bytes => Ok(if let Some(len) = tag.data() {
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
pub struct RemainingSelfDecoder<'a, R, const F: Options, C: ?Sized> {
    remaining: usize,
    decoder: SelfDecoder<'a, R, F, C>,
}

#[musli::decoder]
impl<'a, 'de, R, const F: Options, C> Decoder<'de> for SelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<'this, U> = SelfDecoder<'this, R, F, U> where U: 'this + Context;
    #[cfg(feature = "musli-value")]
    type DecodeBuffer = musli_value::AsValueDecoder<'a, BUFFER_OPTIONS, C>;
    type DecodePack = SelfDecoder<'a, Limit<R>, F, C>;
    type DecodeSome = Self;
    type DecodeSequence = RemainingSelfDecoder<'a, R, F, C>;
    type DecodeTuple = SelfTupleDecoder<'a, R, F, C>;
    type DecodeMap = RemainingSelfDecoder<'a, R, F, C>;
    type DecodeStruct = RemainingSelfDecoder<'a, R, F, C>;
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
        Ok(SelfDecoder::new(cx, self.reader))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the descriptive decoder")
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
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
    fn type_hint(&mut self) -> Result<TypeHint, C::Error> {
        let tag = match self.reader.peek(self.cx)? {
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

    #[cfg(feature = "musli-value")]
    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, C::Error> {
        let cx = self.cx;
        let value = self.decode::<musli_value::Value>()?;
        Ok(value.into_value_decoder(cx))
    }

    #[inline]
    fn decode_unit(mut self) -> Result<(), C::Error> {
        self.skip_any()?;
        Ok(())
    }

    #[inline]
    fn decode_pack(mut self) -> Result<Self::DecodePack, C::Error> {
        let pos = self.cx.mark();
        let len = self.decode_pack_length(pos)?;
        Ok(SelfDecoder::new(self.cx, self.reader.limit(len)))
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], C::Error> {
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Bytes, pos)?;

        if len != N {
            return Err(self.cx.marked_message(
                pos,
                format_args! {
                    "Bad length, got {len} but expect {N}"
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
        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::Bytes, pos)?;
        self.reader.read_bytes(self.cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
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

        let pos = self.cx.mark();
        let len = self.decode_prefix(Kind::String, pos)?;
        self.reader.read_bytes(self.cx, len, Visitor(visitor))
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, C::Error> {
        const FALSE: Tag = Tag::from_mark(Mark::False);
        const TRUE: Tag = Tag::from_mark(Mark::True);

        let pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag {
            FALSE => Ok(false),
            TRUE => Ok(true),
            tag => Err(self.cx.marked_message(
                pos,
                format_args! {
                    "Bad boolean, got {tag:?}"
                },
            )),
        }
    }

    #[inline]
    fn decode_char(mut self) -> Result<char, C::Error> {
        const CHAR: Tag = Tag::from_mark(Mark::Char);

        let pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag != CHAR {
            return Err(self
                .cx
                .marked_message(pos, format_args!("Expected {CHAR:?}, got {tag:?}")));
        }

        let num = c::decode(self.cx, self.reader.borrow_mut())?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(self.cx.marked_message(pos, format_args!("Bad character"))),
        }
    }

    #[inline]
    fn decode_number<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: NumberVisitor<'de, C>,
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
    fn decode_u8(self) -> Result<u8, C::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, C::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, C::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, C::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, C::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, C::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, C::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, C::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, C::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, C::Error> {
        decode_typed_signed(self.cx, self.reader)
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, C::Error> {
        decode_typed_unsigned(self.cx, self.reader)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, C::Error> {
        decode_typed_signed(self.cx, self.reader)
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
        const NONE: Tag = Tag::from_mark(Mark::None);
        const SOME: Tag = Tag::from_mark(Mark::Some);

        let pos = self.cx.mark();
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag {
            NONE => Ok(None),
            SOME => Ok(Some(self)),
            tag => Err(self.cx.marked_message(
                pos,
                format_args! {
                    "Expected option, was {tag:?}"
                },
            )),
        }
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::DecodeSequence, C::Error> {
        self.shared_decode_sequence()
    }

    #[inline]
    fn decode_tuple(mut self, len: usize) -> Result<Self::DecodeTuple, C::Error> {
        let pos = self.cx.mark();
        let actual = self.decode_prefix(Kind::Sequence, pos)?;

        if len != actual {
            return Err(self.cx.message(format_args!(
                "Tuple length mismatch: len: {len}, actual: {actual}"
            )));
        }

        Ok(SelfTupleDecoder::new(self.cx, self.reader))
    }

    #[inline]
    fn decode_map(self) -> Result<Self::DecodeMap, C::Error> {
        self.shared_decode_map()
    }

    #[inline]
    fn decode_struct(self, _: Option<usize>) -> Result<Self::DecodeStruct, C::Error> {
        self.shared_decode_map()
    }

    #[inline]
    fn decode_variant(mut self) -> Result<Self::DecodeVariant, C::Error> {
        const VARIANT: Tag = Tag::from_mark(Mark::Variant);

        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag != VARIANT {
            return Err(self.cx.message(Expected {
                expected: Kind::Mark,
                actual: tag,
            }));
        }

        Ok(self)
    }

    #[inline]
    fn decode_any<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: Visitor<'de, C>,
    {
        let cx = self.cx;

        let tag = match self.reader.peek(cx)? {
            Some(b) => Tag::from_byte(b),
            None => return visitor.visit_any(cx, self, TypeHint::Any),
        };

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
                _ => {
                    let visitor = visitor.visit_number(cx, NumberHint::Any)?;
                    visitor.visit_any(cx, self, TypeHint::Number(NumberHint::Any))
                }
            },
            Kind::Sequence => {
                let sequence = self.shared_decode_sequence()?;
                visitor.visit_sequence(cx, sequence)
            }
            Kind::Map => {
                let map = self.shared_decode_map()?;
                visitor.visit_map(cx, map)
            }
            Kind::Bytes => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_bytes(cx, hint)?;
                self.decode_bytes(visitor)
            }
            Kind::String => {
                let hint = tag
                    .data()
                    .map(|d| SizeHint::Exact(d as usize))
                    .unwrap_or_default();
                let visitor = visitor.visit_string(cx, hint)?;
                self.decode_string(visitor)
            }
            Kind::Mark => match tag.mark() {
                Mark::True | Mark::False => {
                    let value = self.decode_bool()?;
                    visitor.visit_bool(cx, value)
                }
                Mark::Variant => {
                    let value = self.decode_variant()?;
                    visitor.visit_variant(cx, value)
                }
                Mark::Some | Mark::None => {
                    let value = self.decode_option()?;
                    visitor.visit_option(cx, value)
                }
                Mark::Char => {
                    let value = self.decode_char()?;
                    visitor.visit_char(cx, value)
                }
                Mark::Unit => {
                    self.decode_unit()?;
                    visitor.visit_unit(cx)
                }
                _ => visitor.visit_any(cx, self, TypeHint::Any),
            },
            _ => visitor.visit_any(cx, self, TypeHint::Any),
        }
    }
}

impl<'a, 'de, R, const F: Options, C> PackDecoder<'de> for SelfDecoder<'a, Limit<R>, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
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

impl<'a, 'de, R, const F: Options, C> PackDecoder<'de> for SelfTupleDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeNext<'this> = SelfDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'a, R, const F: Options, C: ?Sized> RemainingSelfDecoder<'a, R, F, C> {
    #[inline]
    fn new(remaining: usize, decoder: SelfDecoder<'a, R, F, C>) -> Self {
        Self { remaining, decoder }
    }
}

impl<'a, 'de, R, const F: Options, C> SequenceDecoder<'de> for RemainingSelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeNext<'this> = SelfDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;

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
        Ok(Some(SelfDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        )))
    }
}

#[musli::map_decoder]
impl<'a, 'de, R, const F: Options, C> MapDecoder<'de> for RemainingSelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeEntry<'this> = SelfDecoder<'a, R::Mut<'this>, F, C>
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
        Ok(Some(SelfDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        )))
    }
}

impl<'a, 'de, R, const F: Options, C> MapEntriesDecoder<'de> for RemainingSelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeMapEntryKey<'this> = SelfDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;
    type DecodeMapEntryValue<'this> = SelfDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;

    #[inline]
    fn decode_map_entry_key(&mut self) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(SelfDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        )))
    }

    #[inline]
    fn decode_map_entry_value(&mut self) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        Ok(SelfDecoder::new(
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

impl<'a, 'de, R, const F: Options, C> StructFieldsDecoder<'de> for RemainingSelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeStructFieldName<'this> = SelfDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;
    type DecodeStructFieldValue<'this> = SelfDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;

    #[inline]
    fn decode_struct_field_name(&mut self) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        if self.remaining == 0 {
            return Err(self.decoder.cx.message("Ran out of fields"));
        }

        self.remaining -= 1;
        Ok(SelfDecoder::new(
            self.decoder.cx,
            self.decoder.reader.borrow_mut(),
        ))
    }

    #[inline]
    fn decode_struct_field_value(&mut self) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        MapEntriesDecoder::decode_map_entry_value(self)
    }

    #[inline]
    fn skip_struct_field_value(&mut self) -> Result<bool, C::Error> {
        MapEntriesDecoder::skip_map_entry_value(self)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        MapEntriesDecoder::end(self)
    }
}

impl<'a, 'de, R, const F: Options, C> MapEntryDecoder<'de> for SelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeMapKey<'this> = SelfDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;
    type DecodeMapValue = Self;

    #[inline]
    fn decode_map_key(&mut self) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_map_value(self) -> Result<Self::DecodeMapValue, C::Error> {
        Ok(self)
    }

    #[inline]
    fn skip_map_value(self) -> Result<bool, C::Error> {
        self.skip()?;
        Ok(true)
    }
}

#[musli::struct_decoder]
impl<'a, 'de, R, const F: Options, C> StructDecoder<'de> for RemainingSelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeField<'this> = SelfDecoder<'a, R::Mut<'this>, F, C>
    where
        Self: 'this;
    type IntoStructFields = Self;

    #[inline]
    fn cx(&self) -> &C {
        self.decoder.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::Exact(self.remaining)
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

impl<'a, 'de, R, const F: Options, C> StructFieldDecoder<'de> for SelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeFieldName<'this> = SelfDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;
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

impl<'a, 'de, R, const F: Options, C> VariantDecoder<'de> for SelfDecoder<'a, R, F, C>
where
    R: Reader<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeTag<'this> = SelfDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;
    type DecodeVariant<'this> = SelfDecoder<'a, R::Mut<'this>, F, C> where Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeVariant<'_>, C::Error> {
        Ok(SelfDecoder::new(self.cx, self.reader.borrow_mut()))
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
