//! Parsing of the ASCII text payloads used by JSONB for numbers and of the
//! escape sequences used by the `TEXTJ` and `TEXT5` string types.

use core::fmt;
use core::str;

use crate::Context;
use crate::alloc::Vec;

use super::tag::{FLOAT, FLOAT5, INT, INT5, TEXT5, TEXTJ};

/// An integer which can be decoded from the ASCII payload of an `INT` or
/// `INT5` element.
pub(crate) trait Integer: Sized {
    fn from_str_radix(s: &str, radix: u32) -> Option<Self>;

    fn checked_neg(self) -> Option<Self>;
}

macro_rules! integer {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Integer for $ty {
                #[inline]
                fn from_str_radix(s: &str, radix: u32) -> Option<Self> {
                    <$ty>::from_str_radix(s, radix).ok()
                }

                #[inline]
                fn checked_neg(self) -> Option<Self> {
                    <$ty>::checked_neg(self)
                }
            }
        )*
    }
}

integer!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

/// A float which can be decoded from the ASCII payload of a `FLOAT` or
/// `FLOAT5` element.
pub(crate) trait Float: Sized {
    fn from_str(s: &str) -> Option<Self>;

    fn from_i128(value: i128) -> Self;
}

macro_rules! float {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Float for $ty {
                #[inline]
                fn from_str(s: &str) -> Option<Self> {
                    <$ty as core::str::FromStr>::from_str(s).ok()
                }

                #[inline]
                fn from_i128(value: i128) -> Self {
                    value as $ty
                }
            }
        )*
    }
}

float!(f32, f64);

/// Decode the payload of an integer element.
#[inline]
pub(crate) fn parse_integer<T, C>(cx: C, kind: u8, bytes: &[u8]) -> Result<T, C::Error>
where
    T: Integer,
    C: Context,
{
    let Some(value) = str::from_utf8(bytes)
        .ok()
        .and_then(|s| integer_from(kind, s))
    else {
        return Err(cx.message(BadNumber { kind, bytes }));
    };

    Ok(value)
}

/// Decode the payload of an integer or float element as a float.
#[inline]
pub(crate) fn parse_float<T, C>(cx: C, kind: u8, bytes: &[u8]) -> Result<T, C::Error>
where
    T: Float,
    C: Context,
{
    let Some(value) = str::from_utf8(bytes).ok().and_then(|s| float_from(kind, s)) else {
        return Err(cx.message(BadNumber { kind, bytes }));
    };

    Ok(value)
}

fn integer_from<T>(kind: u8, s: &str) -> Option<T>
where
    T: Integer,
{
    match kind {
        INT => T::from_str_radix(s, 10),
        INT5 => {
            let (negative, rest) = split_sign(s);

            let value = match strip_hex(rest) {
                Some(hex) => T::from_str_radix(hex, 16)?,
                None => T::from_str_radix(rest, 10)?,
            };

            if negative {
                value.checked_neg()
            } else {
                Some(value)
            }
        }
        _ => None,
    }
}

fn float_from<T>(kind: u8, s: &str) -> Option<T>
where
    T: Float,
{
    match kind {
        INT | FLOAT => T::from_str(s),
        // The JSON5 forms which Rust's own float parser does not accept are the
        // hexadecimal integers. Everything else, including `Infinity`, `NaN`,
        // a leading `+` and a leading or trailing `.`, it handles.
        INT5 | FLOAT5 => {
            let (negative, rest) = split_sign(s);

            if let Some(hex) = strip_hex(rest) {
                let value = i128::from_str_radix(hex, 16).ok()?;
                let value = if negative {
                    value.checked_neg()?
                } else {
                    value
                };
                return Some(T::from_i128(value));
            }

            T::from_str(s)
        }
        _ => None,
    }
}

#[inline]
fn split_sign(s: &str) -> (bool, &str) {
    match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    }
}

#[inline]
fn strip_hex(s: &str) -> Option<&str> {
    s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"))
}

/// Test if `bytes` contains anything which has to be escaped in order to be
/// rendered as an RFC 8259 string.
///
/// This decides whether a string is stored as [`TEXT`] or [`TEXTRAW`].
///
/// [`TEXT`]: super::tag::TEXT
/// [`TEXTRAW`]: super::tag::TEXTRAW
#[inline]
pub(crate) fn needs_escape(bytes: &[u8]) -> bool {
    bytes.iter().any(|&b| b < 0x20 || matches!(b, b'"' | b'\\'))
}

/// Translate the payload of a [`TEXTJ`] or [`TEXT5`] element into the string it
/// denotes, appending it to `out`.
///
/// [`TEXTJ`]: super::tag::TEXTJ
/// [`TEXT5`]: super::tag::TEXT5
pub(crate) fn unescape<C>(
    cx: C,
    kind: u8,
    bytes: &[u8],
    out: &mut Vec<u8, C::Allocator>,
) -> Result<(), C::Error>
where
    C: Context,
{
    let json5 = kind == TEXT5;

    let mut it = bytes.iter().copied();
    let mut start = 0;
    let mut index = 0;

    while let Some(b) = it.next() {
        index += 1;

        if b != b'\\' {
            continue;
        }

        out.extend_from_slice(&bytes[start..index - 1])
            .map_err(cx.map())?;

        let Some(escape) = it.next() else {
            return Err(cx.message(BadEscape));
        };

        index += 1;

        let c = match escape {
            b'"' => b'"',
            b'\\' => b'\\',
            b'/' => b'/',
            b'b' => 0x08,
            b'f' => 0x0c,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'u' => {
                let c = decode_unicode(cx, &mut it, &mut index)?;
                let mut buf = [0; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes())
                    .map_err(cx.map())?;
                start = index;
                continue;
            }
            // JSON5 only escapes below.
            b'\'' if json5 => b'\'',
            b'0' if json5 => 0,
            b'v' if json5 => 0x0b,
            b'x' if json5 => {
                let a = hex(cx, it.next(), &mut index)?;
                let b = hex(cx, it.next(), &mut index)?;
                let c = char::from(a << 4 | b);
                let mut buf = [0; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes())
                    .map_err(cx.map())?;
                start = index;
                continue;
            }
            // A line continuation, which contributes nothing to the string.
            b'\n' if json5 => {
                start = index;
                continue;
            }
            b'\r' if json5 => {
                if matches!(it.clone().next(), Some(b'\n')) {
                    it.next();
                    index += 1;
                }

                start = index;
                continue;
            }
            // JSON5 permits escaping a character which needs no escaping, in
            // which case it stands for itself. A multi-byte character is
            // covered by this too, since only its leading byte is escaped and
            // the rest is copied verbatim below.
            other if json5 => other,
            _ => return Err(cx.message(BadEscape)),
        };

        out.push(c).map_err(cx.map())?;
        start = index;
    }

    out.extend_from_slice(&bytes[start..]).map_err(cx.map())?;
    Ok(())
}

fn decode_unicode<C, I>(cx: C, it: &mut I, index: &mut usize) -> Result<char, C::Error>
where
    C: Context,
    I: Iterator<Item = u8>,
{
    let first = code_unit(cx, it, index)?;

    let c = match first {
        // A high surrogate, which must be followed by an escaped low surrogate.
        0xd800..=0xdbff => {
            if !matches!((it.next(), it.next()), (Some(b'\\'), Some(b'u'))) {
                return Err(cx.message(BadEscape));
            }

            *index += 2;

            let second = code_unit(cx, it, index)?;

            if !matches!(second, 0xdc00..=0xdfff) {
                return Err(cx.message(BadEscape));
            }

            0x10000 + ((first - 0xd800) << 10 | (second - 0xdc00))
        }
        other => other,
    };

    let Some(c) = char::from_u32(c) else {
        return Err(cx.message(BadEscape));
    };

    Ok(c)
}

fn code_unit<C, I>(cx: C, it: &mut I, index: &mut usize) -> Result<u32, C::Error>
where
    C: Context,
    I: Iterator<Item = u8>,
{
    let mut out = 0u32;

    for _ in 0..4 {
        out = out << 4 | u32::from(hex(cx, it.next(), index)?);
    }

    Ok(out)
}

#[inline]
fn hex<C>(cx: C, b: Option<u8>, index: &mut usize) -> Result<u8, C::Error>
where
    C: Context,
{
    let Some(b) = b else {
        return Err(cx.message(BadEscape));
    };

    *index += 1;

    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(cx.message(BadEscape)),
    }
}

struct BadNumber<'a> {
    kind: u8,
    bytes: &'a [u8],
}

impl fmt::Display for BadNumber<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BadNumber { kind, bytes } = self;
        let kind = super::tag::Kind(*kind);

        match str::from_utf8(bytes) {
            Ok(string) => write!(f, "Cannot decode {kind} payload {string:?}"),
            Err(..) => write!(f, "Cannot decode {kind} payload, which is not ASCII"),
        }
    }
}

struct BadEscape;

impl fmt::Display for BadEscape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bad escape sequence in string")
    }
}

/// Test if the given string element type stores its payload verbatim, which
/// means it can be borrowed straight out of the input.
#[inline]
pub(crate) const fn is_escaped(kind: u8) -> bool {
    matches!(kind, TEXTJ | TEXT5)
}
