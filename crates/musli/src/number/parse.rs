//! The scanner which every number goes through.
//!
//! Numbers are read straight out of a slice rather than through the byte at a
//! time interface a format exposes, so that the hot loop over the digits is
//! nothing but a multiply and an add. The syntax being read is a type
//! parameter, so a parser only contains the forms that syntax accepts.

use crate::dec2flt::dec2flt;

use super::error::{Error, ErrorKind, Expected};
use super::swar;
use super::syntax::Syntax;
use super::traits::{Float, Signed, Unsigned};

/// A number decoded without knowing which type it was wanted as.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Any {
    /// A number which a `u128` holds exactly.
    Unsigned(u128),
    /// A negative number which an `i128` holds exactly.
    Signed(i128),
    /// A number which no integer holds exactly.
    Float(f64),
}

/// Parse an unsigned integer, ignoring any fraction or exponent which follows
/// it.
#[inline]
pub(crate) fn parse_unsigned_base<S, T>(input: &[u8]) -> Result<(T, usize), Error>
where
    S: Syntax,
    T: Unsigned,
{
    unsigned::<S, T, false>(input)
}

/// Parse an unsigned integer, which may be written with a fraction or an
/// exponent as long as it denotes a whole number.
#[inline]
pub(crate) fn parse_unsigned<S, T>(input: &[u8]) -> Result<(T, usize), Error>
where
    S: Syntax,
    T: Unsigned,
{
    unsigned::<S, T, true>(input)
}

/// Parse a signed integer, ignoring any fraction or exponent which follows it.
#[inline]
pub(crate) fn parse_signed_base<S, T>(input: &[u8]) -> Result<(T, usize), Error>
where
    S: Syntax,
    T: Signed,
{
    signed::<S, T, false>(input)
}

/// Parse a signed integer, which may be written with a fraction or an exponent
/// as long as it denotes a whole number.
#[inline]
pub(crate) fn parse_signed<S, T>(input: &[u8]) -> Result<(T, usize), Error>
where
    S: Syntax,
    T: Signed,
{
    signed::<S, T, true>(input)
}

#[inline]
fn unsigned<S, T, const FULL: bool>(input: &[u8]) -> Result<(T, usize), Error>
where
    S: Syntax,
    T: Unsigned,
{
    // An unsigned number never carries a `-`, so it is rejected here rather
    // than being scanned first and found to be negative afterwards.
    if let Some(b'-') = input.first() {
        return Err(Error::new(0, ErrorKind::Unexpected(Expected::Number, b'-')));
    }

    let mut s = Scan::new(input);
    let value = whole::<S, T, FULL>(&mut s)?;
    Ok((value, s.at))
}

#[inline]
fn signed<S, T, const FULL: bool>(input: &[u8]) -> Result<(T, usize), Error>
where
    S: Syntax,
    T: Signed,
{
    let mut s = Scan::new(input);
    let negative = matches!(input.first(), Some(b'-'));
    let value = whole::<S, T::Unsigned, FULL>(&mut s)?;

    let value = if negative {
        value.negate()
    } else {
        value.signed()
    };

    match value {
        Some(value) => Ok((value, s.at)),
        None => Err(Error::new(0, ErrorKind::Overflow)),
    }
}

/// Read a number and require it to denote a whole number.
#[inline]
fn whole<S, T, const FULL: bool>(s: &mut Scan<'_>) -> Result<T, Error>
where
    S: Syntax,
    T: Unsigned,
{
    let head = head::<S, T, FULL>(s)?;

    // The overwhelming majority of numbers are written as nothing but their
    // digits, which are the answer as they stand.
    if !head.more {
        if head.overflow {
            return Err(Error::new(0, ErrorKind::Overflow));
        }

        return Ok(head.value);
    }

    let tail = tail::<T>(s, head.len)?;
    combine(head, tail)
}

/// Parse a number into the closest float, which never fails on account of the
/// value being out of range since a float saturates to an infinity.
///
/// The syntax decides which forms are picked off before the digits are handed
/// to [`dec2flt`], which is where the correctly rounded conversion lives. That
/// conversion is itself permissive and accepts a superset of every syntax here,
/// including `Infinity`, `NaN`, a leading `+` and a leading point, so a syntax
/// which does not have those relies on its format not routing them here.
#[inline]
pub(crate) fn parse_float<S, F>(input: &[u8]) -> Result<(F, usize), Error>
where
    S: Syntax,
    F: Float,
{
    // A hexadecimal integer is the one form the decimal to float conversion
    // does not understand, so it is picked off first. The branch is on a
    // constant, so it is not there at all for a syntax without hexadecimals.
    if S::HEX {
        let mut s = Scan::new(input);
        let negative = sign::<S>(&mut s);

        if s.eat_hex_prefix() {
            let out = digits::<u128, 16, true>(&mut s);

            if out.len == 0 {
                return Err(s.expected(Expected::Hex));
            }

            if out.overflow {
                return Err(Error::new(0, ErrorKind::Overflow));
            }

            let value = F::from_u128(out.value);
            return Ok((if negative { -value } else { value }, s.at));
        }
    }

    match dec2flt(input) {
        Some((value, len)) => Ok((value, len)),
        None => Err(explain::<S>(input)),
    }
}

/// Parse a number without being told which type it is wanted as, preferring the
/// widest integer which holds it exactly and falling back to a float.
#[inline]
pub(crate) fn parse_any<S>(input: &[u8]) -> Result<(Any, usize), Error>
where
    S: Syntax,
{
    let mut s = Scan::new(input);
    let head = head::<S, u128, true>(&mut s)?;
    let negative = head.negative;

    // Reading the fraction and the exponent can fail on a malformed number,
    // which no amount of falling back makes into a number. Combining them into
    // a whole number is what may legitimately fail.
    let value = if head.more {
        let tail = tail::<u128>(&mut s, head.len)?;
        combine(head, tail)
    } else if head.overflow {
        Err(Error::new(0, ErrorKind::Overflow))
    } else {
        Ok(head.value)
    };

    let len = s.at;

    let any = match value {
        Ok(value) if negative => value.negate().map(Any::Signed),
        Ok(value) => Some(Any::Unsigned(value)),
        // The number is not whole, or is one no integer holds, both of which a
        // float can still represent.
        Err(..) => None,
    };

    if let Some(any) = any {
        return Ok((any, len));
    }

    let (value, _) = parse_float::<S, f64>(&input[..len])?;
    Ok((Any::Float(value), len))
}

/// Measure a well-formed number, returning how many bytes of `input` it takes
/// up.
#[inline]
pub(crate) fn skip<S>(input: &[u8]) -> Result<usize, Error>
where
    S: Syntax,
{
    let mut s = Scan::new(input);
    let head = head::<S, u128, true>(&mut s)?;

    if head.more {
        tail::<u128>(&mut s, head.len)?;
    }

    Ok(s.at)
}

/// Explain why `input` is not a number.
///
/// This is only reached once something has already been rejected, so it can
/// afford to run the whole number through the explicit scanner in order to say
/// exactly which byte is at fault.
#[cold]
#[inline(never)]
fn explain<S>(input: &[u8]) -> Error
where
    S: Syntax,
{
    match skip::<S>(input) {
        // The scanner accepts everything the float conversion does, so the two
        // only disagree if one of them has a bug.
        Ok(..) => Error::new(0, ErrorKind::Float),
        Err(error) => error,
    }
}

/// A cursor over the bytes of a number.
struct Scan<'a> {
    input: &'a [u8],
    at: usize,
}

impl<'a> Scan<'a> {
    #[inline]
    fn new(input: &'a [u8]) -> Self {
        Self { input, at: 0 }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.input.get(self.at).copied()
    }

    /// Consume `b` if it is next.
    #[inline]
    fn eat(&mut self, b: u8) -> bool {
        if self.peek() == Some(b) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    /// Consume the `0x` or `0X` which introduces a hexadecimal number.
    #[inline]
    fn eat_hex_prefix(&mut self) -> bool {
        if matches!(self.input.get(self.at), Some(b'0'))
            && matches!(self.input.get(self.at + 1), Some(b'x' | b'X'))
        {
            self.at += 2;
            true
        } else {
            false
        }
    }

    /// Build an error saying what was expected here and what is here instead.
    #[inline]
    fn expected(&self, expected: Expected) -> Error {
        match self.peek() {
            Some(b) => Error::new(self.at, ErrorKind::Unexpected(expected, b)),
            None => Error::new(self.at, ErrorKind::Eof(expected)),
        }
    }
}

/// The sign and the digits before any point, which is all there is to the vast
/// majority of numbers.
#[derive(Clone, Copy)]
struct Head<T> {
    /// Whether the number carries a `-`.
    negative: bool,
    /// The digits before the point.
    value: T,
    /// How many digits there are.
    len: usize,
    /// Set if the digits do not fit in `T`. This is not an error in itself,
    /// since a number which does not fit an integer can still be decoded as a
    /// float, and holding it back is what lets a number be measured whether or
    /// not it can be represented.
    overflow: bool,
    /// Set if the number goes on past its digits, in which case a fraction and
    /// an exponent are still to be read.
    more: bool,
}

/// Read the sign and the digits before any point.
///
/// With `FULL` unset the number stops at its digits, which is what the
/// `parse-full` feature turns off for the JSON decoder.
#[inline]
fn head<S, T, const FULL: bool>(s: &mut Scan<'_>) -> Result<Head<T>, Error>
where
    S: Syntax,
    T: Unsigned,
{
    let negative = sign::<S>(s);

    // A hexadecimal integer carries neither a fraction nor an exponent, so it
    // is complete as soon as its digits are.
    if S::HEX && s.eat_hex_prefix() {
        let out = digits::<T, 16, true>(s);

        if out.len == 0 {
            return Err(s.expected(Expected::Hex));
        }

        return Ok(Head {
            negative,
            value: out.value,
            len: out.len,
            overflow: out.overflow,
            more: false,
        });
    }

    let zero = matches!(s.peek(), Some(b'0'));
    let out = digits::<T, 10, true>(s);

    if out.len == 0 {
        // Nothing before the point is only a number if something follows it.
        if !(S::LEADING_POINT && matches!(s.peek(), Some(b'.'))) {
            return Err(s.expected(Expected::Number));
        }
    } else if zero && out.len > 1 && !S::LEADING_ZEROS {
        // Only a number of more than one digit can carry a redundant zero.
        return Err(Error::new(s.at - out.len, ErrorKind::LeadingZero));
    }

    Ok(Head {
        negative,
        value: out.value,
        len: out.len,
        overflow: out.overflow,
        more: FULL && matches!(s.peek(), Some(b'.' | b'e' | b'E')),
    })
}

/// The fraction and the exponent which follow the digits of a number.
struct Tail<T> {
    /// The fraction.
    m: Mantissa<T>,
    /// Offset of the first digit of the fraction, for diagnostics.
    at: usize,
    /// How many digits the fraction has, including the trailing zeros.
    len: usize,
    /// The explicit exponent.
    e: i32,
    /// Set if the fraction does not fit. See [`Head::overflow`].
    overflow: bool,
    /// Set if the exponent is out of range, which is likewise only fatal for a
    /// whole number.
    exponent: bool,
}

/// Read the fraction and the exponent which follow the digits of a number.
///
/// This is deliberately not inlined into [`whole`], since a number written with
/// a fraction or an exponent is the exception and keeping the machinery for it
/// out of the way keeps the common path tight.
fn tail<T>(s: &mut Scan<'_>, base_len: usize) -> Result<Tail<T>, Error>
where
    T: Unsigned,
{
    let mut out = Tail {
        m: Mantissa::ZERO,
        at: s.at,
        len: 0,
        e: 0,
        overflow: false,
        exponent: false,
    };

    if s.eat(b'.') {
        out.at = s.at;
        let (m, len, overflow) = fraction::<T>(s);

        // A lone point is not a number, though one on either end of the digits
        // is tolerated.
        if len == 0 && base_len == 0 {
            return Err(s.expected(Expected::Fraction));
        }

        out.m = m;
        out.len = len;
        out.overflow = overflow;
    }

    if matches!(s.peek(), Some(b'e' | b'E')) {
        s.at += 1;
        let (e, overflow) = exponent(s)?;
        out.e = e;
        out.exponent = overflow;
    }

    Ok(out)
}

/// Combine the digits, the fraction and the exponent into the whole number they
/// denote, which fails if that number is not whole or does not fit.
fn combine<T>(head: Head<T>, tail: Tail<T>) -> Result<T, Error>
where
    T: Unsigned,
{
    if tail.exponent {
        return Err(Error::new(0, ErrorKind::ExponentOverflow));
    }

    if head.overflow || tail.overflow {
        return Err(Error::new(0, ErrorKind::Overflow));
    }

    let mut base = head.value;
    let Tail { m, e, .. } = tail;

    let overflow = || Error::new(0, ErrorKind::Overflow);
    // Point at the fraction when there is one, and at the number as a whole when
    // it is the exponent which makes it fractional.
    let fraction = || Error::new(if tail.len > 0 { tail.at } else { 0 }, ErrorKind::Fraction);

    if e == 0 {
        if !m.value.is_zero() {
            return Err(fraction());
        }

        return Ok(base);
    }

    if e > 0 {
        // Scaling the fraction up by less than it is scaled down by would leave
        // something behind the point.
        let Some(exp) = e.checked_sub(m.exp).filter(|n| *n >= 0) else {
            return Err(fraction());
        };

        if !base.is_zero() {
            let Some(value) = base.checked_pow10(e as u32) else {
                return Err(overflow());
            };

            base = value;
        }

        let value = m
            .value
            .checked_pow10(exp as u32)
            .and_then(|m| base.checked_add(m));

        match value {
            Some(value) => Ok(value),
            None => Err(overflow()),
        }
    } else if !m.value.is_zero() {
        Err(fraction())
    } else {
        match base.checked_neg_pow10(e.unsigned_abs()) {
            Some(value) => Ok(value),
            None => Err(fraction()),
        }
    }
}

/// The digits after a point.
#[derive(Clone, Copy)]
struct Mantissa<T> {
    /// The digits as an integer, with any trailing zeros dropped.
    value: T,
    /// The power of ten `value` has to be divided by, which is the number of
    /// digits left once the trailing zeros were dropped.
    exp: i32,
}

impl<T> Mantissa<T>
where
    T: Unsigned,
{
    /// The mantissa of a number written without a point.
    const ZERO: Self = Self {
        value: T::ZERO,
        exp: 0,
    };
}

/// Consume the sign, if there is one, returning whether the number is negative.
#[inline]
fn sign<S>(s: &mut Scan<'_>) -> bool
where
    S: Syntax,
{
    match s.peek() {
        Some(b'-') => {
            s.at += 1;
            true
        }
        Some(b'+') if S::PLUS => {
            s.at += 1;
            false
        }
        _ => false,
    }
}

/// Decode the digits after the point, which are only significant up to the last
/// one which is not a zero.
///
/// Dropping the trailing zeros is what lets `1.2300` decode as the integer `123`
/// scaled down by `10^2` rather than by `10^4`, which keeps a number like
/// `4000.0000` inside the range of the type it is being decoded into.
///
/// Along with the mantissa this returns how many digits the fraction has, and
/// whether they fit.
fn fraction<T>(s: &mut Scan<'_>) -> (Mantissa<T>, usize, bool)
where
    T: Unsigned,
{
    let start = s.at;
    let mut at = start;
    // The end of the last digit in the run which is not a zero.
    let mut end = start;

    while let Some(&b) = s.input.get(at) {
        if !b.is_ascii_digit() {
            break;
        }

        at += 1;

        if b != b'0' {
            end = at;
        }
    }

    s.at = at;

    // Only the significant prefix is accumulated, so a long run of zeros costs
    // nothing beyond having been walked over.
    let mut significant = Scan {
        input: &s.input[..end],
        at: start,
    };

    let out = digits::<T, 10, false>(&mut significant);

    let m = Mantissa {
        value: out.value,
        exp: i32::try_from(out.len).unwrap_or(i32::MAX),
    };

    (m, at - start, out.overflow)
}

/// Decode the exponent which follows an `e` or an `E`.
///
/// An exponent too large to work with is not an error in itself, since a float
/// simply saturates, so it is handed back to be raised only if a whole number is
/// wanted.
fn exponent(s: &mut Scan<'_>) -> Result<(i32, bool), Error> {
    let negative = matches!(s.peek(), Some(b'-'));

    if matches!(s.peek(), Some(b'-' | b'+')) {
        s.at += 1;
    }

    let out = digits::<u32, 10, false>(s);

    if out.len == 0 {
        return Err(s.expected(Expected::Exponent));
    }

    let saturated = if negative { i32::MIN } else { i32::MAX };

    if out.overflow {
        return Ok((saturated, true));
    }

    match if negative {
        out.value.negate()
    } else {
        out.value.signed()
    } {
        Some(e) => Ok((e, false)),
        None => Ok((saturated, true)),
    }
}

/// The outcome of accumulating a run of digits.
struct Digits<T> {
    /// The accumulated value, which is only meaningful if `overflow` is unset.
    value: T,
    /// How many digits were consumed.
    len: usize,
    /// Set if the digits do not fit in `T`.
    overflow: bool,
}

/// Accumulate a run of digits in `RADIX`.
///
/// The whole run is consumed even once it stops fitting in `T`, so that the
/// extent of the number is known whether or not it can be represented.
///
/// With `WORDS` set the digits are read a word at a time where there are enough
/// of them to fill one. That only pays for a run which is long enough to reach
/// eight digits, so a run which is a fraction or an exponent, and is nearly
/// always shorter than that, leaves it out and stays small enough to keep being
/// inlined into its caller.
#[inline]
fn digits<T, const RADIX: u32, const WORDS: bool>(s: &mut Scan<'_>) -> Digits<T>
where
    T: Unsigned,
{
    let buf = s.input;
    let start = s.at;
    let mut at = start;
    let mut value = T::ZERO;

    // This many digits always fit, so the overflow check is hoisted out of the
    // loop which decodes them.
    let unchecked = buf.len().min(at + T::max_safe_digits::<RADIX>());

    // Eight digits at a time out of a word for as long as that many are still
    // known to fit, which is where a long number spends nearly all of its time.
    // The condition is on the width of `T`, so a type too narrow to ever hold
    // eight digits does not carry this at all.
    if const { WORDS && size_of::<T>() >= 4 } {
        while at + 8 <= unchecked {
            let word = swar::word(buf, at);

            let Some(digits) = (if RADIX == 16 {
                swar::hex8(word)
            } else {
                swar::dec8(word)
            }) else {
                break;
            };

            value = value.wrapping_mul_add8::<RADIX>(digits);
            at += 8;
        }
    }

    while at < unchecked {
        let Some(digit) = digit::<RADIX>(buf[at]) else {
            s.at = at;

            return Digits {
                value,
                len: at - start,
                overflow: false,
            };
        };

        value = value.wrapping_mul_add::<RADIX>(digit);
        at += 1;
    }

    let mut overflow = false;

    while let Some(digit) = buf.get(at).copied().and_then(digit::<RADIX>) {
        if !overflow {
            match value.checked_mul_add::<RADIX>(digit) {
                Some(value_) => value = value_,
                None => overflow = true,
            }
        }

        at += 1;
    }

    s.at = at;

    Digits {
        value,
        len: at - start,
        overflow,
    }
}

/// Translate a byte into the digit it stands for in `RADIX`.
#[inline]
pub(super) fn digit<const RADIX: u32>(b: u8) -> Option<u8> {
    if RADIX == 16 {
        return hex_digit(b);
    }

    match b {
        b'0'..=b'9' => Some(b - b'0'),
        _ => None,
    }
}

/// Translate a byte into the hexadecimal digit it stands for.
///
/// Which of the two runs a hexadecimal digit falls in is not something a branch
/// predictor can learn, since a number mixes them freely, so the two are worked
/// out side by side and one of them picked without branching. Setting the case
/// bit first is what makes the letters one run rather than two.
#[inline]
fn hex_digit(b: u8) -> Option<u8> {
    let decimal = b.wrapping_sub(b'0');
    let letter = (b | 0x20).wrapping_sub(b'a');
    let is_letter = letter < 6;
    let value = if is_letter { letter + 10 } else { decimal };

    if is_letter | (decimal < 10) {
        Some(value)
    } else {
        None
    }
}
