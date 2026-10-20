//! The low-level CSV parser.
//!
//! Parsing works directly over borrowed input so that unquoted fields, and
//! quoted fields without escaped quotes, can be borrowed from the input
//! without copying.

use core::fmt;

use crate::Context;
use crate::alloc::{Allocator, Vec};

use super::encoding::Config;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::SliceParser<'_> {}
    impl Sealed for super::MutSliceParser<'_, '_> {}
    impl<'de, P> Sealed for &mut P where P: ?Sized + super::Parser<'de> {}
    impl Sealed for &[u8] {}
    impl Sealed for &str {}
    impl Sealed for &mut &[u8] {}
}

/// Parsing state which is carried along with the input.
///
/// This tracks whether the record currently being parsed has more fields, and
/// how many fields have been parsed in total. The latter is used by record
/// decoders to tell if a field decoder they handed out was used or not.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct ParseState {
    /// Indicates that the current record has no more fields.
    pub(crate) record_end: bool,
    /// The total number of fields parsed so far.
    pub(crate) fields: u64,
}

impl ParseState {
    #[inline]
    const fn new() -> Self {
        Self {
            record_end: true,
            fields: 0,
        }
    }
}

/// Trait governing how CSV is parsed.
///
/// This is implemented for parsers over byte slices and strings, and is sealed
/// so it cannot be implemented outside of this crate.
pub trait Parser<'de>: self::sealed::Sealed {
    /// A borrowed parser.
    type Mut<'this>: Parser<'de>
    where
        Self: 'this;

    /// Borrow the current parser mutably.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// The remaining input.
    #[doc(hidden)]
    fn remaining(&self) -> &'de [u8];

    /// Advance the parser by `n` bytes, which must not be greater than the
    /// number of remaining bytes.
    #[doc(hidden)]
    fn advance<C>(&mut self, cx: C, n: usize)
    where
        C: Context;

    /// Access the parsing state.
    #[doc(hidden)]
    fn state(&mut self) -> &mut ParseState;

    /// Test if the parser has been exhausted, which is the case if all that
    /// remains are line terminators.
    #[inline]
    fn is_exhausted<C>(&mut self, _: C) -> bool
    where
        C: Context,
    {
        self.remaining().iter().all(|&b| matches!(b, b'\r' | b'\n'))
    }
}

impl<'de, P> Parser<'de> for &mut P
where
    P: ?Sized + Parser<'de>,
{
    type Mut<'this>
        = P::Mut<'this>
    where
        Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        (**self).borrow_mut()
    }

    #[inline]
    fn remaining(&self) -> &'de [u8] {
        (**self).remaining()
    }

    #[inline]
    fn advance<C>(&mut self, cx: C, n: usize)
    where
        C: Context,
    {
        (**self).advance(cx, n)
    }

    #[inline]
    fn state(&mut self) -> &mut ParseState {
        (**self).state()
    }

    #[inline]
    fn is_exhausted<C>(&mut self, cx: C) -> bool
    where
        C: Context,
    {
        (**self).is_exhausted(cx)
    }
}

/// A parser over a byte slice.
pub struct SliceParser<'de> {
    slice: &'de [u8],
    state: ParseState,
}

impl<'de> SliceParser<'de> {
    /// Construct a new parser over the given slice.
    #[inline]
    pub(crate) const fn new(slice: &'de [u8]) -> Self {
        Self {
            slice,
            state: ParseState::new(),
        }
    }
}

impl<'de> Parser<'de> for SliceParser<'de> {
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn remaining(&self) -> &'de [u8] {
        self.slice
    }

    #[inline]
    fn advance<C>(&mut self, cx: C, n: usize)
    where
        C: Context,
    {
        self.slice = &self.slice[n..];
        cx.advance(n);
    }

    #[inline]
    fn state(&mut self) -> &mut ParseState {
        &mut self.state
    }
}

/// A parser which advances a mutable reference to a byte slice as it is
/// parsing.
pub struct MutSliceParser<'a, 'de> {
    slice: &'a mut &'de [u8],
    state: ParseState,
}

impl<'de> Parser<'de> for MutSliceParser<'_, 'de> {
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn remaining(&self) -> &'de [u8] {
        self.slice
    }

    #[inline]
    fn advance<C>(&mut self, cx: C, n: usize)
    where
        C: Context,
    {
        *self.slice = &self.slice[n..];
        cx.advance(n);
    }

    #[inline]
    fn state(&mut self) -> &mut ParseState {
        &mut self.state
    }
}

/// Trait for types which can be converted into a CSV [`Parser`].
pub trait IntoParser<'de>: self::sealed::Sealed {
    /// The parser type being converted into.
    type Parser: Parser<'de>;

    /// Convert into a parser.
    fn into_parser(self) -> Self::Parser;
}

impl<'de> IntoParser<'de> for &'de [u8] {
    type Parser = SliceParser<'de>;

    #[inline]
    fn into_parser(self) -> Self::Parser {
        SliceParser::new(self)
    }
}

impl<'de> IntoParser<'de> for &'de str {
    type Parser = SliceParser<'de>;

    #[inline]
    fn into_parser(self) -> Self::Parser {
        SliceParser::new(self.as_bytes())
    }
}

impl<'a, 'de> IntoParser<'de> for &'a mut &'de [u8] {
    type Parser = MutSliceParser<'a, 'de>;

    #[inline]
    fn into_parser(self) -> Self::Parser {
        MutSliceParser {
            slice: self,
            state: ParseState::new(),
        }
    }
}

/// A parsed field.
pub(crate) enum Field<'de, 'scratch> {
    /// The field was borrowed from the input.
    Borrowed(&'de [u8]),
    /// The field had to be unescaped into a scratch buffer.
    Scratch(&'scratch [u8]),
}

impl Field<'_, '_> {
    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match *self {
            Field::Borrowed(bytes) => bytes,
            Field::Scratch(bytes) => bytes,
        }
    }
}

/// Error raised when a quoted field is not terminated.
struct Unterminated;

impl fmt::Display for Unterminated {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unterminated quoted field")
    }
}

/// Error raised when a closing quote is followed by something unexpected.
struct AfterQuote(u8);

impl fmt::Display for AfterQuote {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expected delimiter or end of record after closing quote, but found {:?}",
            char::from(self.0)
        )
    }
}

impl Config {
    /// Test if `b` is a byte which ends a record.
    #[inline]
    pub(crate) fn is_terminator(&self, b: u8) -> bool {
        match self.terminator.read_byte() {
            Some(t) => b == t,
            None => matches!(b, b'\r' | b'\n'),
        }
    }

    /// Length of the record terminator at the start of `input`, if any.
    #[inline]
    fn terminator_len(&self, input: &[u8]) -> Option<usize> {
        let &b = input.first()?;

        if !self.is_terminator(b) {
            return None;
        }

        if self.terminator.read_byte().is_none() && b == b'\r' && input.get(1) == Some(&b'\n') {
            return Some(2);
        }

        Some(1)
    }
}

/// Skip over any blank lines at the current position.
#[inline]
pub(crate) fn skip_blank_lines<'de, P, C>(cx: C, parser: &mut P, cfg: &Config)
where
    P: ?Sized + Parser<'de>,
    C: Context,
{
    let input = parser.remaining();
    let n = input.iter().take_while(|&&b| cfg.is_terminator(b)).count();

    if n > 0 {
        parser.advance(cx, n);
    }
}

/// Begin parsing a record.
///
/// Returns `false` if there are no more records in the input.
#[inline]
pub(crate) fn begin_record<'de, P, C>(cx: C, parser: &mut P, cfg: &Config) -> bool
where
    P: ?Sized + Parser<'de>,
    C: Context,
{
    skip_blank_lines(cx, parser, cfg);

    if parser.remaining().is_empty() {
        parser.state().record_end = true;
        return false;
    }

    parser.state().record_end = false;
    true
}

/// Test if the next field is empty, either because it is unquoted and empty or
/// because it is an empty quoted field. If `record` is set, the field must also
/// be the last field in the record.
///
/// Returns the number of bytes the field occupies, excluding the separator.
#[inline]
pub(crate) fn peek_empty<'de, P>(parser: &mut P, cfg: &Config, record: bool) -> Option<usize>
where
    P: ?Sized + Parser<'de>,
{
    let input = parser.remaining();

    let n = if input.first() == Some(&cfg.quote) && input.get(1) == Some(&cfg.quote) {
        2
    } else {
        0
    };

    match input.get(n) {
        None => Some(n),
        Some(&b) if b == cfg.delimiter && !record => Some(n),
        Some(&b) if cfg.is_terminator(b) => Some(n),
        _ => None,
    }
}

/// Consume the separator after a field which ends at `n`, or raise an error if
/// the field is not followed by a separator.
#[inline]
fn finish_field<'de, P, C>(cx: C, parser: &mut P, cfg: &Config, n: usize) -> Result<(), C::Error>
where
    P: ?Sized + Parser<'de>,
    C: Context,
{
    let input = parser.remaining();

    let record_end = match input.get(n) {
        None => {
            parser.advance(cx, n);
            true
        }
        Some(&b) if b == cfg.delimiter => {
            parser.advance(cx, n + 1);
            false
        }
        Some(&b) => {
            let Some(len) = cfg.terminator_len(&input[n..]) else {
                parser.advance(cx, n);
                return Err(cx.message(AfterQuote(b)));
            };

            parser.advance(cx, n + len);
            true
        }
    };

    let state = parser.state();
    state.record_end = record_end;
    state.fields = state.fields.wrapping_add(1);
    Ok(())
}

/// Find the end of an unquoted field.
#[inline]
fn unquoted_len(input: &[u8], cfg: &Config) -> usize {
    input
        .iter()
        .position(|&b| b == cfg.delimiter || cfg.is_terminator(b))
        .unwrap_or(input.len())
}

/// Parse a single field, which must be present.
pub(crate) fn parse_field<'de, 'scratch, P, C, A>(
    cx: C,
    parser: &mut P,
    cfg: &Config,
    scratch: &'scratch mut Vec<u8, A>,
) -> Result<Field<'de, 'scratch>, C::Error>
where
    P: ?Sized + Parser<'de>,
    C: Context,
    A: Allocator,
{
    let input = parser.remaining();

    if input.first() != Some(&cfg.quote) {
        let n = unquoted_len(input, cfg);
        finish_field(cx, parser, cfg, n)?;
        return Ok(Field::Borrowed(&input[..n]));
    }

    let mark = cx.mark();
    let mut start = 1;
    let mut escaped = false;

    loop {
        let Some(at) = input[start..].iter().position(|&b| b == cfg.quote) else {
            return Err(cx.message_at(&mark, Unterminated));
        };

        let at = start + at;

        if input.get(at + 1) == Some(&cfg.quote) {
            if !escaped {
                scratch.clear();
                escaped = true;
            }

            // Include one of the doubled quotes.
            scratch
                .extend_from_slice(&input[start..=at])
                .map_err(cx.map())?;
            start = at + 2;
            continue;
        }

        finish_field(cx, parser, cfg, at + 1)?;

        if escaped {
            scratch
                .extend_from_slice(&input[start..at])
                .map_err(cx.map())?;
            return Ok(Field::Scratch(scratch.as_slice()));
        }

        return Ok(Field::Borrowed(&input[1..at]));
    }
}

/// Skip over a single field, which must be present.
pub(crate) fn skip_field<'de, P, C>(cx: C, parser: &mut P, cfg: &Config) -> Result<(), C::Error>
where
    P: ?Sized + Parser<'de>,
    C: Context,
{
    let input = parser.remaining();

    if input.first() != Some(&cfg.quote) {
        let n = unquoted_len(input, cfg);
        return finish_field(cx, parser, cfg, n);
    }

    let mark = cx.mark();
    let mut start = 1;

    loop {
        let Some(at) = input[start..].iter().position(|&b| b == cfg.quote) else {
            return Err(cx.message_at(&mark, Unterminated));
        };

        let at = start + at;

        if input.get(at + 1) == Some(&cfg.quote) {
            start = at + 2;
            continue;
        }

        return finish_field(cx, parser, cfg, at + 1);
    }
}

/// Skip over the remaining fields of the current record.
pub(crate) fn skip_record<'de, P, C>(cx: C, parser: &mut P, cfg: &Config) -> Result<usize, C::Error>
where
    P: ?Sized + Parser<'de>,
    C: Context,
{
    let mut n = 0;

    while !parser.state().record_end {
        skip_field(cx, parser, cfg)?;
        n += 1;
    }

    Ok(n)
}
