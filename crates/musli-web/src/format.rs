//! Runtime dispatch over the [`Format`]s which message bodies can be encoded
//! with.
//!
//! Dispatch has to happen at runtime rather than through generics, since a
//! server adapts to whichever format a client asks for. See the [wire format]
//! for details.
//!
//! [wire format]: crate::api#wire-format

use core::fmt;

use alloc::vec::Vec;

use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::SliceReader;
use musli::{Decode, Encode};

use crate::api::{DecodeBody, EncodeBody, Format};

/// Encode the fixed envelope of a message.
///
/// The envelope is always encoded with [`musli::packed`] regardless of which
/// format has been negotiated for bodies, which is what allows both peers to
/// read it unconditionally.
// NB: Only the modules which speak the protocol have any use for envelopes.
#[cfg_attr(
    not(any(feature = "ws", feature = "client", feature = "web03")),
    allow(dead_code)
)]
#[inline]
pub(crate) fn encode_envelope<T>(out: &mut Vec<u8>, value: &T) -> Result<(), Error>
where
    T: ?Sized + Encode<Binary>,
{
    musli::packed::encode(out, value).map_err(Error::packed)?;
    Ok(())
}

/// Decode the fixed envelope of a message, advancing `at` past it.
#[cfg_attr(
    not(any(feature = "ws", feature = "client", feature = "web03")),
    allow(dead_code)
)]
#[inline]
pub(crate) fn decode_envelope<'de, T>(buf: &'de [u8], at: &mut usize) -> Result<T, Error>
where
    T: Decode<'de, Binary, Global>,
{
    let Some(tail) = buf.get(*at..) else {
        return Err(Error::new(ErrorKind::Overflow {
            at: *at,
            len: buf.len(),
        }));
    };

    let mut reader = SliceReader::new(tail);
    let value = musli::packed::decode(&mut reader).map_err(Error::packed)?;
    *at += tail.len() - reader.remaining();
    Ok(value)
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

    use crate::api::Format;

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
        use crate::api::{ChannelId, RequestHeader};

        for format in Format::supported() {
            let header = RequestHeader {
                serial: 7,
                id: 11,
                format: format.to_u8(),
                channel: ChannelId::from_u16(3),
            };

            let mut buf = Vec::new();
            encode_envelope(&mut buf, &header).unwrap();

            let expected = Message {
                message: "body",
                tick: 9,
            };

            format.encode(&mut buf, &expected).unwrap();

            let mut at = 0;
            let decoded: RequestHeader = decode_envelope(&buf, &mut at).unwrap();

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
