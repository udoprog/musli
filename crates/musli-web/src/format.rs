//! Runtime dispatch over the [`Format`]s which message bodies can be encoded
//! with.
//!
//! Dispatch has to happen at runtime rather than through generics, since a
//! server adapts to whichever format a client asks for. See the [wire format]
//! for details.
//!
//! [wire format]: crate::api#wire-format

use core::fmt::{self, Write};
use core::str;

use alloc::vec::Vec;

use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::{Decode, Encode};

use crate::api::{ChannelId, DecodeBody, EncodeBody, Format, Mode, RequestHeader, ResponseHeader};

/// An envelope which can also be spelled out as an http-like block of
/// `<key>: <value>` lines, which is what [`Mode::Text`] uses.
///
/// See the [text envelope] for the shape this produces.
///
/// [text envelope]: crate::api#the-text-envelope
// NB: Only the modules which speak the protocol have any use for envelopes.
#[cfg_attr(
    not(any(feature = "ws", feature = "client", feature = "web03")),
    allow(dead_code)
)]
pub(crate) trait TextEnvelope
where
    Self: Sized,
{
    /// The envelope with every field left at zero, which is what an absent
    /// field reads back as.
    fn empty() -> Self;

    /// Write every field of the envelope as a `<key>: <value>` line.
    fn write_text(&self, out: &mut dyn Write) -> fmt::Result;

    /// Apply a single field read out of a text envelope.
    ///
    /// A key which is not recognized is ignored so that a peer built against a
    /// newer version of the protocol can still be understood.
    fn set_text(&mut self, key: &str, value: &str) -> Result<(), Error>;
}

/// Write a single `<key>: <value>` line.
#[inline]
fn write_field(out: &mut dyn Write, key: &str, value: impl fmt::Display) -> fmt::Result {
    writeln!(out, "{key}: {value}")
}

/// Parse the value of a text envelope field.
#[inline]
fn parse_field<T>(key: &'static str, value: &str) -> Result<T, Error>
where
    T: core::str::FromStr,
{
    match value.parse() {
        Ok(value) => Ok(value),
        Err(..) => Err(Error::new(ErrorKind::TextField { key })),
    }
}

impl TextEnvelope for RequestHeader {
    #[inline]
    fn empty() -> Self {
        Self {
            serial: 0,
            id: 0,
            format: 0,
            channel: ChannelId::NONE,
        }
    }

    #[inline]
    fn write_text(&self, out: &mut dyn Write) -> fmt::Result {
        write_field(out, "serial", self.serial)?;
        write_field(out, "id", self.id)?;
        write_field(out, "format", self.format)?;
        write_field(out, "channel", self.channel.raw())?;
        Ok(())
    }

    #[inline]
    fn set_text(&mut self, key: &str, value: &str) -> Result<(), Error> {
        match key {
            "serial" => self.serial = parse_field("serial", value)?,
            "id" => self.id = parse_field("id", value)?,
            "format" => self.format = parse_field("format", value)?,
            "channel" => self.channel = ChannelId::from_u16(parse_field("channel", value)?),
            _ => {}
        }

        Ok(())
    }
}

impl TextEnvelope for ResponseHeader {
    #[inline]
    fn empty() -> Self {
        Self {
            serial: 0,
            broadcast: 0,
            error: 0,
            format: 0,
            channel: ChannelId::NONE,
        }
    }

    #[inline]
    fn write_text(&self, out: &mut dyn Write) -> fmt::Result {
        write_field(out, "serial", self.serial)?;
        write_field(out, "broadcast", self.broadcast)?;
        write_field(out, "error", self.error)?;
        write_field(out, "format", self.format)?;
        write_field(out, "channel", self.channel.raw())?;
        Ok(())
    }

    #[inline]
    fn set_text(&mut self, key: &str, value: &str) -> Result<(), Error> {
        match key {
            "serial" => self.serial = parse_field("serial", value)?,
            "broadcast" => self.broadcast = parse_field("broadcast", value)?,
            "error" => self.error = parse_field("error", value)?,
            "format" => self.format = parse_field("format", value)?,
            "channel" => self.channel = ChannelId::from_u16(parse_field("channel", value)?),
            _ => {}
        }

        Ok(())
    }
}

/// Adapter which lets the text envelope be written straight into the outgoing
/// buffer without going through an intermediate [`String`].
///
/// [`String`]: alloc::string::String
struct Utf8Writer<'a> {
    out: &'a mut Vec<u8>,
}

impl Write for Utf8Writer<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.out.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

/// Encode the fixed envelope of a message.
///
/// The envelope never depends on which format has been negotiated for bodies,
/// which is what allows both peers to read it unconditionally. `mode` decides
/// how it is spelled, see the [wire format].
///
/// [wire format]: crate::api#wire-format
// NB: Only the modules which speak the protocol have any use for envelopes.
#[cfg_attr(
    not(any(feature = "ws", feature = "client", feature = "web03")),
    allow(dead_code)
)]
#[inline]
pub(crate) fn encode_envelope<T>(mode: Mode, out: &mut Vec<u8>, value: &T) -> Result<(), Error>
where
    T: Encode<Binary> + TextEnvelope,
{
    match mode {
        Mode::Binary => {
            musli::packed::encode(out, value).map_err(Error::packed)?;
        }
        Mode::Text => {
            let mut writer = Utf8Writer { out };

            if value.write_text(&mut writer).is_err() {
                return Err(Error::new(ErrorKind::TextWrite));
            }

            // NB: The empty line is what separates the envelope from the body.
            out.push(b'\n');
        }
    }

    Ok(())
}

/// Decode the fixed envelope of a message, advancing `at` past it.
#[cfg_attr(
    not(any(feature = "ws", feature = "client", feature = "web03")),
    allow(dead_code)
)]
#[inline]
pub(crate) fn decode_envelope<'de, T>(
    mode: Mode,
    buf: &'de [u8],
    at: &mut usize,
) -> Result<T, Error>
where
    T: Decode<'de, Binary, Global> + TextEnvelope,
{
    let Some(tail) = buf.get(*at..) else {
        return Err(Error::new(ErrorKind::Overflow {
            at: *at,
            len: buf.len(),
        }));
    };

    match mode {
        Mode::Binary => {
            let mut reader = SliceReader::new(tail);
            let value = musli::packed::decode(&mut reader).map_err(Error::packed)?;
            *at += tail.len() - reader.remaining();
            Ok(value)
        }
        Mode::Text => {
            let Ok(text) = str::from_utf8(tail) else {
                return Err(Error::new(ErrorKind::TextNotUtf8));
            };

            let mut header = T::empty();
            let mut rest = text;

            loop {
                let Some((line, tail)) = rest.split_once('\n') else {
                    return Err(Error::new(ErrorKind::TextUnterminated));
                };

                rest = tail;

                // NB: Tolerated so that an envelope which has been through
                // something that insists on CRLF still reads.
                let line = line.strip_suffix('\r').unwrap_or(line);

                if line.is_empty() {
                    break;
                }

                let Some((key, value)) = line.split_once(':') else {
                    return Err(Error::new(ErrorKind::TextSeparator));
                };

                header.set_text(key.trim(), value.trim())?;
            }

            *at += tail.len() - rest.len();
            Ok(header)
        }
    }
}

/// Encode `value` with the given format, appending it to `out`.
macro_rules! encode_with {
    ($module:ident, $out:expr, $value:expr, $variant:ident) => {{
        musli::$module::encode($out, $value)
            .map(|_| ())
            .map_err(Error::$variant)
    }};
}

/// Decode a value with the given format from `buf` at `at`, advancing `at` past
/// what was consumed.
macro_rules! decode_with {
    ($module:ident, $tail:expr, $at:expr, $len:expr, $variant:ident) => {{
        let mut reader = SliceReader::new($tail);
        let value = musli::$module::decode(&mut reader).map_err(Error::$variant)?;
        *$at += $tail.len() - reader.remaining();
        let _ = $len;
        Ok(value)
    }};
}

impl Format {
    /// Test if this build of the crate has support for the format.
    ///
    /// Formats are gated behind features, so a peer might genuinely be unable
    /// to speak a format which the other side asks for. This is what the
    /// [negotiation protocol] uses to decide whether a request can be honored.
    ///
    /// [negotiation protocol]: crate::api#negotiating-the-format
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// // The default format is always available.
    /// assert!(Format::DEFAULT.is_supported());
    /// ```
    #[inline]
    pub const fn is_supported(self) -> bool {
        match self {
            Format::Packed => cfg!(feature = "format-packed"),
            Format::Storage => cfg!(feature = "format-storage"),
            Format::Wire => cfg!(feature = "format-wire"),
            Format::Descriptive => cfg!(feature = "format-descriptive"),
            Format::Json => cfg!(feature = "format-json"),
        }
    }

    /// Iterate over every format this build of the crate supports.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert!(Format::supported().any(|f| f == Format::DEFAULT));
    /// ```
    #[inline]
    pub fn supported() -> impl Iterator<Item = Format> {
        Format::ALL.iter().copied().filter(|f| f.is_supported())
    }

    /// Encode `value` with this format, appending it to `out`.
    #[cfg_attr(
        not(any(feature = "ws", feature = "client", feature = "web03")),
        allow(dead_code)
    )]
    pub(crate) fn encode<T>(self, out: &mut Vec<u8>, value: &T) -> Result<(), Error>
    where
        T: ?Sized + EncodeBody,
    {
        match self {
            #[cfg(feature = "format-packed")]
            Format::Packed => encode_with!(packed, out, value, packed),
            #[cfg(feature = "format-storage")]
            Format::Storage => encode_with!(storage, out, value, storage),
            #[cfg(feature = "format-wire")]
            Format::Wire => encode_with!(wire, out, value, wire),
            #[cfg(feature = "format-descriptive")]
            Format::Descriptive => encode_with!(descriptive, out, value, descriptive),
            #[cfg(feature = "format-json")]
            Format::Json => musli::json::encode(out, value)
                .map(|_| ())
                .map_err(Error::json),
            #[allow(unreachable_patterns)]
            _ => Err(Error::unsupported(self)),
        }
    }

    /// Decode a value with this format from `buf` starting at `at`, advancing
    /// `at` past what was consumed.
    ///
    /// Advancing `at` is what allows several payloads to be decoded in sequence
    /// out of a single message.
    #[cfg_attr(
        not(any(feature = "ws", feature = "client", feature = "web03")),
        allow(dead_code)
    )]
    pub(crate) fn decode<'de, T>(self, buf: &'de [u8], at: &mut usize) -> Result<T, Error>
    where
        T: DecodeBody<'de>,
    {
        let Some(tail) = buf.get(*at..) else {
            return Err(Error::new(ErrorKind::Overflow {
                at: *at,
                len: buf.len(),
            }));
        };

        match self {
            #[cfg(feature = "format-packed")]
            Format::Packed => decode_with!(packed, tail, at, buf.len(), packed),
            #[cfg(feature = "format-storage")]
            Format::Storage => decode_with!(storage, tail, at, buf.len(), storage),
            #[cfg(feature = "format-wire")]
            Format::Wire => decode_with!(wire, tail, at, buf.len(), wire),
            #[cfg(feature = "format-descriptive")]
            Format::Descriptive => decode_with!(descriptive, tail, at, buf.len(), descriptive),
            #[cfg(feature = "format-json")]
            Format::Json => {
                // NB: The borrow here is load-bearing. A `&mut &[u8]` selects
                // the parser which keeps the referenced slice up to date as it
                // parses, which is how the consumed length is recovered for a
                // format that is not read through a `Reader`. Passing the slice
                // by value selects a parser which does not write back, and the
                // position would silently never advance.
                let mut rest = tail;
                let cursor = &mut rest;
                let value = musli::json::decode(cursor).map_err(Error::json)?;
                *at += tail.len() - rest.len();
                Ok(value)
            }
            #[allow(unreachable_patterns)]
            _ => Err(Error::unsupported(self)),
        }
    }
}

/// An error raised when encoding or decoding a message.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[inline]
    const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// Construct an error indicating that the given format is not supported by
    /// this build of the crate.
    #[inline]
    pub(crate) const fn unsupported(format: Format) -> Self {
        Self::new(ErrorKind::Unsupported(format))
    }

    /// Test if the error is caused by a format which is not supported, and if
    /// so return it.
    #[inline]
    pub fn unsupported_format(&self) -> Option<Format> {
        match self.kind {
            ErrorKind::Unsupported(format) => Some(format),
            _ => None,
        }
    }
}

macro_rules! error_kinds {
    ($($(#[$meta:meta])* $variant:ident, $ctor:ident, $ty:path;)*) => {
        #[derive(Debug)]
        enum ErrorKind {
            Unsupported(Format),
            Overflow { at: usize, len: usize },
            TextWrite,
            TextNotUtf8,
            TextUnterminated,
            TextSeparator,
            TextField { key: &'static str },
            $($(#[$meta])* $variant($ty),)*
        }

        impl Error {
            $(
                $(#[$meta])*
                #[inline]
                fn $ctor(error: $ty) -> Self {
                    Self::new(ErrorKind::$variant(error))
                }
            )*
        }

        impl fmt::Display for Error {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match &self.kind {
                    ErrorKind::Unsupported(format) => {
                        write!(f, "Format `{format}` is not supported")
                    }
                    ErrorKind::Overflow { at, len } => {
                        write!(f, "Offset {at} is out of bounds for a message of {len} bytes")
                    }
                    ErrorKind::TextWrite => {
                        write!(f, "Failed to write text envelope")
                    }
                    ErrorKind::TextNotUtf8 => {
                        write!(f, "Text envelope is not valid UTF-8")
                    }
                    ErrorKind::TextUnterminated => {
                        write!(f, "Text envelope is missing its terminating empty line")
                    }
                    ErrorKind::TextSeparator => {
                        write!(f, "Text envelope has a line without a `:` separator")
                    }
                    ErrorKind::TextField { key } => {
                        write!(f, "Text envelope field `{key}` is not a valid number")
                    }
                    $($(#[$meta])* ErrorKind::$variant(..) => {
                        write!(f, concat!("Error in the `", stringify!($ctor), "` format"))
                    })*
                }
            }
        }

        impl core::error::Error for Error {
            #[inline]
            fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                match &self.kind {
                    $($(#[$meta])* ErrorKind::$variant(error) => Some(error),)*
                    _ => None,
                }
            }
        }
    };
}

error_kinds! {
    // NB: Always available, since it is what the envelope is encoded with.
    Packed, packed, musli::packed::Error;
    #[cfg(feature = "format-storage")]
    Storage, storage, musli::storage::Error;
    #[cfg(feature = "format-wire")]
    Wire, wire, musli::wire::Error;
    #[cfg(feature = "format-descriptive")]
    Descriptive, descriptive, musli::descriptive::Error;
    #[cfg(feature = "format-json")]
    Json, json, musli::json::Error;
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;

    use musli::{Decode, Encode};

    use crate::api::{ChannelId, Format, Mode, RequestHeader, ResponseHeader};

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct Message<'de> {
        message: &'de str,
        tick: u32,
    }

    /// Every supported format must round-trip a value.
    #[test]
    fn round_trip() {
        for format in Format::supported() {
            let mut buf = Vec::new();
            let expected = Message {
                message: "hello",
                tick: 42,
            };

            format.encode(&mut buf, &expected).unwrap();

            let mut at = 0;
            let actual: Message<'_> = format.decode(&buf, &mut at).unwrap();

            assert_eq!(actual, expected, "round trip failed for `{format}`");
            assert_eq!(at, buf.len(), "`{format}` did not consume the whole body");
        }
    }

    /// Several payloads must be decodable in sequence out of one buffer, which
    /// is what `RawPacket::decode` relies on.
    #[test]
    fn sequential_payloads() {
        for format in Format::supported() {
            let mut buf = Vec::new();

            let first = Message {
                message: "first",
                tick: 1,
            };

            let second = Message {
                message: "second",
                tick: 2,
            };

            format.encode(&mut buf, &first).unwrap();
            let boundary = buf.len();
            format.encode(&mut buf, &second).unwrap();

            let mut at = 0;
            let a: Message<'_> = format.decode(&buf, &mut at).unwrap();
            assert_eq!(a, first, "first payload failed for `{format}`");
            assert_eq!(at, boundary, "`{format}` misreported the first boundary");

            let b: Message<'_> = format.decode(&buf, &mut at).unwrap();
            assert_eq!(b, second, "second payload failed for `{format}`");
            assert_eq!(at, buf.len(), "`{format}` did not consume both payloads");
        }
    }

    /// A body must follow the fixed envelope in the same frame, for every
    /// format, since that is the shape of every message on the wire.
    #[test]
    fn envelope_then_body() {
        for format in Format::supported() {
            let header = RequestHeader {
                serial: 7,
                id: 11,
                format: format.to_u8(),
                channel: ChannelId::from_u16(3),
            };

            let mut buf = Vec::new();
            encode_envelope(Mode::DEFAULT, &mut buf, &header).unwrap();

            let expected = Message {
                message: "body",
                tick: 9,
            };

            format.encode(&mut buf, &expected).unwrap();

            let mut at = 0;
            let decoded: RequestHeader = decode_envelope(Mode::DEFAULT, &buf, &mut at).unwrap();

            assert_eq!(decoded.serial, 7);
            assert_eq!(decoded.id, 11);
            assert_eq!(decoded.format, format.to_u8());

            let body: Message<'_> = format.decode(&buf, &mut at).unwrap();
            assert_eq!(body, expected, "body failed for `{format}`");
            assert_eq!(at, buf.len());
        }
    }

    /// JSON must be keyed by field name, which is the point of offering it.
    #[test]
    #[cfg(feature = "format-json")]
    fn json_is_human_readable() {
        let mut buf = Vec::new();

        Format::Json
            .encode(
                &mut buf,
                &Message {
                    message: "hello",
                    tick: 42,
                },
            )
            .unwrap();

        assert_eq!(
            core::str::from_utf8(&buf).unwrap(),
            r#"{"message":"hello","tick":42}"#
        );
    }

    /// The text envelope is what makes a frame readable by hand, so its exact
    /// shape is part of the protocol.
    #[test]
    fn text_envelope_is_http_like() {
        let mut buf = Vec::new();

        let header = RequestHeader {
            serial: 7,
            id: 11,
            format: Format::Json.to_u8(),
            channel: ChannelId::from_u16(3),
        };

        encode_envelope(Mode::Text, &mut buf, &header).unwrap();

        assert_eq!(
            core::str::from_utf8(&buf).unwrap(),
            "serial: 7\nid: 11\nformat: 5\nchannel: 3\n\n"
        );

        let mut buf = Vec::new();

        let header = ResponseHeader {
            serial: 0,
            broadcast: 13,
            error: 0,
            format: Format::Json.to_u8(),
            channel: ChannelId::NONE,
        };

        encode_envelope(Mode::Text, &mut buf, &header).unwrap();

        assert_eq!(
            core::str::from_utf8(&buf).unwrap(),
            "serial: 0\nbroadcast: 13\nerror: 0\nformat: 5\nchannel: 0\n\n"
        );
    }

    /// Both envelopes must survive a round trip in either mode, and report
    /// exactly how much of the buffer they consumed.
    #[test]
    fn text_envelope_round_trip() {
        for mode in Mode::ALL.iter().copied() {
            let mut buf = Vec::new();

            let request = RequestHeader {
                serial: 4294967295,
                id: 65535,
                format: Format::Json.to_u8(),
                channel: ChannelId::from_u16(65535),
            };

            encode_envelope(mode, &mut buf, &request).unwrap();

            let mut at = 0;
            let decoded: RequestHeader = decode_envelope(mode, &buf, &mut at).unwrap();

            assert_eq!(decoded.serial, request.serial, "`{mode}` lost the serial");
            assert_eq!(decoded.id, request.id, "`{mode}` lost the id");
            assert_eq!(decoded.format, request.format, "`{mode}` lost the format");
            assert_eq!(
                decoded.channel, request.channel,
                "`{mode}` lost the channel"
            );
            assert_eq!(at, buf.len(), "`{mode}` did not consume the whole envelope");

            let mut buf = Vec::new();

            let response = ResponseHeader {
                serial: 9,
                broadcast: 13,
                error: 17,
                format: Format::Json.to_u8(),
                channel: ChannelId::from_u16(21),
            };

            encode_envelope(mode, &mut buf, &response).unwrap();

            let mut at = 0;
            let decoded: ResponseHeader = decode_envelope(mode, &buf, &mut at).unwrap();

            assert_eq!(decoded.serial, response.serial, "`{mode}` lost the serial");
            assert_eq!(
                decoded.broadcast, response.broadcast,
                "`{mode}` lost the broadcast"
            );
            assert_eq!(decoded.error, response.error, "`{mode}` lost the error");
            assert_eq!(decoded.format, response.format, "`{mode}` lost the format");
            assert_eq!(
                decoded.channel, response.channel,
                "`{mode}` lost the channel"
            );
            assert_eq!(at, buf.len(), "`{mode}` did not consume the whole envelope");
        }
    }

    /// A text envelope has to be followed by a body in the same frame, which is
    /// what the empty line separates it from.
    #[test]
    #[cfg(feature = "format-json")]
    fn text_envelope_then_body() {
        let mut buf = Vec::new();

        let header = RequestHeader {
            serial: 1,
            id: 2,
            format: Format::Json.to_u8(),
            channel: ChannelId::NONE,
        };

        encode_envelope(Mode::Text, &mut buf, &header).unwrap();

        let expected = Message {
            message: "hello",
            tick: 42,
        };

        Format::Json.encode(&mut buf, &expected).unwrap();

        assert_eq!(
            core::str::from_utf8(&buf).unwrap(),
            "serial: 1\nid: 2\nformat: 5\nchannel: 0\n\n{\"message\":\"hello\",\"tick\":42}"
        );

        let mut at = 0;
        let decoded: RequestHeader = decode_envelope(Mode::Text, &buf, &mut at).unwrap();
        assert_eq!(decoded.id, 2);

        let body: Message<'_> = Format::Json.decode(&buf, &mut at).unwrap();
        assert_eq!(body, expected);
        assert_eq!(at, buf.len());
    }

    /// Reading has to be lenient about what it accepts, so that a peer built
    /// against a different version of the protocol is still understood.
    #[test]
    fn text_envelope_is_lenient() {
        // Fields in a different order, a field which is not known, a field
        // which is absent, and CRLF line endings throughout.
        let buf = b"channel: 3\r\nfuture: whatever\r\nid: 11\r\n\r\n";

        let mut at = 0;
        let header: RequestHeader = decode_envelope(Mode::Text, buf, &mut at).unwrap();

        assert_eq!(header.id, 11);
        assert_eq!(header.channel, ChannelId::from_u16(3));
        assert_eq!(header.serial, 0, "an absent field reads as zero");
        assert_eq!(header.format, 0, "an absent field reads as zero");
        assert_eq!(at, buf.len());
    }

    /// A malformed text envelope has to be reported rather than silently read
    /// as something else.
    #[test]
    fn text_envelope_rejects_malformed_input() {
        // No empty line, so the envelope never ends.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Text, b"id: 11\n", &mut at).is_err());

        // A line which is not a field.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Text, b"id 11\n\n", &mut at).is_err());

        // A field whose value is not a number.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Text, b"id: none\n\n", &mut at).is_err());

        // Not text at all.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Text, &[0xff, 0xfe], &mut at).is_err());
    }

    /// Only a human readable format can be carried in a text frame, since the
    /// whole frame has to be valid UTF-8.
    #[test]
    fn text_mode_only_accepts_human_readable_formats() {
        for format in Format::ALL.iter().copied() {
            assert!(
                Mode::Binary.accepts(format),
                "`{format}` must fit a binary frame"
            );

            assert_eq!(
                Mode::Text.accepts(format),
                format.is_human_readable(),
                "`{format}` is accepted by `text` exactly when it is human readable"
            );
        }
    }

    /// A format the crate was not built with must fail cleanly rather than
    /// misbehaving.
    #[test]
    fn unsupported_is_reported() {
        for format in Format::ALL.iter().copied() {
            if format.is_supported() {
                continue;
            }

            let mut buf = Vec::new();
            let error = format.encode(&mut buf, &1u32).unwrap_err();
            assert_eq!(error.unsupported_format(), Some(format));
        }
    }
}
