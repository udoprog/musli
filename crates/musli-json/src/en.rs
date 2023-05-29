use core::{fmt, marker};

use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::mode::Mode;
use musli::Context;
use musli_common::writer::Writer;

use crate::error::Error;

/// A JSON encoder for Müsli.
pub struct JsonEncoder<M, W> {
    writer: W,
    _marker: marker::PhantomData<M>,
}

impl<M, W> JsonEncoder<M, W> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self {
            writer,
            _marker: marker::PhantomData,
        }
    }
}

#[musli::encoder]
impl<M, W> Encoder for JsonEncoder<M, W>
where
    M: Mode,
    W: Writer,
    Error: From<W::Error>,
{
    type Error = Error;
    type Ok = ();
    type Pack = JsonArrayEncoder<M, W>;
    type Some = Self;
    type Sequence = JsonArrayEncoder<M, W>;
    type Tuple = JsonArrayEncoder<M, W>;
    type Map = JsonObjectEncoder<M, W>;
    type Struct = JsonObjectEncoder<M, W>;
    type Variant = JsonVariantEncoder<M, W>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be encoded to JSON")
    }

    #[inline]
    fn encode_unit<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_bytes(cx.adapt(), b"null")
    }

    #[inline]
    fn encode_bool<'buf, C>(mut self, cx: &mut C, value: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer
            .write_bytes(cx.adapt(), if value { b"true" } else { b"false" })
    }

    #[inline]
    fn encode_char<'buf, C>(mut self, cx: &mut C, value: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_string(
            cx,
            self.writer.borrow_mut(),
            value.encode_utf8(&mut [0, 0, 0, 0]).as_bytes(),
        )
    }

    #[inline]
    fn encode_u8<'buf, C>(mut self, cx: &mut C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u16<'buf, C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u32<'buf, C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u64<'buf, C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u128<'buf, C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i8<'buf, C>(mut self, cx: &mut C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i16<'buf, C>(mut self, cx: &mut C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i32<'buf, C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i64<'buf, C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i128<'buf, C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_usize<'buf, C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_isize<'buf, C>(mut self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = itoa::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f32<'buf, C>(mut self, cx: &mut C, value: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = ryu::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f64<'buf, C>(mut self, cx: &mut C, value: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buffer = ryu::Buffer::new();
        self.writer
            .write_bytes(cx.adapt(), buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_array<'buf, C, const N: usize>(
        self,
        cx: &mut C,
        bytes: [u8; N],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_bytes(cx, bytes.as_slice())
    }

    #[inline]
    fn encode_bytes<'buf, C>(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut buf = itoa::Buffer::new();
        let mut it = bytes.iter();
        let last = it.next_back();

        self.writer.write_byte(cx.adapt(), b'[')?;

        for b in it {
            self.writer
                .write_bytes(cx.adapt(), buf.format(*b).as_bytes())?;
            self.writer.write_byte(cx.adapt(), b',')?;
        }

        if let Some(b) = last {
            self.writer
                .write_bytes(cx.adapt(), buf.format(*b).as_bytes())?;
        }

        self.writer.write_byte(cx.adapt(), b']')?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<'buf, C>(
        self,
        cx: &mut C,
        bytes: &[&[u8]],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut seq = JsonArrayEncoder::<M, _>::new(cx, self.writer)?;

        for bb in bytes {
            for b in *bb {
                seq.push::<M, _, _>(cx, b)?;
            }
        }

        seq.end(cx)
    }

    #[inline]
    fn encode_string<'buf, C>(mut self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_string(cx, self.writer.borrow_mut(), string.as_bytes())
    }

    #[inline]
    fn encode_some<'buf, C>(self, _: &mut C) -> Result<Self::Some, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(self)
    }

    #[inline]
    fn encode_none<'buf, C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.encode_unit(cx)
    }

    #[inline]
    fn encode_pack<'buf, C>(self, cx: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonArrayEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_sequence<'buf, C>(self, cx: &mut C, _: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonArrayEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_tuple<'buf, C>(self, cx: &mut C, _: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonArrayEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_map<'buf, C>(self, cx: &mut C, _: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonObjectEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_struct<'buf, C>(self, cx: &mut C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonObjectEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_variant<'buf, C>(self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        JsonVariantEncoder::new(cx, self.writer)
    }
}

/// Encoder for a pairs sequence.
pub struct JsonObjectEncoder<M, W> {
    len: usize,
    writer: W,
    _marker: marker::PhantomData<M>,
}

impl<M, W> JsonObjectEncoder<M, W>
where
    W: Writer,
    Error: From<W::Error>,
{
    #[inline]
    fn new<'buf, C>(cx: &mut C, mut writer: W) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        writer.write_byte(cx.adapt(), b'{')?;

        Ok(Self {
            len: 0,
            writer,
            _marker: marker::PhantomData,
        })
    }
}

impl<M, W> PairsEncoder for JsonObjectEncoder<M, W>
where
    M: Mode,
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;

    type Encoder<'this> = JsonObjectPairEncoder<M, W::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let len = self.len;
        self.len += 1;
        Ok(JsonObjectPairEncoder::new(
            len == 0,
            self.writer.borrow_mut(),
        ))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_byte(cx.adapt(), b'}')?;
        Ok(())
    }
}

/// Encoder for a pair.
pub struct JsonObjectPairEncoder<M, W> {
    empty: bool,
    writer: W,
    _marker: marker::PhantomData<M>,
}

impl<M, W> JsonObjectPairEncoder<M, W> {
    #[inline]
    const fn new(empty: bool, writer: W) -> Self {
        Self {
            empty,
            writer,
            _marker: marker::PhantomData,
        }
    }
}

impl<M, W> PairEncoder for JsonObjectPairEncoder<M, W>
where
    M: Mode,
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;

    type First<'this> = JsonObjectKeyEncoder<W::Mut<'this>>
    where
        Self: 'this;

    type Second<'this> = JsonEncoder<M, W::Mut<'this>> where Self: 'this;

    #[inline]
    fn first<'buf, C>(&mut self, cx: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if !self.empty {
            self.writer.write_byte(cx.adapt(), b',')?;
        }

        Ok(JsonObjectKeyEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn second<'buf, C>(&mut self, cx: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_byte(cx.adapt(), b':')?;
        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }
}

/// Encoder for a pair.
pub struct JsonVariantEncoder<M, W> {
    writer: W,
    _marker: marker::PhantomData<M>,
}

impl<M, W> JsonVariantEncoder<M, W>
where
    W: Writer,
    Error: From<W::Error>,
{
    #[inline]
    fn new<'buf, C>(cx: &mut C, mut writer: W) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        writer.write_byte(cx.adapt(), b'{')?;

        Ok(Self {
            writer,
            _marker: marker::PhantomData,
        })
    }
}

impl<M, W> VariantEncoder for JsonVariantEncoder<M, W>
where
    M: Mode,
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;

    type Tag<'this> = JsonObjectKeyEncoder<W::Mut<'this>>
    where
        Self: 'this;

    type Variant<'this> = JsonEncoder<M, W::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(JsonObjectKeyEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn variant<'buf, C>(&mut self, cx: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_byte(cx.adapt(), b':')?;
        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_byte(cx.adapt(), b'}')
    }
}

pub struct JsonObjectKeyEncoder<W> {
    writer: W,
}

impl<W> JsonObjectKeyEncoder<W> {
    #[inline]
    fn new(writer: W) -> Self {
        Self { writer }
    }
}

macro_rules! format_integer {
    ($slf:ident, $cx:expr, $value:ident) => {{
        $slf.writer.write_byte($cx.adapt(), b'"')?;
        let mut buffer = itoa::Buffer::new();
        $slf.writer
            .write_bytes($cx.adapt(), buffer.format($value).as_bytes())?;
        $slf.writer.write_byte($cx.adapt(), b'"')?;
        Ok(())
    }};
}

#[musli::encoder]
impl<W> Encoder for JsonObjectKeyEncoder<W>
where
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any type that can be used as an object key")
    }

    #[inline]
    fn encode_u8<'buf, C>(mut self, cx: &mut C, value: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u16<'buf, C>(mut self, cx: &mut C, value: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u32<'buf, C>(mut self, cx: &mut C, value: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u64<'buf, C>(mut self, cx: &mut C, value: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_u128<'buf, C>(mut self, cx: &mut C, value: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i8<'buf, C>(mut self, cx: &mut C, value: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i16<'buf, C>(mut self, cx: &mut C, value: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i32<'buf, C>(mut self, cx: &mut C, value: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i64<'buf, C>(mut self, cx: &mut C, value: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_i128<'buf, C>(mut self, cx: &mut C, value: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_usize<'buf, C>(mut self, cx: &mut C, value: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_isize<'buf, C>(mut self, cx: &mut C, value: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        format_integer!(self, cx, value)
    }

    #[inline]
    fn encode_string<'buf, C>(self, cx: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        encode_string(cx, self.writer, string.as_bytes())
    }
}

/// Encoder for a pairs sequence.
pub struct JsonArrayEncoder<M, W> {
    first: bool,
    writer: W,
    _marker: marker::PhantomData<M>,
}

impl<M, W> JsonArrayEncoder<M, W>
where
    W: Writer,
    Error: From<W::Error>,
{
    #[inline]
    fn new<'buf, C>(cx: &mut C, mut writer: W) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        writer.write_byte(cx.adapt(), b'[')?;

        Ok(Self {
            first: true,
            writer,
            _marker: marker::PhantomData,
        })
    }
}

impl<M, W> SequenceEncoder for JsonArrayEncoder<M, W>
where
    M: Mode,
    W: Writer,
    Error: From<W::Error>,
{
    type Ok = ();
    type Error = Error;

    type Encoder<'this> = JsonEncoder<M, W::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn next<'buf, C>(&mut self, cx: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if !self.first {
            self.writer.write_byte(cx.adapt(), b',')?;
        }

        self.first = false;
        Ok(JsonEncoder::new(self.writer.borrow_mut()))
    }

    #[inline]
    fn end<'buf, C>(mut self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.writer.write_byte(cx.adapt(), b']')?;
        Ok(())
    }
}

/// Encode a sequence of chars as a string.
#[inline]
fn encode_string<'buf, C, W>(cx: &mut C, mut writer: W, bytes: &[u8]) -> Result<(), C::Error>
where
    C: Context<'buf, Input = Error>,
    W: Writer,
    Error: From<W::Error>,
{
    writer.write_byte(cx.adapt(), b'"')?;

    let mut start = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let escape = ESCAPE[b as usize];

        if escape == 0 {
            continue;
        }

        if start < i {
            writer.write_bytes(cx.adapt(), &bytes[start..i])?;
        }

        write_escape(cx, &mut writer, escape, b)?;
        start = i + 1;
    }

    if start != bytes.len() {
        writer.write_bytes(cx.adapt(), &bytes[start..])?;
    }

    writer.write_byte(cx.adapt(), b'"')?;
    Ok(())
}

// Parts below copied from serde-json under the MIT license:
//
// https://github.com/serde-rs/json

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
static ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

// Hex digits.
static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";

fn write_escape<'buf, C, W>(
    cx: &mut C,
    writer: &mut W,
    escape: u8,
    byte: u8,
) -> Result<(), C::Error>
where
    C: Context<'buf, Input = Error>,
    W: Writer,
    Error: From<W::Error>,
{
    let s = match escape {
        BB => b"\\b",
        TT => b"\\t",
        NN => b"\\n",
        FF => b"\\f",
        RR => b"\\r",
        QU => b"\\\"",
        BS => b"\\\\",
        UU => {
            let bytes = &[
                b'\\',
                b'u',
                b'0',
                b'0',
                HEX_DIGITS[(byte >> 4) as usize],
                HEX_DIGITS[(byte & 0xF) as usize],
            ];
            return writer.write_bytes(cx.adapt(), bytes);
        }
        _ => unreachable!(),
    };

    writer.write_bytes(cx.adapt(), s)
}
