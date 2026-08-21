mod array_encoder;
use self::array_encoder::JsonbArrayEncoder;

mod object_encoder;
use self::object_encoder::JsonbObjectEncoder;

mod object_key_encoder;
use self::object_key_encoder::JsonbObjectKeyEncoder;

mod object_pair_encoder;
use self::object_pair_encoder::JsonbObjectPairEncoder;

mod variant_encoder;
use self::variant_encoder::JsonbVariantEncoder;

use core::fmt;
use core::marker::PhantomData;

use crate::en::{Encode, Encoder};
use crate::hint::{MapHint, SequenceHint};
use crate::writer::BufWriter;
use crate::{Context, Writer};

use super::parse::needs_escape;
use super::tag::{
    ARRAY, FALSE, FLOAT, FLOAT5, INT, NULL, OBJECT, TEXT, TEXTRAW, TRUE, write_header,
};

/// A JSONB encoder for Müsli.
pub(crate) struct JsonbEncoder<W, C, M> {
    cx: C,
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonbEncoder<W, C, M> {
    /// Construct a new JSONB encoder.
    #[inline]
    pub(crate) fn new(cx: C, writer: W) -> Self {
        Self {
            cx,
            writer,
            _marker: PhantomData,
        }
    }
}

/// Write an element which stores its payload as ASCII text.
#[inline]
pub(super) fn encode_text<W, C>(cx: C, writer: &mut W, kind: u8, s: &str) -> Result<(), C::Error>
where
    W: ?Sized + Writer,
    C: Context,
{
    write_header(cx, writer, kind, s.len())?;
    writer.write_bytes(cx, s.as_bytes())
}

/// Write a string, picking the element type based on whether the string would
/// have to be escaped to be rendered as JSON.
///
/// Neither element type stores delimiters or escapes, so the payload is the
/// string itself in both cases.
#[inline]
pub(super) fn encode_string<W, C>(cx: C, writer: &mut W, string: &str) -> Result<(), C::Error>
where
    W: ?Sized + Writer,
    C: Context,
{
    let kind = if needs_escape(string.as_bytes()) {
        TEXTRAW
    } else {
        TEXT
    };

    encode_text(cx, writer, kind, string)
}

macro_rules! encode_integer {
    ($slf:ident, $value:ident) => {{
        let mut buffer = itoa::Buffer::new();
        encode_text($slf.cx, &mut $slf.writer, INT, buffer.format($value))
    }};
}

macro_rules! encode_float {
    ($slf:ident, $value:ident) => {{
        if $value.is_finite() {
            let mut buffer = ryu::Buffer::new();
            encode_text(
                $slf.cx,
                &mut $slf.writer,
                FLOAT,
                buffer.format_finite($value),
            )
        } else if $value.is_nan() {
            // NaN has no JSON representation at all. SQLite turns it into
            // `null`, which cannot be decoded back into a float, so the JSON5
            // spelling is used instead.
            encode_text($slf.cx, &mut $slf.writer, FLOAT5, "NaN")
        } else {
            // The infinities are spelled the way SQLite spells them, as the
            // overflowing exponent `9e999`, which is canonical JSON and parses
            // back to an infinity.
            let s = if $value.is_sign_negative() {
                "-9e999"
            } else {
                "9e999"
            };

            encode_text($slf.cx, &mut $slf.writer, FLOAT, s)
        }
    }};
}

#[crate::trait_defaults(crate)]
impl<W, C, M> Encoder for JsonbEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodePack = JsonbArrayEncoder<W, C, M>;
    type EncodeSome = Self;
    type EncodeSequence = JsonbArrayEncoder<W, C, M>;
    type EncodeMap = JsonbObjectEncoder<W, C, M>;
    type EncodeMapEntries = JsonbObjectEncoder<W, C, M>;
    type EncodeVariant = JsonbVariantEncoder<W, C, M>;
    type EncodeSequenceVariant = JsonbArrayEncoder<W, C, M>;
    type EncodeMapVariant = JsonbObjectEncoder<W, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be encoded to JSONB")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.as_encode().encode(self)
    }

    #[inline]
    fn encode_empty(mut self) -> Result<(), Self::Error> {
        write_header(self.cx, &mut self.writer, NULL, 0)
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<(), Self::Error> {
        write_header(
            self.cx,
            &mut self.writer,
            if value { TRUE } else { FALSE },
            0,
        )
    }

    #[inline]
    fn encode_char(mut self, value: char) -> Result<(), Self::Error> {
        encode_string(
            self.cx,
            &mut self.writer,
            value.encode_utf8(&mut [0, 0, 0, 0]),
        )
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_i8(mut self, value: i8) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_isize(mut self, value: isize) -> Result<(), Self::Error> {
        encode_integer!(self, value)
    }

    #[inline]
    fn encode_f32(mut self, value: f32) -> Result<(), Self::Error> {
        encode_float!(self, value)
    }

    #[inline]
    fn encode_f64(mut self, value: f64) -> Result<(), Self::Error> {
        encode_float!(self, value)
    }

    #[inline]
    fn encode_array<const N: usize>(self, bytes: &[u8; N]) -> Result<(), Self::Error> {
        self.encode_bytes(bytes)
    }

    #[inline]
    fn encode_bytes(self, bytes: &[u8]) -> Result<(), Self::Error> {
        // Just like the JSON encoder, byte strings are encoded as arrays of
        // numbers since JSON has no byte string type.
        let mut buffer = BufWriter::new(self.cx.alloc());
        let mut itoa = itoa::Buffer::new();

        for &b in bytes {
            encode_text(self.cx, &mut buffer, INT, itoa.format(b))?;
        }

        finish_container(self.cx, self.writer, None, buffer, ARRAY)
    }

    #[inline]
    fn encode_bytes_vectored<I>(self, _: usize, vectors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item: AsRef<[u8]>>,
    {
        let mut buffer = BufWriter::new(self.cx.alloc());
        let mut itoa = itoa::Buffer::new();

        for bb in vectors {
            for &b in bb.as_ref() {
                encode_text(self.cx, &mut buffer, INT, itoa.format(b))?;
            }
        }

        finish_container(self.cx, self.writer, None, buffer, ARRAY)
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<(), Self::Error> {
        encode_string(self.cx, &mut self.writer, string)
    }

    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        self.encode_empty()
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Ok(JsonbArrayEncoder::new(self.cx, self.writer))
    }

    #[inline]
    fn encode_sequence(self, _: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        Ok(JsonbArrayEncoder::new(self.cx, self.writer))
    }

    #[inline]
    fn encode_map(self, _: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        Ok(JsonbObjectEncoder::new(self.cx, self.writer))
    }

    #[inline]
    fn encode_map_entries(self, _: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        Ok(JsonbObjectEncoder::new(self.cx, self.writer))
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        Ok(JsonbVariantEncoder::new(self.cx, self.writer))
    }

    #[inline]
    fn encode_sequence_variant<T>(
        self,
        tag: &T,
        _: impl SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        let object = encode_variant_tag::<_, _, M>(self.cx, tag)?;
        Ok(JsonbArrayEncoder::with_variant(
            self.cx,
            self.writer,
            object,
        ))
    }

    #[inline]
    fn encode_map_variant<T>(
        self,
        tag: &T,
        _: impl MapHint,
    ) -> Result<Self::EncodeMapVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        let object = encode_variant_tag::<_, _, M>(self.cx, tag)?;
        Ok(JsonbObjectEncoder::with_variant(
            self.cx,
            self.writer,
            object,
        ))
    }
}

/// Start the payload of the single entry object which an externally tagged
/// variant is encoded as by writing its key.
#[inline]
fn encode_variant_tag<T, C, M>(cx: C, tag: &T) -> Result<BufWriter<C::Allocator>, C::Error>
where
    T: ?Sized + Encode<M>,
    C: Context,
    M: 'static,
{
    let mut object = BufWriter::new(cx.alloc());
    JsonbObjectKeyEncoder::<_, _, M>::new(cx, &mut object).encode(tag)?;
    Ok(object)
}

/// Write a buffered container payload prefixed by its header.
///
/// If `variant` is set it holds the already written key of a single entry
/// object which the container is the value of, so the container is wrapped in
/// that object. This is how an externally tagged variant is encoded.
#[inline]
pub(super) fn finish_container<W, C>(
    cx: C,
    mut writer: W,
    variant: Option<BufWriter<C::Allocator>>,
    buffer: BufWriter<C::Allocator>,
    kind: u8,
) -> Result<(), C::Error>
where
    W: Writer,
    C: Context,
{
    let payload = buffer.into_inner();

    let Some(mut object) = variant else {
        write_header(cx, &mut writer, kind, payload.len())?;
        return writer.extend(cx, payload);
    };

    write_header(cx, &mut object, kind, payload.len())?;
    object.extend(cx, payload)?;

    let object = object.into_inner();
    write_header(cx, &mut writer, OBJECT, object.len())?;
    writer.extend(cx, object)
}
