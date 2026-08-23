use core::fmt;

use super::error::Error;
use super::parse;
use super::syntax::Syntax;

/// An unsigned integer which digits can be accumulated into.
pub(crate) trait Unsigned: Sized + Copy + fmt::Debug {
    /// The signed counterpart of this type.
    type Signed: Signed<Unsigned = Self>;

    const ZERO: Self;

    /// The number of digits in `RADIX` which can always be accumulated into
    /// this type without overflowing.
    fn max_safe_digits<const RADIX: u32>() -> usize;

    /// Calculate `self * RADIX + digit` without checking for overflow.
    ///
    /// Only called while the number of digits accumulated so far is known to
    /// stay within [`max_safe_digits`].
    ///
    /// [`max_safe_digits`]: Self::max_safe_digits
    fn wrapping_mul_add<const RADIX: u32>(self, digit: u8) -> Self;

    /// Calculate `self * RADIX ** 8 + digits` without checking for overflow,
    /// where `digits` is eight digits already folded together.
    ///
    /// Only called while the number of digits accumulated so far is known to
    /// stay within [`max_safe_digits`], and only for a type wide enough to hold
    /// eight digits at all.
    ///
    /// [`max_safe_digits`]: Self::max_safe_digits
    fn wrapping_mul_add8<const RADIX: u32>(self, digits: u32) -> Self;

    /// Calculate `self * RADIX + digit`, checking for overflow.
    fn checked_mul_add<const RADIX: u32>(self, digit: u8) -> Option<Self>;

    fn is_zero(&self) -> bool;

    /// Calculate `self * 10 ** e`.
    fn checked_pow10(self, e: u32) -> Option<Self>;

    /// Calculate `self / 10 ** e`, which fails if the division is not exact.
    fn checked_neg_pow10(self, e: u32) -> Option<Self>;

    fn checked_add(self, other: Self) -> Option<Self>;

    /// Reinterpret as the negative signed value of the same magnitude.
    fn negate(self) -> Option<Self::Signed>;

    /// Reinterpret as the positive signed value of the same magnitude.
    fn signed(self) -> Option<Self::Signed>;
}

/// A signed integer which can be built out of its magnitude and its sign.
pub(crate) trait Signed: Sized + fmt::Debug {
    type Unsigned: Unsigned<Signed = Self>;
}

/// An integer which can be decoded from its ASCII representation.
///
/// This exists so that a decoder which handles every integer width with one
/// generic function does not have to care whether the type it is decoding into
/// is signed.
pub(crate) trait Integer: Sized {
    /// Parse the number at the start of `input`, returning it along with the
    /// number of bytes it occupied.
    fn parse<S>(input: &[u8]) -> Result<(Self, usize), Error>
    where
        S: Syntax;
}

macro_rules! count {
    (()) => { 0 };
    ((_)) => { 1 };
    ((_ _)) => { 2 };
    ((_ _ _)) => { 3 };
    ((_ _ _ _)) => { 4 };
    ((_ _ _ _ _)) => { 5 };
    ((_ _ _ _ _ _)) => { 6 };
    ((_ _ _ _ _ _ _)) => { 7 };
    ((_ _ _ _ _ _ _ _)) => { 8 };
    ((_ _ _ _ _ _ _ _ _)) => { 9 };
    ((_ _ _ _ _ _ _ _ _ _)) => { 10 };
    ((_ _ _ _ _ _ _ _ _ _ _)) => { 11 };
    ((_ _ _ _ _ _ _ _ _ _ _ _)) => { 12 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _)) => { 13 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 14 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 15 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 16 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 17 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 18 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 19 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 20 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 21 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 22 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 23 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 24 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 25 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 26 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 27 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 28 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 29 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 30 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 31 };
    ((_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _)) => { 32 };

    (($($s:tt)*) $first:tt $($tt:tt)*) => {
        count!(($($s)* _) $($tt)*)
    };
}

macro_rules! unsigned {
    ($unsigned:ty, $signed:ty, [$($pows:literal),* $(,)?]) => {
        impl Unsigned for $unsigned {
            type Signed = $signed;

            const ZERO: Self = 0;

            #[inline]
            fn max_safe_digits<const RADIX: u32>() -> usize {
                match RADIX {
                    // Every pair of hexadecimal digits is exactly one byte.
                    16 => size_of::<$unsigned>() * 2,
                    // One less than the number of powers of ten which fit,
                    // since the largest of those is not reachable with every
                    // combination of digits.
                    _ => count!(() $($pows)*) - 1,
                }
            }

            #[inline]
            fn wrapping_mul_add<const RADIX: u32>(self, digit: u8) -> Self {
                self.wrapping_mul(RADIX as $unsigned)
                    .wrapping_add(digit as $unsigned)
            }

            #[inline]
            fn wrapping_mul_add8<const RADIX: u32>(self, digits: u32) -> Self {
                // The weight wraps for a type which eight digits fill exactly,
                // `u32` in base sixteen being the one that does. That is also
                // the case where eight digits are all there is room for, so
                // `self` is still zero and what the weight wrapped to does not
                // matter.
                self.wrapping_mul((RADIX as $unsigned).wrapping_pow(8))
                    .wrapping_add(digits as $unsigned)
            }

            #[inline]
            fn checked_mul_add<const RADIX: u32>(self, digit: u8) -> Option<Self> {
                self.checked_mul(RADIX as $unsigned)?
                    .checked_add(digit as $unsigned)
            }

            #[inline]
            fn is_zero(&self) -> bool {
                *self == 0
            }

            #[inline]
            fn checked_pow10(self, e: u32) -> Option<Self> {
                static POWS: [$unsigned; count!(() $($pows)*)] = [
                    $($pows),*
                ];

                let n = if let Some(e) = POWS.get(e as usize) {
                    *e
                } else {
                    <$unsigned>::checked_pow(10, e)?
                };

                self.checked_mul(n)
            }

            #[inline]
            fn checked_neg_pow10(self, e: u32) -> Option<Self> {
                const ONE: $unsigned = 1;
                let div = ONE.checked_pow10(e)?;

                if self % div != 0 {
                    None
                } else {
                    Some(self / div)
                }
            }

            #[inline]
            fn checked_add(self, other: Self) -> Option<Self> {
                <$unsigned>::checked_add(self, other)
            }

            #[inline]
            fn negate(self) -> Option<Self::Signed> {
                if self > (<$unsigned>::MAX >> 1) + 1 {
                    None
                } else {
                    Some((!self).wrapping_add(1) as $signed)
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
        }

        impl Integer for $unsigned {
            #[inline]
            fn parse<S>(input: &[u8]) -> Result<(Self, usize), Error>
            where
                S: Syntax,
            {
                parse::parse_unsigned::<S, Self>(input)
            }
        }

        impl Integer for $signed {
            #[inline]
            fn parse<S>(input: &[u8]) -> Result<(Self, usize), Error>
            where
                S: Syntax,
            {
                parse::parse_signed::<S, Self>(input)
            }
        }
    };
}

unsigned!(u8, i8, [1, 10, 100,]);

unsigned!(u16, i16, [1, 10, 100, 1000, 10000,]);

unsigned!(
    u32,
    i32,
    [
        1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
    ]
);

unsigned!(
    u64,
    i64,
    [
        1,
        10,
        100,
        1000,
        10000,
        100000,
        1000000,
        10000000,
        100000000,
        1000000000,
        10000000000,
        100000000000,
        1000000000000,
        10000000000000,
        100000000000000,
        1000000000000000,
        10000000000000000,
        100000000000000000,
        1000000000000000000,
        10000000000000000000,
    ]
);

unsigned!(
    u128,
    i128,
    [
        1,
        10,
        100,
        1000,
        10000,
        100000,
        1000000,
        10000000,
        100000000,
        1000000000,
        10000000000,
        100000000000,
        1000000000000,
        10000000000000,
        100000000000000,
        1000000000000000,
        10000000000000000,
        100000000000000000,
        1000000000000000000,
        10000000000000000000,
        100000000000000000000,
        1000000000000000000000,
        10000000000000000000000,
        100000000000000000000000,
        1000000000000000000000000,
        10000000000000000000000000,
        100000000000000000000000000,
        1000000000000000000000000000,
        10000000000000000000000000000,
        100000000000000000000000000000,
        1000000000000000000000000000000,
        10000000000000000000000000000000,
    ]
);

#[cfg(target_pointer_width = "32")]
unsigned!(
    usize,
    isize,
    [
        1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
    ]
);

#[cfg(target_pointer_width = "64")]
unsigned!(
    usize,
    isize,
    [
        1,
        10,
        100,
        1000,
        10000,
        100000,
        1000000,
        10000000,
        100000000,
        1000000000,
        10000000000,
        100000000000,
        1000000000000,
        10000000000000,
        100000000000000,
        1000000000000000,
        10000000000000000,
        100000000000000000,
        1000000000000000000,
        10000000000000000000,
    ]
);

/// A float which a number can be decoded into.
pub(crate) trait Float: crate::dec2flt::float::RawFloat {
    /// Convert an unsigned integer into this float, rounding if it does not fit
    /// exactly.
    fn from_u128(value: u128) -> Self;
}

impl Float for f32 {
    #[inline]
    fn from_u128(value: u128) -> Self {
        value as f32
    }
}

impl Float for f64 {
    #[inline]
    fn from_u128(value: u128) -> Self {
        value as f64
    }
}
