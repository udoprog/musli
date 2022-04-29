use crate::reader::{ParseError, ParseErrorKind, Parser, SliceParser};

use crate::reader::Scratch;

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
    Borrowed(&'de [u8]),
    Scratch(&'scratch [u8]),
}

/// Specialized reader implementation from a slice.
pub(crate) fn parse_string_slice_reader<'de, 'sratch>(
    reader: &mut SliceParser<'de>,
    scratch: &'sratch mut Scratch,
    validate: bool,
) -> Result<StringReference<'de, 'sratch>, ParseError> {
    // Index of the first byte not yet copied into the scratch space.
    let mut start = reader.index;

    loop {
        while reader.index < reader.slice.len() && !ESCAPE[reader.slice[reader.index] as usize] {
            reader.index += 1;
        }

        if reader.index == reader.slice.len() {
            return Err(ParseError::at(reader.index as u32, ParseErrorKind::Eof));
        }

        match reader.slice[reader.index] {
            b'"' => {
                if scratch.is_empty() {
                    // Fast path: return a slice of the raw JSON without any
                    // copying.
                    let borrowed = &reader.slice[start..reader.index];
                    reader.index += 1;
                    return Ok(StringReference::Borrowed(borrowed));
                } else {
                    scratch.extend_from_slice(&reader.slice[start..reader.index]);
                    reader.index += 1;
                    return Ok(StringReference::Scratch(scratch.as_bytes()));
                }
            }
            b'\\' => {
                scratch.extend_from_slice(&reader.slice[start..reader.index]);
                reader.index += 1;

                if !parse_escape(reader, validate, scratch)? {
                    return Err(ParseError::spanned(
                        start as u32,
                        reader.index as u32,
                        ParseErrorKind::BufferOverflow,
                    ));
                }

                start = reader.index;
            }
            _ => {
                if validate {
                    return Err(ParseError::at(
                        reader.pos(),
                        ParseErrorKind::ControlCharacterInString,
                    ));
                }

                reader.index += 1;
            }
        }
    }
}

/// Parses a JSON escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
fn parse_escape<'de, P>(
    parser: &mut P,
    validate: bool,
    scratch: &mut Scratch,
) -> Result<bool, ParseError>
where
    P: Parser<'de>,
{
    let start = parser.pos();
    let b = parser.read_byte()?;

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
            fn encode_surrogate(scratch: &mut Scratch, n: u16) -> bool {
                scratch.extend_from_slice(&[
                    (n >> 12 & 0b0000_1111) as u8 | 0b1110_0000,
                    (n >> 6 & 0b0011_1111) as u8 | 0b1000_0000,
                    (n & 0b0011_1111) as u8 | 0b1000_0000,
                ])
            }

            let c = match parser.parse_hex_escape()? {
                n @ 0xDC00..=0xDFFF => {
                    return if validate {
                        Err(ParseError::spanned(
                            start,
                            parser.pos(),
                            ParseErrorKind::LoneLeadingSurrogatePair,
                        ))
                    } else {
                        Ok(encode_surrogate(scratch, n))
                    };
                }

                // Non-BMP characters are encoded as a sequence of two hex
                // escapes, representing UTF-16 surrogates. If deserializing a
                // utf-8 string the surrogates are required to be paired,
                // whereas deserializing a byte string accepts lone surrogates.
                n1 @ 0xD800..=0xDBFF => {
                    let pos = parser.pos();

                    if parser.read_byte()? != b'\\' {
                        return if validate {
                            Err(ParseError::at(pos, ParseErrorKind::UnexpectedHexEscapeEnd))
                        } else {
                            Ok(encode_surrogate(scratch, n1))
                        };
                    }

                    if parser.read_byte()? != b'u' {
                        return if validate {
                            Err(ParseError::at(pos, ParseErrorKind::UnexpectedHexEscapeEnd))
                        } else {
                            if !encode_surrogate(scratch, n1) {
                                return Ok(false);
                            }

                            // The \ prior to this byte started an escape sequence,
                            // so we need to parse that now. This recursive call
                            // does not blow the stack on malicious input because
                            // the escape is not \u, so it will be handled by one
                            // of the easy nonrecursive cases.
                            parse_escape(parser, validate, scratch)
                        };
                    }

                    let n2 = parser.parse_hex_escape()?;

                    if n2 < 0xDC00 || n2 > 0xDFFF {
                        return Err(ParseError::spanned(
                            start,
                            parser.pos(),
                            ParseErrorKind::LoneLeadingSurrogatePair,
                        ));
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                    match char::from_u32(n) {
                        Some(c) => c,
                        None => {
                            return Err(ParseError::spanned(
                                start,
                                parser.pos(),
                                ParseErrorKind::InvalidUnicode,
                            ));
                        }
                    }
                }

                // Every u16 outside of the surrogate ranges above is guaranteed
                // to be a legal char.
                n => char::from_u32(n as u32).unwrap(),
            };

            scratch.extend_from_slice(c.encode_utf8(&mut [0_u8; 4]).as_bytes())
        }
        _ => {
            return Err(ParseError::spanned(
                start,
                parser.pos(),
                ParseErrorKind::InvalidEscape,
            ));
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
pub(crate) fn skip_string<'de, P>(p: &mut P, validate: bool) -> Result<(), ParseError>
where
    P: ?Sized + Parser<'de>,
{
    loop {
        while let Some(b) = p.peek_byte()? {
            if ESCAPE[b as usize] {
                break;
            }

            p.skip(1)?;
        }

        let b = p.read_byte()?;

        match b {
            b'"' => {
                return Ok(());
            }
            b'\\' => {
                skip_escape(p, validate)?;
            }
            _ => {
                if validate {
                    return Err(ParseError::at(
                        p.pos(),
                        ParseErrorKind::ControlCharacterInString,
                    ));
                }
            }
        }
    }
}

/// Parses a JSON escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
fn skip_escape<'de, P>(p: &mut P, validate: bool) -> Result<(), ParseError>
where
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();
    let b = p.read_byte()?;

    match b {
        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
        b'u' => {
            match p.parse_hex_escape()? {
                0xDC00..=0xDFFF => {
                    return if validate {
                        Err(ParseError::spanned(
                            start,
                            p.pos(),
                            ParseErrorKind::LoneLeadingSurrogatePair,
                        ))
                    } else {
                        Ok(())
                    };
                }

                // Non-BMP characters are encoded as a sequence of two hex
                // escapes, representing UTF-16 surrogates. If deserializing a
                // utf-8 string the surrogates are required to be paired,
                // whereas deserializing a byte string accepts lone surrogates.
                n1 @ 0xD800..=0xDBFF => {
                    let pos = p.pos();

                    if p.read_byte()? != b'\\' {
                        return if validate {
                            Err(ParseError::at(pos, ParseErrorKind::UnexpectedHexEscapeEnd))
                        } else {
                            Ok(())
                        };
                    }

                    if p.read_byte()? != b'u' {
                        return if validate {
                            Err(ParseError::at(pos, ParseErrorKind::UnexpectedHexEscapeEnd))
                        } else {
                            // The \ prior to this byte started an escape sequence,
                            // so we need to parse that now. This recursive call
                            // does not blow the stack on malicious input because
                            // the escape is not \u, so it will be handled by one
                            // of the easy nonrecursive cases.
                            skip_escape(p, validate)
                        };
                    }

                    let n2 = p.parse_hex_escape()?;

                    if n2 < 0xDC00 || n2 > 0xDFFF {
                        return Err(ParseError::spanned(
                            start,
                            p.pos(),
                            ParseErrorKind::LoneLeadingSurrogatePair,
                        ));
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                    if char::from_u32(n).is_none() {
                        return Err(ParseError::spanned(
                            start,
                            p.pos(),
                            ParseErrorKind::InvalidUnicode,
                        ));
                    }
                }

                // Every u16 outside of the surrogate ranges above is guaranteed
                // to be a legal char.
                _ => (),
            }
        }
        _ => {
            return Err(ParseError::spanned(
                start,
                p.pos(),
                ParseErrorKind::InvalidEscape,
            ));
        }
    };

    Ok(())
}
