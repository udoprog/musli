//! Element types and headers of the [SQLite JSONB] format.
//!
//! Every element consists of a header of between 1 and 9 bytes followed by a
//! payload. The lower four bits of the first header byte is the element type,
//! the upper four bits describe how the size of the payload is encoded.
//!
//! [SQLite JSONB]: https://sqlite.org/draft/jsonb.html

use core::fmt;

use crate::{Context, Writer};

/// The JSON value `null`. The payload is empty.
pub(crate) const NULL: u8 = 0;
/// The JSON value `true`. The payload is empty.
pub(crate) const TRUE: u8 = 1;
/// The JSON value `false`. The payload is empty.
pub(crate) const FALSE: u8 = 2;
/// An integer in canonical RFC 8259 form stored as ASCII text.
pub(crate) const INT: u8 = 3;
/// An integer in extended JSON5 form stored as ASCII text.
pub(crate) const INT5: u8 = 4;
/// A float in canonical RFC 8259 form stored as ASCII text.
pub(crate) const FLOAT: u8 = 5;
/// A float in extended JSON5 form stored as ASCII text.
pub(crate) const FLOAT5: u8 = 6;
/// A UTF-8 string which contains no escapes and needs none.
pub(crate) const TEXT: u8 = 7;
/// A UTF-8 string containing RFC 8259 escape sequences.
pub(crate) const TEXTJ: u8 = 8;
/// A UTF-8 string containing JSON5 escape sequences.
pub(crate) const TEXT5: u8 = 9;
/// A UTF-8 string stored verbatim which must be escaped to be rendered as JSON.
pub(crate) const TEXTRAW: u8 = 10;
/// An array. The payload is a sequence of elements.
pub(crate) const ARRAY: u8 = 11;
/// An object. The payload is a sequence of key and value elements.
pub(crate) const OBJECT: u8 = 12;

/// The element type of a JSONB element, for use in diagnostics.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Kind(pub(crate) u8);

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.0 {
            NULL => "NULL",
            TRUE => "TRUE",
            FALSE => "FALSE",
            INT => "INT",
            INT5 => "INT5",
            FLOAT => "FLOAT",
            FLOAT5 => "FLOAT5",
            TEXT => "TEXT",
            TEXTJ => "TEXTJ",
            TEXT5 => "TEXT5",
            TEXTRAW => "TEXTRAW",
            ARRAY => "ARRAY",
            OBJECT => "OBJECT",
            other => return write!(f, "RESERVED({other})"),
        };

        f.write_str(name)
    }
}

impl fmt::Debug for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Test if the given element type is one of the four string types.
#[inline]
pub(crate) const fn is_text(kind: u8) -> bool {
    matches!(kind, TEXT | TEXTJ | TEXT5 | TEXTRAW)
}

/// Test if the given element type is one of the two integer types.
#[inline]
pub(crate) const fn is_int(kind: u8) -> bool {
    matches!(kind, INT | INT5)
}

/// Test if the given element type is one of the two float types.
#[inline]
pub(crate) const fn is_float(kind: u8) -> bool {
    matches!(kind, FLOAT | FLOAT5)
}

/// Write the header for an element of the given `kind` with a payload of `len`
/// bytes.
///
/// Payloads of up to and including 11 bytes are stored inline in the upper four
/// bits of the header byte, larger ones are stored as a big-endian integer
/// following it.
#[inline]
pub(crate) fn write_header<W, C>(
    cx: C,
    writer: &mut W,
    kind: u8,
    len: usize,
) -> Result<(), C::Error>
where
    W: ?Sized + Writer,
    C: Context,
{
    debug_assert!(kind <= OBJECT, "Element type out of range");

    if len <= 11 {
        return writer.write_byte(cx, ((len as u8) << 4) | kind);
    }

    if let Ok(len) = u8::try_from(len) {
        writer.write_byte(cx, 0xc0 | kind)?;
        return writer.write_byte(cx, len);
    }

    if let Ok(len) = u16::try_from(len) {
        writer.write_byte(cx, 0xd0 | kind)?;
        return writer.write_bytes(cx, &len.to_be_bytes());
    }

    if let Ok(len) = u32::try_from(len) {
        writer.write_byte(cx, 0xe0 | kind)?;
        return writer.write_bytes(cx, &len.to_be_bytes());
    }

    writer.write_byte(cx, 0xf0 | kind)?;
    writer.write_bytes(cx, &(len as u64).to_be_bytes())
}
