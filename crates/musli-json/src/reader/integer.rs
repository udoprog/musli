use core::fmt;
use core::ops::{AddAssign, MulAssign, Not};

use crate::reader::{ParseError, ParseErrorKind, Parser};

pub(crate) trait Unsigned:
    Copy + fmt::Debug + Sized + MulAssign<Self> + AddAssign<Self> + Eq
{
    type Signed: Signed<Unsigned = Self>;

    const ZERO: Self;
    const TEN: Self;

    fn from_byte(b: u8) -> Self;

    fn checked_add(self, other: Self) -> Option<Self>;

    fn checked_mul(self, other: Self) -> Option<Self>;

    fn checked_pow(self, exp: u32) -> Option<Self>;

    fn negate(self) -> Option<Self::Signed>;

    fn signed(self) -> Option<Self::Signed>;
}

pub(crate) trait Signed: Sized {
    type Unsigned: Unsigned<Signed = Self>;

    fn negate(self) -> Option<Self::Unsigned>;
}

macro_rules! unsigned {
    ($unsigned:ty, $signed:ty) => {
        impl Unsigned for $unsigned {
            type Signed = $signed;

            const ZERO: Self = 0;
            const TEN: Self = 10;

            #[inline]
            fn from_byte(b: u8) -> Self {
                b as $unsigned
            }

            #[inline]
            fn checked_add(self, other: Self) -> Option<Self> {
                <$unsigned>::checked_add(self, other)
            }

            #[inline]
            fn checked_mul(self, other: Self) -> Option<Self> {
                <$unsigned>::checked_mul(self, other)
            }

            #[inline]
            fn checked_pow(self, exp: u32) -> Option<Self> {
                <$unsigned>::checked_pow(self, exp)
            }

            #[inline]
            fn negate(self) -> Option<Self::Signed> {
                if self > (<$unsigned>::MAX >> 1) + 1 {
                    None
                } else {
                    Some(self.not().wrapping_add(1) as $signed)
                }
            }

            #[inline]
            fn signed(self) -> Option<Self::Signed> {
                if self > <$unsigned>::MAX >> 1 {
                    None
                } else {
                    Some(self as $signed)
                }
            }
        }

        impl Signed for $signed {
            type Unsigned = $unsigned;

            #[inline]
            fn negate(self) -> Option<Self::Unsigned> {
                if self < 0 {
                    None
                } else {
                    Some(-self as $unsigned)
                }
            }
        }
    };
}

unsigned!(u8, i8);
unsigned!(u16, i16);
unsigned!(u32, i32);
unsigned!(u64, i64);
unsigned!(u128, i128);
unsigned!(usize, isize);

static NUM: [bool; 256] = {
    const NM: bool = true; // numeric characters
    const __: bool = false; // allow unescaped
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        NM, NM, NM, NM, NM, NM, NM, NM, NM, NM, NM, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
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

/// Implementation to skip over a well-formed JSON number.
pub(crate) fn skip_number<'de, P>(p: &mut P) -> Result<(), ParseError>
where
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();

    if p.peek_byte()? == Some(b'-') {
        p.skip(1)?;
    }

    let first = p.read_byte()?;

    if !NUM[first as usize] {
        return Err(ParseError::spanned(
            start,
            p.pos(),
            ParseErrorKind::InvalidNumeric,
        ));
    }

    while let Some(b) = p.peek_byte()? {
        if !NUM[b as usize] {
            break;
        }

        p.skip(1)?;
    }

    if p.peek_byte()? == Some(b'.') {
        p.skip(1)?;

        while let Some(b'0') = p.peek_byte()? {
            p.skip(1)?;
        }

        while let Some(b) = p.peek_byte()? {
            if !NUM[b as usize] {
                break;
            }

            p.skip(1)?;
        }
    }

    if matches!(p.peek_byte()?, Some(b'e') | Some(b'E')) {
        p.skip(1)?;

        match p.peek_byte()? {
            Some(b'-') => {
                p.skip(1)?;
            }
            Some(b'+') => {
                p.skip(1)?;
            }
            _ => (),
        };

        while let Some(b) = p.peek_byte()? {
            if !NUM[b as usize] {
                break;
            }

            p.skip(1)?;
        }
    }

    Ok(())
}

#[inline]
pub(crate) fn decode_unsigned<'de, T, P>(p: &mut P) -> Result<T, ParseError>
where
    T: Unsigned,
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();
    decode_unsigned_inner(p, start)
}

/// Generically decode a single (whole) integer from a stream of bytes abiding
/// by JSON convention for format.
pub(crate) fn decode_unsigned_inner<'de, T, P>(p: &mut P, start: u32) -> Result<T, ParseError>
where
    T: Unsigned,
    P: ?Sized + Parser<'de>,
{
    let first = p.read_byte()?;

    macro_rules! check {
        ($expr:expr, $kind:ident) => {
            match $expr {
                Some(value) => value,
                None => return Err(ParseError::spanned(start, p.pos(), ParseErrorKind::$kind)),
            }
        };
    }

    macro_rules! overflow {
        ($expr:expr) => {
            check!($expr, NumericalOverflow)
        };
    }

    if !NUM[first as usize] {
        return Err(ParseError::spanned(
            start,
            p.pos(),
            ParseErrorKind::InvalidNumeric,
        ));
    }

    let mut base = T::from_byte(first - b'0');

    while let Some(b) = p.peek_byte()? {
        if !NUM[b as usize] {
            break;
        }

        base = digit(base, p, start)?;
    }

    let mut mantissa = T::ZERO;
    let mut mantissa_exp = 0u32;

    if p.peek_byte()? == Some(b'.') {
        p.skip(1)?;

        mantissa_exp += decode_zeros(p)?;

        // Stored zeros so that the last set of zeros can be ignored.
        let mut zeros = 0;

        while let Some(b) = p.peek_byte()? {
            if !NUM[b as usize] {
                break;
            }

            if zeros > 0 {
                mantissa_exp += zeros;
                mantissa = overflow!(T::TEN
                    .checked_pow(zeros)
                    .and_then(|e| mantissa.checked_mul(e)));
            }

            mantissa_exp += 1;
            mantissa = digit(mantissa, p, start)?;
            zeros = decode_zeros(p)?;
        }
    }

    if matches!(p.peek_byte()?, Some(b'e') | Some(b'E')) {
        p.skip(1)?;

        let is_negative = match p.peek_byte()? {
            Some(b'-') => {
                p.skip(1)?;
                true
            }
            Some(b'+') => {
                p.skip(1)?;
                false
            }
            _ => false,
        };

        let mut exp = 0u32;

        while let Some(b) = p.peek_byte()? {
            if !NUM[b as usize] {
                break;
            }

            exp = digit(exp, p, start)?;
        }

        // Decoding the specified mantissa would result in a fractional number.
        mantissa_exp = check!(exp.checked_sub(mantissa_exp), ExpectedWholeNumber);

        if is_negative {
            return Err(ParseError::spanned(
                start,
                p.pos(),
                ParseErrorKind::UnsupportedExponent,
            ));
        }

        mantissa = overflow!(T::TEN
            .checked_pow(mantissa_exp)
            .and_then(|e| mantissa.checked_mul(e)));

        if base != T::ZERO {
            base = overflow!(T::TEN.checked_pow(exp).and_then(|e| base.checked_mul(e)));
        }

        base = overflow!(base.checked_add(mantissa));
    } else {
        if mantissa != T::ZERO {
            return Err(ParseError::spanned(
                start,
                p.pos(),
                ParseErrorKind::ExpectedWholeNumber,
            ));
        }
    }

    Ok(base)
}

/// Decode a single digit into `out`.
#[inline]
fn digit<'de, T, P>(mut out: T, p: &mut P, start: u32) -> Result<T, ParseError>
where
    T: Unsigned,
    P: ?Sized + Parser<'de>,
{
    let digit = T::from_byte(p.read_byte()? - b'0');

    out = match out.checked_mul(T::TEN) {
        Some(value) => value,
        None => {
            return Err(ParseError::spanned(
                start,
                p.pos(),
                ParseErrorKind::NumericalOverflow,
            ));
        }
    };

    out += digit;
    Ok(out)
}

/// Decode sequence of zeros.
fn decode_zeros<'de, P>(p: &mut P) -> Result<u32, ParseError>
where
    P: ?Sized + Parser<'de>,
{
    let mut count = 0;

    while let Some(b'0') = p.peek_byte()? {
        count += 1;
        p.skip(1)?;
    }

    Ok(count)
}

/// Decode a signed integer.
pub(crate) fn decode_signed<'de, T, P>(p: &mut P) -> Result<T, ParseError>
where
    T: Signed,
    P: ?Sized + Parser<'de>,
{
    let start = p.pos();

    let is_negative = if p.peek_byte()? == Some(b'-') {
        p.skip(1)?;
        true
    } else {
        false
    };

    let unsigned = decode_unsigned_inner::<T::Unsigned, _>(p, start)?;

    match if is_negative {
        unsigned.negate()
    } else {
        unsigned.signed()
    } {
        Some(value) => Ok(value),
        None => Err(ParseError::spanned(
            start,
            p.pos(),
            ParseErrorKind::NumericalOverflow,
        )),
    }
}
