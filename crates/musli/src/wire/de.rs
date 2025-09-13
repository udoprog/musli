use core::fmt;
use core::marker::PhantomData;
use core::mem::take;

use crate::alloc::Vec;
use crate::de::{
    Decoder, EntriesDecoder, EntryDecoder, MapDecoder, SequenceDecoder, SizeHint, Skip,
    UnsizedVisitor, VariantDecoder,
};
use crate::hint::{MapHint, SequenceHint};
use crate::int::continuation as c;
use crate::reader::Limit;
use crate::storage::de::StorageDecoder;
use crate::{Context, Options, Reader};

use super::tag::{Kind, Tag};

/// A very simple decoder.
pub struct WireDecoder<const OPT: Options, R, C, M> {
    cx: C,
    reader: R,
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, R, C, M> WireDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
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

impl<'de, const OPT: Options, R, C, M> WireDecoder<OPT, Limit<R>, C, M>
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

impl<'de, const OPT: Options, R, C, M> WireDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    /// Skip over any sequences of values.
    pub(crate) fn skip_any(mut self) -> Result<(), C::Error> {
        let mut remaining = 1;

        while remaining > 0 {
            remaining -= 1;

            let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

            match tag.kind() {
                Kind::Prefix => {
                    let len = if let Some(len) = tag.data() {
                        len as usize
                    } else {
                        crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?
                    };

                    self.reader.skip(self.cx, len)?;
                }
                Kind::Sequence => {
                    let len = if let Some(len) = tag.data() {
                        len as usize
                    } else {
                        crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?
                    };

                    remaining += len;
                }
                Kind::Continuation => {
                    if tag.data().is_none() {
                        let _ = c::decode::<_, _, u128>(self.cx, self.reader.borrow_mut())?;
                    }
                }
                kind => {
                    return Err(self
                        .cx
                        .message(format_args!("Cannot skip over kind {kind:?}")));
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
                crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?
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
    ) -> Result<RemainingWireDecoder<OPT, R, C, M>, C::Error> {
        let len = self.decode_sequence_len()?;
        Ok(RemainingWireDecoder::new(self.cx, self.reader, len / 2))
    }

    // Standard function for decoding a pair sequence.
    #[inline]
    fn shared_decode_sequence(mut self) -> Result<RemainingWireDecoder<OPT, R, C, M>, C::Error> {
        let len = self.decode_sequence_len()?;
        Ok(RemainingWireDecoder::new(self.cx, self.reader, len))
    }

    /// Decode the length of a prefix.
    #[inline]
    fn decode_len(&mut self, start: &C::Mark) -> Result<usize, C::Error> {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        match tag.kind() {
            Kind::Prefix => Ok(if let Some(len) = tag.data() {
                len as usize
            } else {
                crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?
            }),
            kind => Err(self
                .cx
                .message_at(start, format_args!("Expected prefix, but got {kind:?}"))),
        }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
pub struct RemainingWireDecoder<const OPT: Options, R, C, M> {
    cx: C,
    reader: R,
    remaining: usize,
    _marker: PhantomData<M>,
}

impl<'de, const OPT: Options, R, C, M> RemainingWireDecoder<OPT, R, C, M>
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
        loop {
            let Some(value) = SequenceDecoder::try_decode_next(&mut self)? else {
                break;
            };

            value.skip()?;
        }

        Ok(())
    }

    #[inline]
    fn skip_remaining_entries(mut self) -> Result<(), C::Error> {
        loop {
            let Some(value) = self.decode_entry_key()? else {
                break;
            };

            value.skip()?;
            self.decode_entry_value()?.skip()?;
        }

        Ok(())
    }
}

#[crate::trait_defaults(crate)]
impl<'de, const OPT: Options, R, C, M> Decoder<'de> for WireDecoder<OPT, R, C, M>
where
    R: Reader<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = WireDecoder<OPT, R::TryClone, C, M>;
    type DecodePack = WireDecoder<OPT, Limit<R>, C, M>;
    type DecodeSome = Self;
    type DecodeSequence = RemainingWireDecoder<OPT, R, C, M>;
    type DecodeMap = RemainingWireDecoder<OPT, R, C, M>;
    type DecodeMapEntries = RemainingWireDecoder<OPT, R, C, M>;
    type DecodeVariant = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the wire decoder")
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(WireDecoder::new(self.cx, self.reader.try_clone()?))
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
    fn decode_empty(self) -> Result<(), Self::Error> {
        self.skip()
    }

    #[inline]
    fn decode_pack<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack<'_>) -> Result<O, Self::Error>,
    {
        let mark = self.cx.mark();
        let len = self.decode_len(&mark)?;
        let mut decoder = WireDecoder::new(self.cx, self.reader.limit(len));
        let output = f(&mut decoder)?;
        decoder.end()?;
        Ok(output)
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], Self::Error> {
        let mark = self.cx.mark();
        let len = self.decode_len(&mark)?;

        if len != N {
            return Err(self.cx.message_at(
                &mark,
                BadLength {
                    actual: len,
                    expected: N,
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
        let mark = self.cx.mark();
        let len = self.decode_len(&mark)?;
        self.reader.read_bytes(self.cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
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
                bytes: Vec<u8, C::Allocator>,
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

        self.decode_bytes(Visitor(visitor))
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, Self::Error> {
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
    fn decode_char(self) -> Result<char, Self::Error> {
        let cx = self.cx;
        let num = self.decode_u32()?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.message(BadCharacter(num))),
        }
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        crate::wire::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        crate::wire::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        crate::wire::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        crate::wire::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        crate::wire::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        Ok(self.decode_u8()? as i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        crate::wire::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        crate::wire::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        crate::wire::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        crate::wire::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
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
    fn decode_usize(self) -> Result<usize, Self::Error> {
        crate::wire::int::decode_length::<_, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        Ok(self.decode_usize()? as isize)
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, Self::Error> {
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
    fn decode_sequence_hint<F, O>(self, _: impl SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        self.decode_sequence(f)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        let mut decoder = self.shared_decode_pair_sequence()?;
        let output = f(&mut decoder)?;
        decoder.skip_remaining_entries()?;
        Ok(output)
    }

    #[inline]
    fn decode_map_hint<F, O>(self, _: impl MapHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_variant<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        let tag = Tag::from_byte(self.reader.read_byte(self.cx)?);

        if tag != Tag::new(Kind::Sequence, 2) {
            return Err(self.cx.message(Expected {
                expected: Kind::Sequence,
                actual: tag,
            }));
        }

        f(&mut self)
    }
}

impl<'de, const OPT: Options, R, C, M> SequenceDecoder<'de> for WireDecoder<OPT, Limit<R>, C, M>
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
        Ok(Some(self.decode_next()?))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

impl<'de, const OPT: Options, R, C, M> SequenceDecoder<'de> for RemainingWireDecoder<OPT, R, C, M>
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
        = WireDecoder<OPT, R::Mut<'this>, C, M>
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
        Ok(Some(WireDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        if self.remaining == 0 {
            return Err(self
                .cx
                .message(format_args!("No more tuple elements to decode")));
        }

        self.remaining -= 1;
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

impl<'de, const OPT: Options, R, C, M> VariantDecoder<'de> for WireDecoder<OPT, R, C, M>
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
        = WireDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = WireDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error> {
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error> {
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

impl<'de, const OPT: Options, R, C, M> MapDecoder<'de> for RemainingWireDecoder<OPT, R, C, M>
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
        = WireDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = RemainingWireDecoder<OPT, R::Mut<'this>, C, M>
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
        Ok(Some(WireDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, Self::Error> {
        Ok(RemainingWireDecoder::new(
            self.cx,
            self.reader.borrow_mut(),
            take(&mut self.remaining),
        ))
    }
}

impl<'de, const OPT: Options, R, C, M> EntryDecoder<'de> for WireDecoder<OPT, R, C, M>
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
        = WireDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, Self::Error> {
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, Self::Error> {
        Ok(self)
    }
}

impl<'de, const OPT: Options, R, C, M> EntriesDecoder<'de> for RemainingWireDecoder<OPT, R, C, M>
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
        = WireDecoder<OPT, R::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = WireDecoder<OPT, R::Mut<'this>, C, M>
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
        Ok(Some(WireDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        Ok(WireDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        self.skip_remaining_entries()?;
        Ok(())
    }
}

struct Expected {
    expected: Kind,
    actual: Tag,
}

impl fmt::Display for Expected {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = *self;
        write!(f, "Expected {expected:?} but was {actual:?}")
    }
}

struct BadBoolean {
    actual: Tag,
}

impl fmt::Display for BadBoolean {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Bad boolean tag {actual:?}")
    }
}

struct BadCharacter(u32);

impl fmt::Display for BadCharacter {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bad character number 0x{:02x}", self.0)
    }
}

struct ExpectedOption {
    tag: Tag,
}

impl fmt::Display for ExpectedOption {
    #[inline]
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
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual, expected } = *self;
        write!(f, "Bad length, got {actual} but expect {expected}")
    }
}
