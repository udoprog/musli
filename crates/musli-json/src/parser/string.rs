#![allow(clippy::zero_prefixed_literal)]

use musli::{Buf, Context};

use crate::parser::{Parser, SliceParser};

// Copied and adapter form the serde-json project under the MIT and Apache 2.0
// license.
//
// See: https://github.com/serde-rs/json

// Lookup table of bytes that must be escaped. A value of true at index i means
// that byte i requires an escape sequence in the input.
static ESCAPE: [bool; 256] = {
    const CT: bool = true; // control character \x00..=\x1F
    const QU: bool = true; // quote \x22
    const BS: bool = true; // backslash \x5C
    const __: bool = false; // allow unescaped
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
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
    ]
};

/// A parsed string reference.
#[doc(hidden)]
pub enum StringReference<'de, 'scratch> {
    Borrowed(&'de str),
    Scratch(&'scratch str),
}

/// Specialized reader implementation from a slice.
pub(crate) fn parse_string_slice_reader<'de, 'scratch, C, S>(
    cx: &C,
    reader: &mut SliceParser<'de>,
    validate: bool,
    start: C::Mark,
    scratch: &'scratch mut S,
) -> Result<StringReference<'de, 'scratch>, C::Error>
where
    C: ?Sized + Context,
    S: ?Sized + Buf,
{
    // Index of the first byte not yet copied into the scratch space.
    let mut open_mark = cx.mark();
    let mut open = reader.index;

    loop {
        while reader.index < reader.slice.len() && !ESCAPE[reader.slice[reader.index] as usize] {
            reader.index = reader.index.wrapping_add(1);
            cx.advance(1);
        }

        if reader.index == reader.slice.len() {
            return Err(cx.message("End of input"));
        }

        match reader.slice[reader.index] {
            b'"' => {
                if scratch.is_empty() {
                    // Fast path: return a slice of the raw JSON without any
                    // copying.
                    let borrowed = &reader.slice[open..reader.index];
                    reader.index = reader.index.wrapping_add(1);
                    cx.advance(1);
                    check_utf8(cx, borrowed, start)?;
                    // SAFETY: we've checked each segment to be valid UTF-8.
                    let borrowed = unsafe { core::str::from_utf8_unchecked(borrowed) };
                    return Ok(StringReference::Borrowed(borrowed));
                } else {
                    let slice = &reader.slice[open..reader.index];
                    check_utf8(cx, slice, start)?;

                    if !scratch.write(slice) {
                        return Err(cx.message("Scratch buffer overflow"));
                    }

                    reader.index = reader.index.wrapping_add(1);
                    cx.advance(1);
                    // SAFETY: we've checked each segment to be valid UTF-8.
                    let scratch = unsafe { core::str::from_utf8_unchecked(scratch.as_slice()) };
                    return Ok(StringReference::Scratch(scratch));
                }
            }
            b'\\' => {
                let slice = &reader.slice[open..reader.index];
                check_utf8(cx, slice, start)?;

                if !scratch.write(slice) {
                    return Err(cx.message("Scratch buffer overflow"));
                }

                reader.index = reader.index.wrapping_add(1);
                cx.advance(1);

                if !parse_escape(cx, reader, validate, scratch)? {
                    return Err(cx.marked_message(open_mark, "Buffer overflow"));
                }

                open = reader.index;
                open_mark = cx.mark();
            }
            _ => {
                if validate {
                    return Err(
                        cx.marked_message(open_mark, "Control character while parsing string")
                    );
                }

                reader.index = reader.index.wrapping_add(1);
                cx.advance(1);
            }
        }
    }
}

/// Check that the given slice is valid UTF-8.
#[inline]
fn check_utf8<C>(cx: &C, bytes: &[u8], start: C::Mark) -> Result<(), C::Error>
where
    C: ?Sized + Context,
{
    if crate::str::from_utf8(bytes).is_err() {
        Err(cx.marked_message(start, "Invalid unicode string"))
    } else {
        Ok(())
    }
}

/// Parses a JSON escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
fn parse_escape<C, B>(
    cx: &C,
    parser: &mut SliceParser<'_>,
    validate: bool,
    scratch: &mut B,
) -> Result<bool, C::Error>
where
    C: ?Sized + Context,
    B: ?Sized + Buf,
{
    let start = cx.mark();
    let b = parser.read_byte(cx)?;

    let extend = match b {
        b'"' => scratch.push(b'"'),
        b'\\' => scratch.push(b'\\'),
        b'/' => scratch.push(b'/'),
        b'b' => scratch.push(b'\x08'),
        b'f' => scratch.push(b'\x0c'),
        b'n' => scratch.push(b'\n'),
        b'r' => scratch.push(b'\r'),
        b't' => scratch.push(b'\t'),
        b'u' => {
            fn encode_surrogate<B>(scratch: &mut B, n: u16) -> bool
            where
                B: ?Sized + Buf,
            {
                scratch.write(&[
                    (n >> 12 & 0b0000_1111) as u8 | 0b1110_0000,
                    (n >> 6 & 0b0011_1111) as u8 | 0b1000_0000,
                    (n & 0b0011_1111) as u8 | 0b1000_0000,
                ])
            }

            let c = match parser.parse_hex_escape(cx)? {
                n @ 0xDC00..=0xDFFF => {
                    return if validate {
                        Err(cx.marked_message(start, "Lone leading surrogate in hex escape"))
                    } else {
                        Ok(encode_surrogate(scratch, n))
                    };
                }

                // Non-BMP characters are encoded as a sequence of two hex
                // escapes, representing UTF-16 surrogates. If deserializing a
                // utf-8 string the surrogates are required to be paired,
                // whereas deserializing a byte string accepts lone surrogates.
                n1 @ 0xD800..=0xDBFF => {
                    let pos = cx.mark();

                    if parser.read_byte(cx)? != b'\\' {
                        return if validate {
                            Err(cx.marked_message(pos, "Unexpected end of hex escape"))
                        } else {
                            Ok(encode_surrogate(scratch, n1))
                        };
                    }

                    if parser.read_byte(cx)? != b'u' {
                        return if validate {
                            Err(cx.marked_message(pos, "Unexpected end of hex escape"))
                        } else {
                            if !encode_surrogate(scratch, n1) {
                                return Ok(false);
                            }

                            // The \ prior to this byte started an escape sequence,
                            // so we need to parse that now. This recursive call
                            // does not blow the stack on malicious input because
                            // the escape is not \u, so it will be handled by one
                            // of the easy nonrecursive cases.
                            parse_escape(cx, parser, validate, scratch)
                        };
                    }

                    let n2 = parser.parse_hex_escape(cx)?;

                    if !(0xDC00..=0xDFFF).contains(&n2) {
                        return Err(
                            cx.marked_message(start, "Lone leading surrogate in hex escape")
                        );
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                    match char::from_u32(n) {
                        Some(c) => c,
                        None => {
                            return Err(cx.marked_message(start, "Invalid unicode"));
                        }
                    }
                }

                // Every u16 outside of the surrogate ranges above is guaranteed
                // to be a legal char.
                n => char::from_u32(n as u32).unwrap(),
            };

            scratch.write(c.encode_utf8(&mut [0u8; 4]).as_bytes())
        }
        _ => {
            return Err(cx.marked_message(start, "Invalid string escape"));
        }
    };

    Ok(extend)
}

static HEX: [u8; 256] = {
    const __: u8 = 255; // not a hex digit
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        00, 01, 02, 03, 04, 05, 06, 07, 08, 09, __, __, __, __, __, __, // 3
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

pub(crate) fn decode_hex_val(val: u8) -> Option<u16> {
    let n = HEX[val as usize] as u16;

    if n == 255 {
        None
    } else {
        Some(n)
    }
}

/// Specialized reader implementation from a slice.
pub(crate) fn skip_string<'de, P, C>(cx: &C, mut p: P, validate: bool) -> Result<(), C::Error>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    loop {
        while let Some(b) = p.peek_byte(cx)? {
            if ESCAPE[b as usize] {
                break;
            }

            p.skip(cx, 1)?;
        }

        let b = p.read_byte(cx)?;

        match b {
            b'"' => {
                return Ok(());
            }
            b'\\' => {
                skip_escape(cx, p.borrow_mut(), validate)?;
            }
            _ => {
                if validate {
                    return Err(cx.message("Control character while parsing string"));
                }
            }
        }
    }
}

/// Parses a JSON escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
fn skip_escape<'de, P, C>(cx: &C, mut p: P, validate: bool) -> Result<(), C::Error>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    let start = cx.mark();
    let b = p.read_byte(cx)?;

    match b {
        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
        b'u' => {
            match p.parse_hex_escape(cx)? {
                0xDC00..=0xDFFF => {
                    return if validate {
                        Err(cx.marked_message(start, "Lone leading surrogate in hex escape"))
                    } else {
                        Ok(())
                    };
                }

                // Non-BMP characters are encoded as a sequence of two hex
                // escapes, representing UTF-16 surrogates. If deserializing a
                // utf-8 string the surrogates are required to be paired,
                // whereas deserializing a byte string accepts lone surrogates.
                n1 @ 0xD800..=0xDBFF => {
                    let pos = cx.mark();

                    if p.read_byte(cx)? != b'\\' {
                        return if validate {
                            Err(cx.marked_message(pos, "Unexpected end of hex escape"))
                        } else {
                            Ok(())
                        };
                    }

                    if p.read_byte(cx)? != b'u' {
                        return if validate {
                            Err(cx.marked_message(pos, "Unexpected end of hex escape"))
                        } else {
                            // The \ prior to this byte started an escape sequence,
                            // so we need to parse that now. This recursive call
                            // does not blow the stack on malicious input because
                            // the escape is not \u, so it will be handled by one
                            // of the easy nonrecursive cases.
                            skip_escape(cx, p, validate)
                        };
                    }

                    let n2 = p.parse_hex_escape(cx)?;

                    if !(0xDC00..=0xDFFF).contains(&n2) {
                        return Err(
                            cx.marked_message(start, "Lone leading surrogate in hex escape")
                        );
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                    if char::from_u32(n).is_none() {
                        return Err(cx.marked_message(start, "Invalid unicode"));
                    }
                }

                // Every u16 outside of the surrogate ranges above is guaranteed
                // to be a legal char.
                _ => (),
            }
        }
        _ => {
            return Err(cx.marked_message(start, "Invalid string escape"));
        }
    };

    Ok(())
}
