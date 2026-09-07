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

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use musli::reader::SliceReader;

use crate::api::{
    ChannelId, DecodeBody, EncodeBody, Format, Mode, RequestHeader, ResponseHeader, VERSION,
};

/// The tag byte which terminates a binary header block.
///
/// A header id is never zero, so this can never be confused for one.
const END_OF_HEADERS: u8 = 0;

/// The mask which recovers the header id out of a binary tag byte.
const ID_MASK: u8 = 0b0011_1111;

/// The shift which recovers the width class out of a binary tag byte.
const WIDTH_SHIFT: u32 = 6;

/// One header of a message envelope.
///
/// Each header is addressed by a small `id` in the binary envelope and by
/// `name` in the text one, so the two spellings carry exactly the same model.
#[derive(Clone, Copy)]
pub(crate) struct Header {
    /// The identifier used in the binary envelope, which is never zero and
    /// never wider than [`ID_MASK`].
    pub(crate) id: u8,
    /// The name used in the text envelope.
    pub(crate) name: &'static str,
}

/// A value which can be carried by a header.
///
/// Every header value is an unsigned integer, so `u32` is the common
/// representation they are read and written through. This is what lets an
/// unknown header be skipped: the tag says how wide the value is without
/// saying anything about what it means.
pub(crate) trait HeaderValue
where
    Self: Copy,
{
    /// Widen the value so it can be written.
    fn to_u32(self) -> u32;

    /// Narrow a value which was read, failing if it does not fit.
    fn from_u32(value: u32) -> Option<Self>;
}

macro_rules! header_value {
    ($($ty:ty),* $(,)?) => {
        $(
            impl HeaderValue for $ty {
                #[inline]
                fn to_u32(self) -> u32 {
                    u32::from(self)
                }

                #[inline]
                fn from_u32(value: u32) -> Option<Self> {
                    Self::try_from(value).ok()
                }
            }
        )*
    };
}

header_value!(u8, u16);

impl HeaderValue for u32 {
    #[inline]
    fn to_u32(self) -> u32 {
        self
    }

    #[inline]
    fn from_u32(value: u32) -> Option<Self> {
        Some(value)
    }
}

impl HeaderValue for ChannelId {
    #[inline]
    fn to_u32(self) -> u32 {
        u32::from(self.raw())
    }

    #[inline]
    fn from_u32(value: u32) -> Option<Self> {
        Some(ChannelId::from_u16(u16::try_from(value).ok()?))
    }
}

/// A message envelope, which is a block of headers in either spelling.
///
/// See the [wire format] for the shape this produces.
///
/// [wire format]: crate::api#wire-format
// NB: Only the modules which speak the protocol have any use for envelopes.
#[cfg_attr(
    not(any(feature = "ws", feature = "client", feature = "web03")),
    allow(dead_code)
)]
pub(crate) trait Envelope
where
    Self: Sized,
{
    /// Every header this envelope knows about, in the order they are written.
    const HEADERS: &'static [Header];

    /// The envelope with every header left at zero, which is what an absent
    /// header reads back as.
    fn empty() -> Self;

    /// Read the value of one of [`Envelope::HEADERS`].
    fn get(&self, id: u8) -> u32;

    /// Apply a value to one of [`Envelope::HEADERS`].
    ///
    /// Fails if the value does not fit the header, which is how a peer which
    /// writes a wider value than the header can hold is caught.
    fn set(&mut self, id: u8, value: u32) -> Result<(), Error>;
}

macro_rules! envelope {
    ($ty:ident { $($id:literal => $field:ident,)* }) => {
        impl Envelope for $ty {
            const HEADERS: &'static [Header] = &[
                $(Header { id: $id, name: stringify!($field) },)*
            ];

            #[inline]
            fn empty() -> Self {
                Self {
                    $($field: HeaderValue::from_u32(0).expect("Zero fits every header"),)*
                }
            }

            #[inline]
            fn get(&self, id: u8) -> u32 {
                match id {
                    $($id => self.$field.to_u32(),)*
                    _ => 0,
                }
            }

            #[inline]
            fn set(&mut self, id: u8, value: u32) -> Result<(), Error> {
                match id {
                    $(
                        $id => {
                            let Some(value) = HeaderValue::from_u32(value) else {
                                return Err(Error::new(ErrorKind::HeaderRange {
                                    name: stringify!($field),
                                    value,
                                }));
                            };

                            self.$field = value;
                        }
                    )*
                    // NB: Callers only ever pass an id which came out of
                    // `HEADERS`, an unknown one is reported before it gets
                    // here.
                    _ => return Err(Error::new(ErrorKind::UnknownHeader { id, name: None })),
                }

                Ok(())
            }
        }
    };
}

// NB: `version` is first in both, and its id is fixed forever. It is the one
// header which has to keep meaning what it means across every version of the
// protocol, since it is what says which version the rest is written against.
envelope! {
    RequestHeader {
        1 => version,
        2 => serial,
        3 => id,
        4 => format,
        5 => channel,
    }
}

envelope! {
    ResponseHeader {
        1 => version,
        2 => serial,
        3 => broadcast,
        4 => error,
        5 => format,
        6 => channel,
    }
}

/// The id of the `version` header, which is the same in every envelope.
const VERSION_ID: u8 = 1;

/// Look up a header by its binary id.
#[inline]
fn header_by_id<T>(id: u8) -> Option<&'static Header>
where
    T: Envelope,
{
    T::HEADERS.iter().find(|h| h.id == id)
}

/// Look up a header by its text name.
#[inline]
fn header_by_name<T>(name: &str) -> Option<&'static Header>
where
    T: Envelope,
{
    T::HEADERS.iter().find(|h| h.name == name)
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

/// Write one header of a binary envelope.
///
/// The tag byte carries both the header id and how wide the value is, so a
/// reader can step over a header it does not know rather than losing its place.
#[inline]
fn write_binary_header(out: &mut Vec<u8>, id: u8, value: u32) {
    debug_assert!(id != END_OF_HEADERS && id & ID_MASK == id);

    // NB: The narrowest class which holds the value, so a small value costs no
    // more than it has to.
    if let Ok(value) = u8::try_from(value) {
        out.push(id);
        out.push(value);
    } else if let Ok(value) = u16::try_from(value) {
        out.push(id | (1 << WIDTH_SHIFT));
        out.extend_from_slice(&value.to_le_bytes());
    } else {
        out.push(id | (2 << WIDTH_SHIFT));
        out.extend_from_slice(&value.to_le_bytes());
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
    T: Envelope,
{
    match mode {
        Mode::Binary => {
            for header in T::HEADERS {
                let field = value.get(header.id);

                // NB: An absent header reads back as zero, so writing one would
                // say nothing.
                if field == 0 {
                    continue;
                }

                write_binary_header(out, header.id, field);
            }

            out.push(END_OF_HEADERS);
        }
        Mode::Text => {
            let mut writer = Utf8Writer { out };

            // NB: Every header is written, zero or not. This spelling is for
            // people to read, so being explicit beats being terse.
            for header in T::HEADERS {
                let field = value.get(header.id);

                if writeln!(writer, "{}: {field}", header.name).is_err() {
                    return Err(Error::new(ErrorKind::TextWrite));
                }
            }

            // NB: The empty line is what separates the envelope from the body.
            out.push(b'\n');
        }
    }

    Ok(())
}

/// Decode the fixed envelope of a message, advancing `at` past it.
///
/// A header which this build does not know about is stepped over and then
/// reported, since a peer which speaks headers we do not is a peer we cannot
/// safely act on.
#[cfg_attr(
    not(any(feature = "ws", feature = "client", feature = "web03")),
    allow(dead_code)
)]
pub(crate) fn decode_envelope<T>(mode: Mode, buf: &[u8], at: &mut usize) -> Result<T, Error>
where
    T: Envelope,
{
    let Some(tail) = buf.get(*at..) else {
        return Err(Error::new(ErrorKind::Overflow {
            at: *at,
            len: buf.len(),
        }));
    };

    let mut header = T::empty();
    // NB: Held rather than returned immediately so that the whole block is
    // stepped over first, which is what makes the error precise rather than a
    // desync somewhere further along.
    let mut unknown = None;

    let consumed = match mode {
        Mode::Binary => {
            let mut rest = tail;

            loop {
                let Some((&tag, next)) = rest.split_first() else {
                    return Err(Error::new(ErrorKind::HeadersUnterminated));
                };

                rest = next;

                if tag == END_OF_HEADERS {
                    break;
                }

                let id = tag & ID_MASK;

                let width = match tag >> WIDTH_SHIFT {
                    0 => 1,
                    1 => 2,
                    2 => 4,
                    // NB: Reserved for a future width, which cannot be stepped
                    // over since nothing says how wide it is.
                    _ => return Err(Error::new(ErrorKind::HeaderWidth { id })),
                };

                let Some((value, next)) = rest.split_at_checked(width) else {
                    return Err(Error::new(ErrorKind::HeaderTruncated { id, width }));
                };

                rest = next;

                let mut bytes = [0; 4];
                bytes[..width].copy_from_slice(value);
                let value = u32::from_le_bytes(bytes);

                // NB: The tag said how wide this is, so an unknown header has
                // already been stepped over by the time we get here.
                if header_by_id::<T>(id).is_none() {
                    if unknown.is_none() {
                        unknown = Some(ErrorKind::UnknownHeader { id, name: None });
                    }

                    continue;
                }

                header.set(id, value)?;
            }

            tail.len() - rest.len()
        }
        Mode::Text => {
            let Ok(text) = str::from_utf8(tail) else {
                return Err(Error::new(ErrorKind::TextNotUtf8));
            };

            let mut rest = text;

            loop {
                let Some((line, next)) = rest.split_once('\n') else {
                    return Err(Error::new(ErrorKind::HeadersUnterminated));
                };

                rest = next;

                // NB: Tolerated so that an envelope which has been through
                // something that insists on CRLF still reads.
                let line = line.strip_suffix('\r').unwrap_or(line);

                if line.is_empty() {
                    break;
                }

                let Some((name, value)) = line.split_once(':') else {
                    return Err(Error::new(ErrorKind::TextSeparator));
                };

                let name = name.trim();
                let value = value.trim();

                let Some(found) = header_by_name::<T>(name) else {
                    if unknown.is_none() {
                        unknown = Some(ErrorKind::UnknownHeader {
                            id: 0,
                            name: Some(name.to_string()),
                        });
                    }

                    continue;
                };

                let Ok(value) = value.parse::<u32>() else {
                    return Err(Error::new(ErrorKind::TextField { key: found.name }));
                };

                header.set(found.id, value)?;
            }

            tail.len() - rest.len()
        }
    };

    // NB: Ahead of the unknown header, since the version is what explains it.
    // A peer which states a version we do not have is refused outright: there
    // is no way to know which parts of what it said still mean what they used
    // to, so acting on the parts we recognize would be acting on a message we
    // have not understood.
    let version = header.get(VERSION_ID);

    if version != VERSION {
        return Err(Error::new(ErrorKind::UnsupportedVersion { version }));
    }

    if let Some(kind) = unknown {
        return Err(Error::new(kind));
    }

    *at += consumed;
    Ok(header)
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

    /// Test if the error is caused by a peer speaking a different version of
    /// the protocol, and if so return the version it stated.
    ///
    /// A session between peers which do not agree on this is refused before
    /// either of them has acted on anything the other said, see [the protocol
    /// version].
    ///
    /// [the protocol version]: crate::api#the-protocol-version
    #[inline]
    pub fn unsupported_version(&self) -> Option<u32> {
        match self.kind {
            ErrorKind::UnsupportedVersion { version } => Some(version),
            _ => None,
        }
    }

    /// Test if the error is caused by a header this build does not know about.
    ///
    /// A peer which writes one is speaking a protocol this build has never
    /// seen, which is never something to carry on through.
    #[inline]
    pub fn is_unknown_header(&self) -> bool {
        matches!(self.kind, ErrorKind::UnknownHeader { .. })
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
            TextSeparator,
            TextField { key: &'static str },
            HeadersUnterminated,
            HeaderWidth { id: u8 },
            HeaderTruncated { id: u8, width: usize },
            HeaderRange { name: &'static str, value: u32 },
            UnknownHeader { id: u8, name: Option<String> },
            UnsupportedVersion { version: u32 },
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
                    ErrorKind::TextSeparator => {
                        write!(f, "Text envelope has a line without a `:` separator")
                    }
                    ErrorKind::TextField { key } => {
                        write!(f, "Text envelope header `{key}` is not a valid number")
                    }
                    ErrorKind::HeadersUnterminated => {
                        write!(f, "Envelope is missing its end of headers marker")
                    }
                    ErrorKind::HeaderWidth { id } => {
                        write!(f, "Header {id} uses a width this build cannot skip over")
                    }
                    ErrorKind::HeaderTruncated { id, width } => {
                        write!(f, "Header {id} is missing its {width} byte value")
                    }
                    ErrorKind::HeaderRange { name, value } => {
                        write!(f, "Header `{name}` cannot hold the value {value}")
                    }
                    ErrorKind::UnknownHeader { name: Some(name), .. } => {
                        write!(f, "Unknown header `{name}`")
                    }
                    ErrorKind::UnknownHeader { id, name: None } => {
                        write!(f, "Unknown header with id {id}")
                    }
                    ErrorKind::UnsupportedVersion { version } => {
                        write!(
                            f,
                            "Peer speaks protocol version {version}, this build speaks {VERSION}"
                        )
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
    #[cfg(feature = "format-packed")]
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

    use crate::api::{ChannelId, Format, Mode, RequestHeader, ResponseHeader, VERSION};

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
                version: VERSION,
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
            version: VERSION,
            serial: 7,
            id: 11,
            format: Format::Json.to_u8(),
            channel: ChannelId::from_u16(3),
        };

        encode_envelope(Mode::Text, &mut buf, &header).unwrap();

        assert_eq!(
            core::str::from_utf8(&buf).unwrap(),
            "version: 1\nserial: 7\nid: 11\nformat: 5\nchannel: 3\n\n"
        );

        let mut buf = Vec::new();

        let header = ResponseHeader {
            version: VERSION,
            serial: 0,
            broadcast: 13,
            error: 0,
            format: Format::Json.to_u8(),
            channel: ChannelId::NONE,
        };

        encode_envelope(Mode::Text, &mut buf, &header).unwrap();

        assert_eq!(
            core::str::from_utf8(&buf).unwrap(),
            "version: 1\nserial: 0\nbroadcast: 13\nerror: 0\nformat: 5\nchannel: 0\n\n"
        );
    }

    /// Both envelopes must survive a round trip in either mode, and report
    /// exactly how much of the buffer they consumed.
    #[test]
    fn text_envelope_round_trip() {
        for mode in Mode::ALL.iter().copied() {
            let mut buf = Vec::new();

            let request = RequestHeader {
                version: VERSION,
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
                version: VERSION,
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
            version: VERSION,
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
            "version: 1\nserial: 1\nid: 2\nformat: 5\nchannel: 0\n\n{\"message\":\"hello\",\"tick\":42}"
        );

        let mut at = 0;
        let decoded: RequestHeader = decode_envelope(Mode::Text, &buf, &mut at).unwrap();
        assert_eq!(decoded.id, 2);

        let body: Message<'_> = Format::Json.decode(&buf, &mut at).unwrap();
        assert_eq!(body, expected);
        assert_eq!(at, buf.len());
    }

    /// Headers may arrive in any order and an absent one reads as zero, which
    /// is what lets a header be dropped from a message that has no use for it.
    #[test]
    fn text_envelope_is_order_independent() {
        // Headers in a different order, one which is absent, and CRLF line
        // endings throughout.
        let buf = b"channel: 3\r\nid: 11\r\nversion: 1\r\n\r\n";

        let mut at = 0;
        let header: RequestHeader = decode_envelope(Mode::Text, buf, &mut at).unwrap();

        assert_eq!(header.id, 11);
        assert_eq!(header.channel, ChannelId::from_u16(3));
        assert_eq!(header.serial, 0, "an absent header reads as zero");
        assert_eq!(header.format, 0, "an absent header reads as zero");
        assert_eq!(at, buf.len());
    }

    /// The binary envelope is a block of tagged headers so that an unknown one
    /// can be stepped over, and a header which is zero is simply not written.
    #[test]
    fn binary_envelope_is_tagged() {
        let mut buf = Vec::new();

        let header = RequestHeader {
            version: VERSION,
            serial: 7,
            id: 300,
            format: Format::Json.to_u8(),
            channel: ChannelId::NONE,
        };

        encode_envelope(Mode::Binary, &mut buf, &header).unwrap();

        assert_eq!(
            buf,
            [
                // `version` comes first and fits a byte.
                0x01, 1, //
                // `serial` fits a byte, so it is written as one.
                0x02, 7, //
                // `id` needs two, which the tag says.
                0x43, 0x2c, 0x01, //
                // `format` fits a byte.
                0x04, 5, //
                // `channel` is zero, so it is not written at all.
                0x00,
            ]
        );

        let mut at = 0;
        let decoded: RequestHeader = decode_envelope(Mode::Binary, &buf, &mut at).unwrap();

        assert_eq!(decoded.serial, 7);
        assert_eq!(decoded.id, 300);
        assert_eq!(decoded.format, Format::Json.to_u8());
        assert_eq!(decoded.channel, ChannelId::NONE);
        assert_eq!(at, buf.len());
    }

    /// A header this build does not know about has to be stepped over and then
    /// reported, in both spellings.
    ///
    /// Carrying on would mean acting on a message from a peer which speaks a
    /// protocol we have never seen, so it is refused outright.
    #[test]
    fn unknown_headers_are_rejected() {
        // A two byte header with id 63, which nothing is ever assigned.
        let buf = [0x01, 1, 0x02, 7, 0x7f, 0xff, 0xff, 0x04, 5, 0x00];

        let mut at = 0;
        let error = decode_envelope::<RequestHeader>(Mode::Binary, &buf, &mut at).unwrap_err();

        assert!(error.is_unknown_header(), "Unexpected error: {error}");
        assert!(
            error.to_string().contains("63"),
            "Unexpected error: {error}"
        );
        assert_eq!(at, 0, "a refused envelope must not advance the cursor");

        let buf = b"version: 1\nserial: 7\nfuture: whatever\nid: 11\n\n";

        let mut at = 0;
        let error = decode_envelope::<RequestHeader>(Mode::Text, buf, &mut at).unwrap_err();

        assert!(error.is_unknown_header(), "Unexpected error: {error}");

        assert!(
            error.to_string().contains("future"),
            "The error has to name the header: {error}"
        );

        assert_eq!(at, 0, "a refused envelope must not advance the cursor");
    }

    /// A malformed binary envelope has to be reported rather than read as
    /// something else.
    #[test]
    fn binary_envelope_rejects_malformed_input() {
        // No end of headers marker.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Binary, &[0x01, 1], &mut at).is_err());

        // A header whose value is cut short.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Binary, &[0x43, 1], &mut at).is_err());

        // A width class which is reserved, so the header cannot be skipped.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Binary, &[0xc2, 1, 0x00], &mut at).is_err());

        // A value which does not fit the header it is written to: `id` holds a
        // `u16`, and this writes 65536 into it.
        let mut at = 0;
        assert!(
            decode_envelope::<RequestHeader>(
                Mode::Binary,
                &[0x01, 1, 0x83, 0, 0, 1, 0, 0x00],
                &mut at
            )
            .is_err()
        );
    }

    /// A malformed text envelope has to be reported rather than silently read
    /// as something else.
    #[test]
    fn text_envelope_rejects_malformed_input() {
        // No empty line, so the envelope never ends.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Text, b"version: 1\n", &mut at).is_err());

        // A value which does not fit the header it is written to.
        let mut at = 0;
        assert!(
            decode_envelope::<RequestHeader>(Mode::Text, b"version: 1\nid: 65536\n\n", &mut at)
                .is_err()
        );

        // A line which is not a field.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Text, b"id 11\n\n", &mut at).is_err());

        // A field whose value is not a number.
        let mut at = 0;
        assert!(
            decode_envelope::<RequestHeader>(Mode::Text, b"version: 1\nid: none\n\n", &mut at)
                .is_err()
        );

        // Not text at all.
        let mut at = 0;
        assert!(decode_envelope::<RequestHeader>(Mode::Text, &[0xff, 0xfe], &mut at).is_err());
    }

    /// A peer speaking a version this build does not have is refused, which is
    /// what keeps a session between mismatched peers from getting started.
    #[test]
    fn an_unsupported_version_is_refused() {
        // A well formed envelope which states a version from the future.
        let buf = [0x01, VERSION as u8 + 1, 0x02, 7, 0x00];

        let mut at = 0;
        let error = decode_envelope::<RequestHeader>(Mode::Binary, &buf, &mut at).unwrap_err();

        assert_eq!(error.unsupported_version(), Some(VERSION + 1));
        assert_eq!(at, 0, "a refused envelope must not advance the cursor");

        let buf = b"version: 2\nserial: 7\n\n";

        let mut at = 0;
        let error = decode_envelope::<RequestHeader>(Mode::Text, buf, &mut at).unwrap_err();

        assert_eq!(error.unsupported_version(), Some(2));
        assert_eq!(at, 0, "a refused envelope must not advance the cursor");
    }

    /// An envelope which states no version at all is refused too, since a peer
    /// which does not say what it speaks has not been understood either.
    #[test]
    fn an_unstated_version_is_refused() {
        for (mode, buf) in [
            (Mode::Binary, &[0x02, 7, 0x00][..]),
            (Mode::Text, &b"serial: 7\n\n"[..]),
        ] {
            let mut at = 0;
            let error = decode_envelope::<RequestHeader>(mode, buf, &mut at).unwrap_err();

            assert_eq!(
                error.unsupported_version(),
                Some(0),
                "`{mode}` accepted an envelope which states no version"
            );
        }
    }

    /// The version is what explains which headers may appear, so a version
    /// which does not match is reported ahead of a header we do not know.
    #[test]
    fn the_version_is_reported_before_an_unknown_header() {
        // Both wrong at once: a version from the future, and a header from it.
        let buf = [0x01, VERSION as u8 + 1, 0x7f, 0xff, 0xff, 0x00];

        let mut at = 0;
        let error = decode_envelope::<RequestHeader>(Mode::Binary, &buf, &mut at).unwrap_err();

        assert_eq!(error.unsupported_version(), Some(VERSION + 1));

        assert!(
            !error.is_unknown_header(),
            "The version explains the header, so it is the more useful error"
        );
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
