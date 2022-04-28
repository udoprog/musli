use core::fmt;

use musli::en::{Encoder, PairEncoder, PairsEncoder};
use musli::never::Never;
use musli_binary_common::buffered_writer::BufferedWriter;
use musli_binary_common::writer::Writer;

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
    type Sequence = Never<Self>;
    type Tuple = Never<Self>;
    type Map = Never<Self>;
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
    fn encode_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_bytes(b"null")
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, Self::Error> {
        encode_chars(&mut self.writer, string.chars())
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_bytes(if value { b"true" } else { b"false" })
    }

    #[inline]
    fn encode_char(mut self, value: char) -> Result<Self::Ok, Self::Error> {
        encode_chars(&mut self.writer, [value])
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
    fn encode_string(self, string: &str) -> Result<Self::Ok, Self::Error> {
        encode_chars(self.writer, string.chars())
    }
}

/// Encode a sequence of chars as a string.
#[inline]
fn encode_chars<W, I>(writer: W, it: I) -> Result<(), W::Error>
where
    W: Writer,
    I: IntoIterator<Item = char>,
{
    let mut writer = BufferedWriter::<64, _>::new(writer);

    writer.write_byte(b'"')?;

    for c in it {
        match c {
            '"' => {
                writer.write_bytes(b"\\\"")?;
            }
            '\\' => {
                writer.write_bytes(b"\\\\")?;
            }
            // Specially encoded control characters.
            '\u{0008}' => {
                writer.write_bytes(b"\\b")?;
            }
            '\u{000C}' => {
                writer.write_bytes(b"\\f")?;
            }
            '\n' => {
                writer.write_bytes(b"\\n")?;
            }
            '\r' => {
                writer.write_bytes(b"\\r")?;
            }
            '\t' => {
                writer.write_bytes(b"\\t")?;
            }
            c => {
                if c.is_control() {
                    todo!("control characters are not supported yet")
                } else {
                    let mut bytes = [0, 0, 0, 0];
                    let bytes = c.encode_utf8(bytes.as_mut_slice());
                    writer.write_bytes(bytes.as_bytes())?;
                }
            }
        }
    }

    writer.write_byte(b'"')?;
    writer.finish()
}
