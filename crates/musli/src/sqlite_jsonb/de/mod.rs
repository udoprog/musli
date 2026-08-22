mod key_decoder;
use self::key_decoder::JsonbKeyDecoder;

mod object_decoder;
use self::object_decoder::JsonbObjectDecoder;

mod object_pair_decoder;
use self::object_pair_decoder::JsonbObjectPairDecoder;

mod sequence_decoder;
use self::sequence_decoder::JsonbSequenceDecoder;

mod variant_decoder;
use self::variant_decoder::JsonbVariantDecoder;

use core::fmt;
use core::marker::PhantomData;

use crate::alloc::Vec;
use crate::de::{Decoder, SequenceDecoder, SizeHint, Skip, UnsizedVisitor, Visitor};
use crate::hint::{MapHint, SequenceHint};
use crate::value::{IntoValueDecoder, Value};
use crate::{Context, Options, options};

use super::cursor::{Cursor, SliceCursor};
use crate::number::{Float, Integer};

use super::parse::{is_escaped, parse_any, parse_float, parse_integer, unescape};
use super::tag::{ARRAY, FALSE, INT, Kind, NULL, OBJECT, TRUE, is_float, is_int, is_text};

// JSONB, like JSON, stores object keys as strings, so buffered values have to
// permit treating them as numbers again.
const BUFFER_OPTIONS: Options = options::new().map_keys_as_numbers().build();

/// Read the header of the next element, returning its type and the size of its
/// payload.
///
/// The upper four bits of the first byte determine how large the header is and
/// where the payload size is stored, the lower four bits are the element type.
fn read_header<'de, P, C>(cx: C, p: &mut P) -> Result<(u8, usize), C::Error>
where
    P: Cursor<'de>,
    C: Context,
{
    let first = p.read_byte(cx)?;
    let kind = first & 0x0f;

    let len = match first >> 4 {
        len @ 0..=11 => usize::from(len),
        12 => usize::from(p.read_byte(cx)?),
        13 => {
            let bytes = p.read_slice(cx, 2)?;
            usize::from(u16::from_be_bytes([bytes[0], bytes[1]]))
        }
        14 => {
            let bytes = p.read_slice(cx, 4)?;
            let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            into_len(cx, u64::from(len))?
        }
        _ => {
            let bytes = p.read_slice(cx, 8)?;
            let mut buf = [0; 8];
            buf.copy_from_slice(bytes);
            into_len(cx, u64::from_be_bytes(buf))?
        }
    };

    if kind > OBJECT {
        return Err(cx.message(format_args!("Unsupported element type {}", Kind(kind))));
    }

    Ok((kind, len))
}

#[inline]
fn into_len<C>(cx: C, len: u64) -> Result<usize, C::Error>
where
    C: Context,
{
    let Ok(len) = usize::try_from(len) else {
        return Err(cx.message(format_args!("Element of {len} bytes is too large")));
    };

    Ok(len)
}

/// A JSONB decoder for Müsli.
pub(crate) struct JsonbDecoder<P, C, M> {
    cx: C,
    cursor: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonbDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    /// Construct a new JSONB decoder.
    #[inline]
    pub(crate) fn new(cx: C, cursor: P) -> Self {
        Self {
            cx,
            cursor,
            _marker: PhantomData,
        }
    }

    /// Read the next element in full, returning its type and its payload.
    #[inline]
    fn element(&mut self) -> Result<(u8, &'de [u8]), C::Error> {
        let (kind, len) = read_header(self.cx, &mut self.cursor)?;
        let payload = self.cursor.read_slice(self.cx, len)?;
        Ok((kind, payload))
    }

    /// Read the next element, requiring it to be of one of the types accepted
    /// by `expected`.
    #[inline]
    fn expect(
        &mut self,
        expected: impl Fn(u8) -> bool,
        what: &'static str,
    ) -> Result<(u8, &'de [u8]), C::Error> {
        let mark = self.cx.mark();
        let (kind, payload) = self.element()?;

        if !expected(kind) {
            return Err(self.cx.message_at(
                &mark,
                format_args!("Expected {what}, but was {}", Kind(kind)),
            ));
        }

        Ok((kind, payload))
    }

    /// Read the payload of the next element, requiring it to be a container of
    /// the given type, as a cursor over its contents.
    #[inline]
    fn container(&mut self, expected: u8) -> Result<SliceCursor<'de>, C::Error> {
        let (_, payload) = self.expect(|kind| kind == expected, kind_name(expected))?;
        Ok(SliceCursor::new(payload))
    }

    /// Peek at the type of the next element without consuming it.
    #[inline]
    fn peek_kind(&self) -> Option<u8> {
        Some(self.cursor.peek()? & 0x0f)
    }

    #[inline]
    fn decode_integer<T>(mut self) -> Result<T, C::Error>
    where
        T: Integer,
    {
        let (kind, payload) = self.expect(is_int, "integer")?;
        parse_integer(self.cx, kind, payload)
    }

    #[inline]
    fn decode_number_as<T>(mut self) -> Result<T, C::Error>
    where
        T: Float,
    {
        let (kind, payload) = self.expect(|kind| is_int(kind) || is_float(kind), "number")?;
        parse_float(self.cx, kind, payload)
    }
}

#[inline]
const fn kind_name(kind: u8) -> &'static str {
    match kind {
        ARRAY => "array",
        OBJECT => "object",
        _ => "element",
    }
}

/// Decode the payload of a string element, which for the escaped string types
/// has to be translated through `scratch` first.
#[inline]
pub(super) fn decode_text<'de, 'scratch, C>(
    cx: C,
    kind: u8,
    payload: &'de [u8],
    scratch: &'scratch mut Vec<u8, C::Allocator>,
) -> Result<Text<'de, 'scratch>, C::Error>
where
    C: Context,
{
    if !is_escaped(kind) {
        return Ok(Text::Borrowed(payload));
    }

    unescape(cx, kind, payload, scratch)?;
    Ok(Text::Scratch(scratch.as_slice()))
}

/// The contents of a string element, which is borrowed out of the input unless
/// it had to be unescaped.
pub(super) enum Text<'de, 'scratch> {
    Borrowed(&'de [u8]),
    Scratch(&'scratch [u8]),
}

#[crate::trait_defaults(crate)]
impl<'de, P, C, M> Decoder<'de> for JsonbDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = JsonbDecoder<P::TryClone, C, M>;
    type DecodeBuffer = IntoValueDecoder<BUFFER_OPTIONS, C, C::Allocator, M>;
    type DecodePack = JsonbSequenceDecoder<SliceCursor<'de>, C, M>;
    type DecodeSome = Self;
    type DecodeSequence = JsonbSequenceDecoder<SliceCursor<'de>, C, M>;
    type DecodeMap = JsonbObjectDecoder<SliceCursor<'de>, C, M>;
    type DecodeMapEntries = JsonbObjectDecoder<SliceCursor<'de>, C, M>;
    type DecodeVariant = JsonbVariantDecoder<SliceCursor<'de>, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from JSONB")
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(JsonbDecoder::new(self.cx, self.cursor.try_clone()?))
    }

    #[inline]
    fn skip(mut self) -> Result<(), Self::Error> {
        // Every element knows the size of its payload, including the containers
        // whose payload is the elements they contain, so skipping never has to
        // look at the contents.
        self.element()?;
        Ok(())
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
    fn decode_bool(mut self) -> Result<bool, Self::Error> {
        let (kind, _) = self.expect(|kind| matches!(kind, TRUE | FALSE), "boolean")?;
        Ok(kind == TRUE)
    }

    #[inline]
    fn decode_char(mut self) -> Result<char, Self::Error> {
        let mark = self.cx.mark();
        let (kind, payload) = self.expect(is_text, "string")?;
        let mut scratch = Vec::new_in(self.cx.alloc());

        let bytes = match decode_text(self.cx, kind, payload, &mut scratch)? {
            Text::Borrowed(bytes) => bytes,
            Text::Scratch(bytes) => bytes,
        };

        let string = crate::str::from_utf8(bytes).map_err(self.cx.map())?;

        let mut it = string.chars();

        match (it.next(), it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(self
                .cx
                .message_at(&mark, "Expected string with a single character")),
        }
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.decode_integer()
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.decode_number_as()
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        self.decode_number_as()
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        let cx = self.cx;
        let mark = cx.mark();

        self.decode_sequence(|seq| {
            let mut bytes = [0; N];
            let mut index = 0;

            while let Some(item) = seq.try_decode_next()? {
                if index < N {
                    bytes[index] = item.decode_u8()?;
                } else {
                    item.skip()?;
                }

                index += 1;
            }

            if index != N {
                return Err(cx.message_at(
                    &mark,
                    format_args!(
                        "Array with length {index} does not have the expected {N} number of elements"
                    ),
                ));
            }

            Ok(bytes)
        })
    }

    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, [u8], Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;

        self.decode_sequence(|seq| {
            let mut bytes = Vec::with_capacity_in(seq.size_hint().or_default(), cx.alloc())
                .map_err(cx.map())?;

            while let Some(item) = seq.try_decode_next()? {
                let b = item.decode_u8()?;
                bytes.push(b).map_err(cx.map())?;
            }

            visitor.visit_owned(cx, bytes)
        })
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;
        let (kind, payload) = self.expect(is_text, "string")?;
        let mut scratch = Vec::new_in(cx.alloc());

        match decode_text(cx, kind, payload, &mut scratch)? {
            Text::Borrowed(bytes) => {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                visitor.visit_borrowed(cx, string)
            }
            Text::Scratch(bytes) => {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                visitor.visit_ref(cx, string)
            }
        }
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        if self.peek_kind() == Some(NULL) {
            self.element()?;
            return Ok(None);
        }

        Ok(Some(self))
    }

    #[inline]
    fn decode_pack<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonbSequenceDecoder::new(self.cx, self.container(ARRAY)?);
        let output = f(&mut decoder)?;
        decoder.end()?;
        Ok(output)
    }

    #[inline]
    fn decode_sequence<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonbSequenceDecoder::new(self.cx, self.container(ARRAY)?);
        let output = f(&mut decoder)?;
        decoder.end()?;
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
    fn decode_map<F, O>(mut self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        let mut decoder = JsonbObjectDecoder::new(self.cx, self.container(OBJECT)?);
        let output = f(&mut decoder)?;
        decoder.end()?;
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
        let mut decoder = JsonbVariantDecoder::new(self.cx, self.container(OBJECT)?);
        let output = f(&mut decoder)?;
        decoder.end()?;
        Ok(output)
    }

    #[inline]
    fn decode_number<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;
        let (kind, payload) = self.expect(|kind| is_int(kind) || is_float(kind), "number")?;
        visit_number(cx, kind, payload, visitor)
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let cx = self.cx;

        let Some(kind) = self.peek_kind() else {
            return Err(cx.message("Expected element in input"));
        };

        match kind {
            NULL => {
                self.skip()?;
                visitor.visit_empty(cx)
            }
            TRUE | FALSE => {
                let value = self.decode_bool()?;
                visitor.visit_bool(cx, value)
            }
            kind if is_int(kind) || is_float(kind) => self.decode_number(visitor),
            kind if is_text(kind) => {
                let visitor = visitor.visit_string(cx, SizeHint::any())?;
                self.decode_string(visitor)
            }
            ARRAY => self.decode_sequence(|decoder| visitor.visit_sequence(decoder)),
            OBJECT => self.decode_map(|decoder| visitor.visit_map(decoder)),
            kind => Err(cx.message(format_args!("Unsupported element type {}", Kind(kind)))),
        }
    }
}

/// Hand a number payload to the visitor, picking the narrowest type which
/// holds it exactly.
pub(super) fn visit_number<'de, C, V>(
    cx: C,
    kind: u8,
    payload: &[u8],
    visitor: V,
) -> Result<V::Ok, V::Error>
where
    C: Context,
    V: Visitor<'de, C, Error = C::Error, Allocator = C::Allocator>,
{
    if is_float(kind) {
        let value = parse_float(cx, kind, payload)?;
        return visitor.visit_f64(cx, value);
    }

    let any = parse_any(cx, kind, payload)?;
    crate::number::visit_any(cx, any, visitor)
}

/// Decode an integer which has been stored as an object key, and is therefore
/// one of the string element types.
#[inline]
pub(super) fn decode_key_integer<'de, T, P, C>(cx: C, cursor: &mut P) -> Result<T, C::Error>
where
    T: Integer,
    P: Cursor<'de>,
    C: Context,
{
    let mark = cx.mark();
    let (kind, len) = read_header(cx, cursor)?;
    let payload = cursor.read_slice(cx, len)?;

    if !is_text(kind) {
        return Err(cx.message_at(
            &mark,
            format_args!("Expected object key, but was {}", Kind(kind)),
        ));
    }

    let mut scratch = Vec::new_in(cx.alloc());

    match decode_text(cx, kind, payload, &mut scratch)? {
        Text::Borrowed(bytes) => parse_integer(cx, INT, bytes),
        Text::Scratch(bytes) => parse_integer(cx, INT, bytes),
    }
}
