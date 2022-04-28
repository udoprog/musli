use core::fmt;

use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder};
use musli::never::Never;
use musli_common::writer::Writer;

/// A JSON encoder for Müsli.
pub struct JsonEncoder<W> {
    writer: W,
}

impl<W> JsonEncoder<W> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> Encoder for JsonEncoder<W>
where
    W: Writer,
{
    type Error = W::Error;
    type Ok = ();
    type Pack = Never<Self>;
    type Some = Never<Self>;
    type Sequence = JsonArrayEncoder<W>;
    type Tuple = Never<Self>;
    type Map = JsonObjectEncoder<W>;
    type Struct = JsonObjectEncoder<W>;
    type TupleStruct = Never<Self>;
    type StructVariant = Never<Self>;
    type TupleVariant = Never<Self>;
    type UnitVariant = Never<Self>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be encoded to JSON")
    }

    #[inline]
    fn encode_sequence(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        JsonArrayEncoder::new(self.writer)
    }

    #[inline]
    fn encode_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_bytes(b"null")
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, Self::Error> {
        encode_string(&mut self.writer, string.as_bytes())
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_bytes(if value { b"true" } else { b"false" })
    }

    #[inline]
    fn encode_char(mut self, value: char) -> Result<Self::Ok, Self::Error> {
        encode_string(
            &mut self.writer,
            value.encode_utf8(&mut [0, 0, 0, 0]).as_bytes(),
        )
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f32(mut self, value: f32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = ryu::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_f64(mut self, value: f64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = ryu::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())
    }

    #[inline]
    fn encode_map(self, _: usize) -> Result<Self::Map, Self::Error> {
        JsonObjectEncoder::new(self.writer)
    }

    #[inline]
    fn encode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        JsonObjectEncoder::new(self.writer)
    }
}

/// Encoder for a pairs sequence.
pub struct JsonObjectEncoder<W> {
    len: usize,
    writer: W,
}

impl<W> JsonObjectEncoder<W>
where
    W: Writer,
{
    #[inline]
    fn new(mut writer: W) -> Result<Self, W::Error> {
        writer.write_byte(b'{')?;

        Ok(Self { len: 0, writer })
    }
}

impl<'a, W> PairsEncoder for JsonObjectEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;

    type Encoder<'this> = JsonObjectPairEncoder<&'this mut W>
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        let len = self.len;
        self.len += 1;

        Ok(JsonObjectPairEncoder {
            empty: len == 0,
            writer: &mut self.writer,
        })
    }

    #[inline]
    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(b'}')?;
        Ok(())
    }
}

/// Encoder for a pair.
pub struct JsonObjectPairEncoder<W> {
    empty: bool,
    writer: W,
}

impl<'a, W> PairEncoder for JsonObjectPairEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;

    type First<'this> = JsonObjectKeyEncoder<&'this mut W>
    where
        Self: 'this;

    type Second<'this> = JsonEncoder<&'this mut W>
    where
        Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        if !self.empty {
            self.writer.write_byte(b',')?;
        }

        Ok(JsonObjectKeyEncoder::new(&mut self.writer))
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        self.writer.write_byte(b':')?;
        Ok(JsonEncoder::new(&mut self.writer))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
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

impl<W> Encoder for JsonObjectKeyEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;
    type Pack = Never<Self>;
    type Some = Never<Self>;
    type Sequence = Never<Self>;
    type Tuple = Never<Self>;
    type Map = Never<Self>;
    type Struct = Never<Self>;
    type TupleStruct = Never<Self>;
    type StructVariant = Never<Self>;
    type TupleVariant = Never<Self>;
    type UnitVariant = Never<Self>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any type that can be used as an object key")
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(b'"')?;
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())?;
        self.writer.write_byte(b'"')?;
        Ok(())
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(b'"')?;
        let mut buffer = itoa::Buffer::new();
        self.writer.write_bytes(buffer.format(value).as_bytes())?;
        self.writer.write_byte(b'"')?;
        Ok(())
    }

    #[inline]
    fn encode_string(self, string: &str) -> Result<Self::Ok, Self::Error> {
        encode_string(self.writer, string.as_bytes())
    }
}

/// Encoder for a pairs sequence.
pub struct JsonArrayEncoder<W> {
    first: bool,
    writer: W,
}

impl<W> JsonArrayEncoder<W>
where
    W: Writer,
{
    #[inline]
    fn new(mut writer: W) -> Result<Self, W::Error> {
        writer.write_byte(b'[')?;
        Ok(Self {
            first: true,
            writer,
        })
    }
}

impl<W> SequenceEncoder for JsonArrayEncoder<W>
where
    W: Writer,
{
    type Ok = ();
    type Error = W::Error;

    type Encoder<'this> = JsonEncoder<&'this mut W>
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        if !self.first {
            self.writer.write_byte(b',')?;
        }

        Ok(JsonEncoder::new(&mut self.writer))
    }

    #[inline]
    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(b']')?;
        Ok(())
    }
}

/// Encode a sequence of chars as a string.
#[inline]
fn encode_string<W>(mut writer: W, bytes: &[u8]) -> Result<(), W::Error>
where
    W: Writer,
{
    writer.write_byte(b'"')?;

    let mut start = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];

        if escape == 0 {
            continue;
        }

        if start < i {
            writer.write_bytes(&bytes[start..1])?;
        }

        write_escape(&mut writer, escape, byte)?;
        start = i + 1;
    }

    if start != bytes.len() {
        writer.write_bytes(&bytes[start..])?;
    }

    writer.write_byte(b'"')?;
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

fn write_escape<W>(writer: &mut W, escape: u8, byte: u8) -> Result<(), W::Error>
where
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
            return writer.write_bytes(bytes);
        }
        _ => unreachable!(),
    };

    writer.write_bytes(s)
}
