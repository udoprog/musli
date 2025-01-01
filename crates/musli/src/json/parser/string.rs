#![allow(clippy::zero_prefixed_literal)]

use crate::alloc::{Allocator, Vec};
use crate::Context;

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

/// Accessor for a slice.
pub(crate) struct SliceAccess<'de, C> {
    cx: C,
    slice: &'de [u8],
    pub(crate) index: usize,
}

impl<'de, C> SliceAccess<'de, C>
where
    C: Context,
{
    #[inline]
    pub(crate) fn new(cx: C, slice: &'de [u8], index: usize) -> Self {
        Self { cx, slice, index }
    }

    #[inline]
    fn next(&mut self) -> Result<u8, C::Error> {
        let Some(b) = self.slice.get(self.index) else {
            return Err(self.cx.message("End of input"));
        };

        self.cx.advance(1);
        self.index += 1;
        Ok(*b)
    }

    #[inline]
    fn parse_hex_escape(&mut self) -> Result<u16, C::Error> {
        let &[a, b, c, d, ..] = &self.slice[self.index..] else {
            return Err(self.cx.message("Unexpected end of hex escape"));
        };

        let mut n = 0;
        let start = self.cx.mark();

        for b in [a, b, c, d] {
            let Some(val) = decode_hex_val(b) else {
                return Err(self
                    .cx
                    .marked_message(&start, "Non-hex digit in escape sequence"));
            };

            n = (n << 4) + val;
        }

        self.index += 4;
        self.cx.advance(4);
        Ok(n)
    }

    /// Parses a JSON escape sequence and appends it into the scratch space. Assumes
    /// the previous byte read was a backslash.
    pub(crate) fn parse_escape(
        &mut self,
        validate: bool,
        scratch: &mut Vec<u8, C::Allocator>,
    ) -> Result<bool, C::Error> {
        let start = self.cx.mark();
        let b = self.next()?;

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
                fn encode_surrogate(scratch: &mut Vec<u8, impl Allocator>, n: u16) -> bool {
                    scratch.write(&[
                        (n >> 12 & 0b0000_1111) as u8 | 0b1110_0000,
                        (n >> 6 & 0b0011_1111) as u8 | 0b1000_0000,
                        (n & 0b0011_1111) as u8 | 0b1000_0000,
                    ])
                }

                let c = match self.parse_hex_escape()? {
                    n @ 0xDC00..=0xDFFF => {
                        return if validate {
                            Err(self
                                .cx
                                .marked_message(&start, "Lone leading surrogate in hex escape"))
                        } else {
                            Ok(encode_surrogate(scratch, n))
                        };
                    }

                    // Non-BMP characters are encoded as a sequence of two hex
                    // escapes, representing UTF-16 surrogates. If deserializing a
                    // utf-8 string the surrogates are required to be paired,
                    // whereas deserializing a byte string accepts lone surrogates.
                    n1 @ 0xD800..=0xDBFF => {
                        let pos = self.cx.mark();

                        if self.next()? != b'\\' {
                            return if validate {
                                Err(self.cx.marked_message(&pos, "Unexpected end of hex escape"))
                            } else {
                                Ok(encode_surrogate(scratch, n1))
                            };
                        }

                        if self.next()? != b'u' {
                            return if validate {
                                Err(self.cx.marked_message(&pos, "Unexpected end of hex escape"))
                            } else {
                                if !encode_surrogate(scratch, n1) {
                                    return Ok(false);
                                }

                                // The \ prior to this byte started an escape sequence,
                                // so we need to parse that now. This recursive call
                                // does not blow the stack on malicious input because
                                // the escape is not \u, so it will be handled by one
                                // of the easy nonrecursive cases.
                                self.parse_escape(validate, scratch)
                            };
                        }

                        let n2 = self.parse_hex_escape()?;

                        if !(0xDC00..=0xDFFF).contains(&n2) {
                            return Err(self
                                .cx
                                .marked_message(&start, "Lone leading surrogate in hex escape"));
                        }

                        let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                        match char::from_u32(n) {
                            Some(c) => c,
                            None => {
                                return Err(self.cx.marked_message(&start, "Invalid unicode"));
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
                return Err(self.cx.marked_message(&start, "Invalid string escape"));
            }
        };

        Ok(extend)
    }

    /// Parses a JSON escape sequence and appends it into the scratch space. Assumes
    /// the previous byte read was a backslash.
    fn skip_escape(&mut self, validate: bool) -> Result<(), C::Error> {
        let start = self.cx.mark();
        let b = self.next()?;

        match b {
            b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => (),
            b'u' => {
                match self.parse_hex_escape()? {
                    0xDC00..=0xDFFF => {
                        return if validate {
                            Err(self
                                .cx
                                .marked_message(&start, "Lone leading surrogate in hex escape"))
                        } else {
                            Ok(())
                        };
                    }

                    // Non-BMP characters are encoded as a sequence of two hex
                    // escapes, representing UTF-16 surrogates. If deserializing a
                    // utf-8 string the surrogates are required to be paired,
                    // whereas deserializing a byte string accepts lone surrogates.
                    n1 @ 0xD800..=0xDBFF => {
                        let pos = self.cx.mark();

                        if self.next()? != b'\\' {
                            return if validate {
                                Err(self.cx.marked_message(&pos, "Unexpected end of hex escape"))
                            } else {
                                Ok(())
                            };
                        }

                        if self.next()? != b'u' {
                            return if validate {
                                Err(self.cx.marked_message(&pos, "Unexpected end of hex escape"))
                            } else {
                                // The \ prior to this byte started an escape sequence,
                                // so we need to parse that now. This recursive call
                                // does not blow the stack on malicious input because
                                // the escape is not \u, so it will be handled by one
                                // of the easy nonrecursive cases.
                                self.skip_escape(validate)
                            };
                        }

                        let n2 = self.parse_hex_escape()?;

                        if !(0xDC00..=0xDFFF).contains(&n2) {
                            return Err(self
                                .cx
                                .marked_message(&start, "Lone leading surrogate in hex escape"));
                        }

                        let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                        if char::from_u32(n).is_none() {
                            return Err(self.cx.marked_message(&start, "Invalid unicode"));
                        }
                    }

                    // Every u16 outside of the surrogate ranges above is guaranteed
                    // to be a legal char.
                    _ => (),
                }
            }
            _ => {
                return Err(self.cx.marked_message(&start, "Invalid string escape"));
            }
        };

        Ok(())
    }

    /// Specialized reader implementation from a slice.
    pub(crate) fn parse_string<'scratch>(
        &mut self,
        validate: bool,
        start: &C::Mark,
        scratch: &'scratch mut Vec<u8, C::Allocator>,
    ) -> Result<StringReference<'de, 'scratch>, C::Error> {
        // Index of the first byte not yet copied into the scratch space.
        let mut open_mark = self.cx.mark();
        let mut open = self.index;

        loop {
            while self.index < self.slice.len() && !ESCAPE[self.slice[self.index] as usize] {
                self.index = self.index.wrapping_add(1);
                self.cx.advance(1);
            }

            if self.index == self.slice.len() {
                return Err(self.cx.message("End of input"));
            }

            match self.slice[self.index] {
                b'"' => {
                    if scratch.is_empty() {
                        // Fast path: return a slice of the raw JSON without any
                        // copying.
                        let borrowed = &self.slice[open..self.index];

                        self.index = self.index.wrapping_add(1);
                        self.cx.advance(1);

                        self.check_utf8(borrowed, start)?;

                        // SAFETY: we've checked each segment to be valid UTF-8.
                        let borrowed = unsafe { core::str::from_utf8_unchecked(borrowed) };
                        return Ok(StringReference::Borrowed(borrowed));
                    } else {
                        let slice = &self.slice[open..self.index];
                        self.check_utf8(slice, start)?;

                        if !scratch.write(slice) {
                            return Err(self.cx.message("Scratch buffer overflow"));
                        }

                        self.index = self.index.wrapping_add(1);
                        self.cx.advance(1);

                        // SAFETY: we've checked each segment to be valid UTF-8.
                        let scratch = unsafe { core::str::from_utf8_unchecked(scratch.as_slice()) };
                        return Ok(StringReference::Scratch(scratch));
                    }
                }
                b'\\' => {
                    let slice = &self.slice[open..self.index];
                    self.check_utf8(slice, start)?;

                    if !scratch.write(slice) {
                        return Err(self.cx.message("Scratch buffer overflow"));
                    }

                    self.index = self.index.wrapping_add(1);
                    self.cx.advance(1);

                    if !self.parse_escape(validate, scratch)? {
                        return Err(self.cx.marked_message(&open_mark, "Buffer overflow"));
                    }

                    open = self.index;
                    open_mark = self.cx.mark();
                }
                _ => {
                    if validate {
                        return Err(self
                            .cx
                            .marked_message(&open_mark, "Control character while parsing string"));
                    }

                    self.index = self.index.wrapping_add(1);
                    self.cx.advance(1);
                }
            }
        }
    }

    /// Specialized reader implementation from a slice.
    pub(crate) fn skip_string(&mut self) -> Result<(), C::Error> {
        loop {
            while let Some(b) = self.slice.get(self.index) {
                if ESCAPE[*b as usize] {
                    break;
                }

                self.index = self.index.wrapping_add(1);
                self.cx.advance(1);
            }

            let b = self.next()?;

            match b {
                b'"' => {
                    return Ok(());
                }
                b'\\' => {
                    self.skip_escape(true)?;
                }
                _ => {
                    return Err(self.cx.message("Control character while parsing string"));
                }
            }
        }
    }

    /// Check that the given slice is valid UTF-8.
    #[inline]
    fn check_utf8(&self, bytes: &[u8], start: &C::Mark) -> Result<(), C::Error> {
        if crate::str::from_utf8(bytes).is_err() {
            Err(self.cx.marked_message(start, "Invalid unicode string"))
        } else {
            Ok(())
        }
    }
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

#[inline]
pub(crate) fn decode_hex_val(val: u8) -> Option<u16> {
    let n = HEX[val as usize] as u16;

    if n == 255 {
        None
    } else {
        Some(n)
    }
}
