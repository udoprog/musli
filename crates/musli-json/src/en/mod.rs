mod array_encoder;
use self::array_encoder::JsonArrayEncoder;

mod object_encoder;
use self::object_encoder::JsonObjectEncoder;

mod object_key_encoder;
use self::object_key_encoder::JsonObjectKeyEncoder;

mod object_pair_encoder;
use self::object_pair_encoder::JsonObjectPairEncoder;

mod variant_encoder;
use self::variant_encoder::JsonVariantEncoder;

use core::fmt;

use musli::en::{Encoder, SequenceEncoder};
use musli::{Context, Encode};
use musli_common::writer::Writer;

/// A JSON encoder for Müsli.
pub(crate) struct JsonEncoder<W> {
    writer: W,
}

impl<W> JsonEncoder<W> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }
}

#[musli::encoder]
impl<C, W> Encoder<C> for JsonEncoder<W>
where
    W: Writer,
    C: ?Sized + Context,
{
    type Ok = ();
    type WithContext<U> = Self where U: Context;
    type EncodePack<'this> = JsonArrayEncoder<W> where C: 'this;
    type EncodeSome = Self;
    type EncodeSequence = JsonArrayEncoder<W>;
    type EncodeTuple = JsonArrayEncoder<W>;
    type EncodeMap = JsonObjectEncoder<W>;
    type EncodeMapEntries = JsonObjectEncoder<W>;
    type EncodeStruct = JsonObjectEncoder<W>;
    type EncodeVariant = JsonVariantEncoder<W>;
    type EncodeTupleVariant = JsonArrayEncoder<W>;
    type EncodeStructVariant = JsonObjectEncoder<W>;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be encoded to JSON")
    }

    #[inline]
    fn encode_unit(mut self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(cx, b"null")
    }

    #[inline]
    fn encode_bool(mut self, cx: &C, value: bool) -> Result<Self::Ok, C::Error> {
        self.writer
            .write_bytes(cx, if value { b"true" } else { b"false" })
    }

    #[inline]
    fn encode_char(mut self, cx: &C, value: char) -> Result<Self::Ok, C::Error> {
        encode_string(
            cx,
            self.writer.borrow_mut(),
            value.encode_utf8(&mut [0, 0, 0, 0]).as_bytes(),
        )
    }

    #[inline]
    fn encode_u8(mut self, cx: &C, value: u8) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u16(mut self, cx: &C, value: u16) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u32(mut self, cx: &C, value: u32) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u64(mut self, cx: &C, value: u64) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u128(mut self, cx: &C, value: u128) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i8(mut self, cx: &C, value: i8) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i16(mut self, cx: &C, value: i16) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i32(mut self, cx: &C, value: i32) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i64(mut self, cx: &C, value: i64) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i128(mut self, cx: &C, value: i128) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_usize(mut self, cx: &C, value: usize) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_isize(mut self, cx: &C, value: isize) -> Result<Self::Ok, C::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f32(mut self, cx: &C, value: f32) -> Result<Self::Ok, C::Error> {
        let mut buffer = ryu::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f64(mut self, cx: &C, value: f64) -> Result<Self::Ok, C::Error> {
        let mut buffer = ryu::Buffer::new();
        self.writer.write_bytes(cx, buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_array<const N: usize>(self, cx: &C, bytes: [u8; N]) -> Result<Self::Ok, C::Error> {
        self.encode_bytes(cx, bytes.as_slice())
    }

    #[inline]
    fn encode_bytes(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        let mut buf = itoa::Buffer::new();
        let mut it = bytes.iter();
        let last = it.next_back();

        self.writer.write_byte(cx, b'[')?;

        for b in it {
            self.writer.write_bytes(cx, buf.format(*b).as_bytes())?;
            self.writer.write_byte(cx, b',')?;
        }

        if let Some(b) = last {
            self.writer.write_bytes(cx, buf.format(*b).as_bytes())?;
        }

        self.writer.write_byte(cx, b']')?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(self, cx: &C, bytes: &[&[u8]]) -> Result<Self::Ok, C::Error> {
        let mut seq = JsonArrayEncoder::<_>::new(cx, self.writer)?;

        for bb in bytes {
            for b in *bb {
                seq.push(cx, b)?;
            }
        }

        seq.end(cx)
    }

    #[inline]
    fn encode_string(mut self, cx: &C, string: &str) -> Result<Self::Ok, C::Error> {
        encode_string(cx, self.writer.borrow_mut(), string.as_bytes())
    }

    #[inline]
    fn encode_some(self, _: &C) -> Result<Self::EncodeSome, C::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_none(self, cx: &C) -> Result<Self::Ok, C::Error> {
        self.encode_unit(cx)
    }

    #[inline]
    fn encode_pack(self, cx: &C) -> Result<Self::EncodePack<'_>, C::Error> {
        JsonArrayEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_sequence(self, cx: &C, _: usize) -> Result<Self::EncodeSequence, C::Error> {
        JsonArrayEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_tuple(self, cx: &C, _: usize) -> Result<Self::EncodeTuple, C::Error> {
        JsonArrayEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_map(self, cx: &C, _: usize) -> Result<Self::EncodeMap, C::Error> {
        JsonObjectEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_map_entries(self, cx: &C, _: usize) -> Result<Self::EncodeMapEntries, C::Error> {
        JsonObjectEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_struct(self, cx: &C, _: usize) -> Result<Self::EncodeStruct, C::Error> {
        JsonObjectEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_variant(self, cx: &C) -> Result<Self::EncodeVariant, C::Error> {
        JsonVariantEncoder::new(cx, self.writer)
    }

    #[inline]
    fn encode_tuple_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        _: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer.write_byte(cx, b'{')?;
        tag.encode(cx, JsonObjectKeyEncoder::new(self.writer.borrow_mut()))?;
        self.writer.write_byte(cx, b':')?;
        JsonArrayEncoder::with_end(cx, self.writer, b"]}")
    }

    #[inline]
    fn encode_struct_variant<T>(
        mut self,
        cx: &C,
        tag: &T,
        _: usize,
    ) -> Result<Self::EncodeStructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        self.writer.write_byte(cx, b'{')?;
        tag.encode(cx, JsonObjectKeyEncoder::new(self.writer.borrow_mut()))?;
        self.writer.write_byte(cx, b':')?;
        JsonObjectEncoder::with_end(cx, self.writer, b"}}")
    }
}

/// Encode a sequence of chars as a string.
#[inline]
fn encode_string<C, W>(cx: &C, mut w: W, bytes: &[u8]) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
{
    w.write_byte(cx, b'"')?;

    let mut start = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let escape = ESCAPE[b as usize];

        if escape == 0 {
            continue;
        }

        if start < i {
            w.write_bytes(cx, &bytes[start..i])?;
        }

        write_escape(cx, w.borrow_mut(), escape, b)?;
        start = i + 1;
    }

    if start != bytes.len() {
        w.write_bytes(cx, &bytes[start..])?;
    }

    w.write_byte(cx, b'"')?;
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

fn write_escape<C, W>(cx: &C, mut writer: W, escape: u8, byte: u8) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
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
            return writer.write_bytes(cx, bytes);
        }
        _ => unreachable!(),
    };

    writer.write_bytes(cx, s)
}
