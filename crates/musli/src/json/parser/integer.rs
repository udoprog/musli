use crate::json::error::IntegerError;
use crate::json::parser::Parser;
use crate::Context;

use self::traits::FromUnsigned;
pub(crate) use self::traits::{Float, Signed, Unsigned};

/// Fully deconstructed parts of a signed number.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub(crate) struct SignedPartsBase<T>
where
    T: Signed,
{
    /// If the number is negative.
    pub(crate) is_negative: bool,
    /// The unsigned component of the number.
    pub(crate) unsigned: T::Unsigned,
}

impl<T> SignedPartsBase<T>
where
    T: Signed,
{
    pub(crate) fn compute(self) -> Result<T, IntegerError> {
        let Self {
            is_negative,
            unsigned,
        } = self;

        match if is_negative {
            unsigned.negate()
        } else {
            unsigned.signed()
        } {
            Some(value) => Ok(value),
            None => Err(IntegerError::IntegerOverflow),
        }
    }
}

/// Fully deconstructed parts of a signed number.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub(crate) struct SignedPartsFull<T>
where
    T: Signed,
{
    /// If the number is negative.
    pub(crate) is_negative: bool,
    /// The unsigned component of the number.
    pub(crate) unsigned: Parts<T::Unsigned>,
}

impl<T> SignedPartsFull<T>
where
    T: Signed,
{
    #[inline(always)]
    pub(crate) fn compute(self) -> Result<T, IntegerError> {
        let Self {
            is_negative,
            unsigned: parts,
        } = self;

        let value = parts.compute()?;

        match if is_negative {
            value.negate()
        } else {
            value.signed()
        } {
            Some(value) => Ok(value),
            None => Err(IntegerError::IntegerOverflow),
        }
    }

    #[inline(always)]
    pub(crate) fn compute_float<F>(self) -> F
    where
        F: Float,
        F: FromUnsigned<T::Unsigned>,
    {
        let Self {
            is_negative,
            unsigned: parts,
        } = self;
        let value = parts.compute_float::<F>();

        if is_negative {
            value.negate()
        } else {
            value
        }
    }
}

/// The mantissa, or anything after a decimal point.
#[derive(Clone, Copy)]
pub(crate) struct Mantissa<T> {
    /// The value of the mantissa.
    value: T,
    /// The exponent of the mantissa.
    exp: i32,
}

impl<T> Mantissa<T>
where
    T: Unsigned,
{
    fn into_float<F>(self) -> Mantissa<F>
    where
        F: FromUnsigned<T>,
    {
        Mantissa {
            value: self.value.into_float::<F>(),
            exp: self.exp,
        }
    }
}

impl<F> Mantissa<F>
where
    F: Float,
{
    /// Compute as float with a negative exponent.
    #[inline]
    fn compute_float(self, e: i32) -> F {
        self.value.pow10(e - self.exp)
    }
}

impl<T> Default for Mantissa<T>
where
    T: Unsigned,
{
    fn default() -> Self {
        Self {
            value: T::ZERO,
            exp: 0i32,
        }
    }
}

/// Fully deconstructed parts of an unsigned number.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub(crate) struct Parts<T> {
    /// The base, or everything before a decimal point.
    pub(crate) base: T,
    /// The mantissa, or anything after a decimal point.
    pub(crate) m: Mantissa<T>,
    /// The exponent of the number.
    pub(crate) e: i32,
}

impl<T> Parts<T>
where
    T: Unsigned,
{
    #[inline(always)]
    pub(crate) fn compute(self) -> Result<T, IntegerError> {
        macro_rules! check {
            ($expr:expr, $kind:ident) => {
                match $expr {
                    Some(value) => value,
                    None => return Err(IntegerError::$kind),
                }
            };
        }

        let Self { mut base, m, e } = self;

        if e == 0 {
            if !m.value.is_zero() {
                return Err(IntegerError::Decimal);
            }

            return Ok(base);
        }

        if e >= 0 {
            // Decoding the specified mantissa would result in a fractional number.
            let mantissa_exp = check!(e.checked_sub(m.exp).filter(|n| *n >= 0), Decimal) as u32;

            if !base.is_zero() {
                base = check!(base.checked_pow10(e as u32), IntegerOverflow);
            }

            let base = check! {
                m.value
                    .checked_pow10(mantissa_exp)
                    .and_then(|m| base.checked_add(m)),
                IntegerOverflow
            };

            Ok(base)
        } else if !m.value.is_zero() {
            Err(IntegerError::Decimal)
        } else {
            Ok(check!(base.checked_neg_pow10(-e as u32), Decimal))
        }
    }

    #[inline(always)]
    pub(crate) fn compute_float<F>(self) -> F
    where
        F: Float,
        F: FromUnsigned<T>,
    {
        let Self { base, m, e } = self;
        base.into_float::<F>().pow10(e) + m.into_float::<F>().compute_float(e)
    }
}

/// Implementation to skip over a well-formed JSON number.
pub(crate) fn skip_number<'de, P, C>(cx: &C, mut p: P) -> Result<(), C::Error>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    p.skip_whitespace(cx);

    let start = cx.mark();

    if p.peek() == Some(b'-') {
        p.skip(cx, 1)?;
    }

    match p.read_byte(cx)? {
        b'0' => (),
        b if is_digit_nonzero(b) => {
            p.consume_while(cx, is_digit)?;
        }
        _ => {
            return Err(cx.marked_message(&start, IntegerError::InvalidNumeric));
        }
    }

    if p.peek() == Some(b'.') {
        p.skip(cx, 1)?;
        p.consume_while(cx, is_digit)?;
    }

    if matches!(p.peek(), Some(b'e') | Some(b'E')) {
        p.skip(cx, 1)?;

        match p.peek() {
            Some(b'-') => {
                p.skip(cx, 1)?;
            }
            Some(b'+') => {
                p.skip(cx, 1)?;
            }
            _ => (),
        };

        p.consume_while(cx, is_digit)?;
    }

    Ok(())
}

/// Partially parse an unsigned value.
#[cfg_attr(feature = "parse-full", allow(unused))]
#[inline(never)]
pub(crate) fn parse_unsigned_base<'de, T, C, P>(cx: &C, mut p: P) -> Result<T, C::Error>
where
    T: Unsigned,
    P: Parser<'de>,
    C: ?Sized + Context,
{
    p.skip_whitespace(cx);

    let start = cx.mark();
    decode_unsigned_base::<T, _, _>(cx, p, &start)
}

/// Fully parse an unsigned value.
#[cfg_attr(not(feature = "parse-full"), allow(unused))]
#[inline(never)]
pub(crate) fn parse_unsigned_full<'de, T, C, P>(cx: &C, mut p: P) -> Result<T, C::Error>
where
    T: Unsigned,
    P: Parser<'de>,
    C: ?Sized + Context,
{
    p.skip_whitespace(cx);

    let start = cx.mark();

    match decode_unsigned_full(cx, p, &start)?.compute() {
        Ok(value) => Ok(value),
        Err(error) => Err(cx.marked_message(&start, error)),
    }
}

/// Decode a signed integer.
#[inline(always)]
fn decode_signed_base<'de, T, C, P>(cx: &C, mut p: P) -> Result<SignedPartsBase<T>, C::Error>
where
    C: ?Sized + Context,
    T: Signed,
    P: Parser<'de>,
{
    let start = cx.mark();

    let is_negative = if p.peek() == Some(b'-') {
        p.skip(cx, 1)?;
        true
    } else {
        false
    };

    let unsigned = decode_unsigned_base::<T::Unsigned, _, _>(cx, p, &start)?;

    Ok(SignedPartsBase {
        is_negative,
        unsigned,
    })
}

/// Decode a full signed integer.
pub(crate) fn decode_signed_full<'de, T, C, P>(
    cx: &C,
    p: &mut P,
) -> Result<SignedPartsFull<T>, C::Error>
where
    C: ?Sized + Context,
    T: Signed,
    P: ?Sized + Parser<'de>,
{
    p.skip_whitespace(cx);

    decode_signed_full_inner(cx, p)
}

/// Decode a full signed integer.
#[inline(always)]
fn decode_signed_full_inner<'de, T, C, P>(cx: &C, mut p: P) -> Result<SignedPartsFull<T>, C::Error>
where
    C: ?Sized + Context,
    T: Signed,
    P: Parser<'de>,
{
    let start = cx.mark();

    let is_negative = if p.peek() == Some(b'-') {
        p.skip(cx, 1)?;
        true
    } else {
        false
    };

    let parts = decode_unsigned_full::<T::Unsigned, _, _>(cx, p, &start)?;

    Ok(SignedPartsFull {
        is_negative,
        unsigned: parts,
    })
}

/// Fully parse a signed value.
#[cfg_attr(feature = "parse-full", allow(unused))]
#[inline(never)]
pub(crate) fn parse_signed_base<'de, T, C, P>(cx: &C, mut p: P) -> Result<T, C::Error>
where
    T: Signed,
    P: Parser<'de>,
    C: ?Sized + Context,
{
    p.skip_whitespace(cx);

    let start = cx.mark();

    match decode_signed_base(cx, p)?.compute() {
        Ok(value) => Ok(value),
        Err(error) => Err(cx.marked_message(&start, error)),
    }
}

/// Fully parse a signed value.
#[cfg_attr(not(feature = "parse-full"), allow(unused))]
#[inline(never)]
pub(crate) fn parse_signed_full<'de, T, C, P>(cx: &C, mut p: P) -> Result<T, C::Error>
where
    T: Signed,
    P: Parser<'de>,
    C: ?Sized + Context,
{
    p.skip_whitespace(cx);

    let start = cx.mark();

    match decode_signed_full_inner(cx, p)?.compute() {
        Ok(value) => Ok(value),
        Err(error) => Err(cx.marked_message(&start, error)),
    }
}

/// Generically decode a single (whole) integer from a stream of bytes abiding
/// by JSON convention for format.
#[inline(always)]
fn decode_unsigned_base<'de, T, C, P>(cx: &C, mut p: P, start: &C::Mark) -> Result<T, C::Error>
where
    T: Unsigned,
    P: Parser<'de>,
    C: ?Sized + Context,
{
    let base = match p.read_byte(cx)? {
        b'0' => T::ZERO,
        b if is_digit_nonzero(b) => {
            let mut base = T::from_byte(b - b'0');

            while let Some(true) = p.peek().map(is_digit) {
                base = digit(cx, base, p.borrow_mut(), start)?;
            }

            base
        }
        _ => {
            return Err(cx.marked_message(start, IntegerError::InvalidNumeric));
        }
    };

    Ok(base)
}

/// Generically decode a single (whole) integer from a stream of bytes abiding
/// by JSON convention for format.
#[inline(always)]
fn decode_unsigned_full<'de, T, C, P>(
    cx: &C,
    mut p: P,
    start: &C::Mark,
) -> Result<Parts<T>, C::Error>
where
    T: Unsigned,
    P: Parser<'de>,
    C: ?Sized + Context,
{
    let base = decode_unsigned_base(cx, p.borrow_mut(), start)?;

    let mut m = Mantissa::<T>::default();

    if let Some(b'.') = p.peek() {
        p.skip(cx, 1)?;

        // NB: we use unchecked operations over mantissa_exp since the mantissa
        // for any supported type would overflow long before this.
        m.exp = m.exp.wrapping_add(decode_zeros(cx, p.borrow_mut())?);

        // Stored zeros so that the last segment of zeros can be ignored since
        // they have no bearing on the value of the integer.
        let mut zeros = 0;

        while let Some(true) = p.peek().map(is_digit) {
            // Accrue accumulated zeros.
            if zeros > 0 {
                m.exp += zeros;
                m.value = match m.value.checked_pow10(zeros as u32) {
                    Some(mantissa) => mantissa,
                    None => {
                        return Err(cx.marked_message(start, IntegerError::IntegerOverflow));
                    }
                };
            }

            m.exp += 1;
            m.value = digit(cx, m.value, p.borrow_mut(), start)?;
            zeros = decode_zeros(cx, p.borrow_mut())?;
        }
    }

    let e = if matches!(p.peek(), Some(b'e' | b'E')) {
        p.skip(cx, 1)?;
        decode_exponent(cx, p, start)?
    } else {
        0
    };

    Ok(Parts { base, m, e })
}

/// Decode an exponent.
#[inline(always)]
fn decode_exponent<'de, P, C>(cx: &C, mut p: P, start: &C::Mark) -> Result<i32, C::Error>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    let mut is_negative = false;
    let mut e = 0u32;

    match p.peek() {
        Some(b'-') => {
            p.skip(cx, 1)?;
            is_negative = true
        }
        Some(b'+') => {
            p.skip(cx, 1)?;
        }
        _ => (),
    };

    while let Some(true) = p.peek().map(is_digit) {
        e = digit(cx, e, p.borrow_mut(), start)?;
    }

    match if is_negative { e.negate() } else { e.signed() } {
        Some(value) => Ok(value),
        None => Err(cx.marked_message(start, IntegerError::IntegerOverflow)),
    }
}

/// Decode a single digit into `out`.
#[inline(always)]
fn digit<'de, T, C, P>(cx: &C, out: T, mut p: P, start: &C::Mark) -> Result<T, C::Error>
where
    T: Unsigned,
    P: Parser<'de>,
    C: ?Sized + Context,
{
    let Some(out) = out.checked_mul10() else {
        return Err(cx.marked_message(start, IntegerError::IntegerOverflow));
    };

    Ok(out + T::from_byte(p.read_byte(cx)? - b'0'))
}

/// Decode sequence of zeros.
#[inline(always)]
fn decode_zeros<'de, P, C>(cx: &C, mut p: P) -> Result<i32, C::Error>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    let mut count = 0i32;

    while let Some(b'0') = p.peek() {
        count = count.wrapping_add(1);
        p.skip(cx, 1)?;
    }

    Ok(count)
}

// Test if b is numeric.
#[inline]
fn is_digit(b: u8) -> bool {
    b.is_ascii_digit()
}

// Test if b is numeric.
#[inline(always)]
fn is_digit_nonzero(b: u8) -> bool {
    (b'1'..=b'9').contains(&b)
}

mod traits {
    use core::fmt;
    use core::ops::{Add, Not};

    pub(crate) trait Unsigned: Sized + fmt::Debug + Add<Self, Output = Self> {
        type Signed: Signed<Unsigned = Self>;

        const ZERO: Self;

        fn from_byte(b: u8) -> Self;

        fn is_zero(&self) -> bool;

        /// Calculate `self * 10 ** e`.
        fn checked_pow10(self, exp: u32) -> Option<Self>;

        /// Calculate `self / 10 ** e`.
        fn checked_neg_pow10(self, e: u32) -> Option<Self>;

        fn checked_mul10(self) -> Option<Self>;

        fn checked_add(self, other: Self) -> Option<Self>;

        fn checked_pow(self, exp: u32) -> Option<Self>;

        fn negate(self) -> Option<Self::Signed>;

        fn signed(self) -> Option<Self::Signed>;

        fn into_float<F>(self) -> F
        where
            F: FromUnsigned<Self>;
    }

    pub(crate) trait Signed: Sized + fmt::Debug {
        type Unsigned: Unsigned<Signed = Self>;
    }

    pub(crate) trait FromUnsigned<T> {
        fn from_unsigned(value: T) -> Self;
    }

    pub(crate) trait Float: Sized + Add<Self, Output = Self> {
        fn negate(self) -> Self;

        fn pow10(self, e: i32) -> Self;
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

                #[inline(always)]
                fn from_byte(b: u8) -> Self {
                    b as $unsigned
                }

                #[inline(always)]
                fn is_zero(&self) -> bool {
                    *self == 0
                }

                #[inline(always)]
                fn checked_pow10(self, e: u32) -> Option<Self> {
                    static POWS: [$unsigned; count!(() $($pows)*)] = [
                        $($pows),*
                    ];

                    let n = if let Some(e) = POWS.get(e as usize) {
                        *e
                    } else {
                        10.checked_pow(e)?
                    };

                    self.checked_mul(n)
                }

                #[inline(always)]
                fn checked_neg_pow10(self, e: u32) -> Option<Self> {
                    const ONE: $unsigned = 1;
                    let div = ONE.checked_pow10(e)?;

                    if self % div != 0 {
                        None
                    } else {
                        Some(self / div)
                    }
                }

                #[inline(always)]
                fn checked_mul10(self) -> Option<Self> {
                    self.checked_mul(10)
                }

                #[inline(always)]
                fn checked_add(self, other: Self) -> Option<Self> {
                    <$unsigned>::checked_add(self, other)
                }

                #[inline(always)]
                fn checked_pow(self, exp: u32) -> Option<Self> {
                    <$unsigned>::checked_pow(self, exp)
                }

                #[inline(always)]
                fn negate(self) -> Option<Self::Signed> {
                    if self > (<$unsigned>::MAX >> 1) + 1 {
                        None
                    } else {
                        Some(self.not().wrapping_add(1) as $signed)
                    }
                }

                #[inline(always)]
                fn signed(self) -> Option<Self::Signed> {
                    if self > <$unsigned>::MAX >> 1 {
                        None
                    } else {
                        Some(self as $signed)
                    }
                }

                #[inline(always)]
                fn into_float<F>(self) -> F where F: FromUnsigned<Self> {
                    F::from_unsigned(self)
                }
            }

            impl Signed for $signed {
                type Unsigned = $unsigned;
            }
        };
    }

    unsigned!(u8, i8, [1, 10, 100,]);

    unsigned!(u16, i16, [1, 10, 100, 1000, 10000,]);

    unsigned!(
        u32,
        i32,
        [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,]
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
        [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,]
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

    macro_rules! float {
        ($float:ty, $fallback:path) => {
            impl Float for $float {
                #[inline]
                fn negate(self) -> Self {
                    -self
                }

                #[inline]
                #[cfg(feature = "std")]
                fn pow10(self, e: i32) -> Self {
                    self * <$float>::powi(10.0, e)
                }

                #[inline]
                #[cfg(not(feature = "std"))]
                fn pow10(self, e: i32) -> Self {
                    self * $fallback(10.0, e)
                }
            }

            impl FromUnsigned<u8> for $float {
                fn from_unsigned(value: u8) -> Self {
                    value as $float
                }
            }

            impl FromUnsigned<u16> for $float {
                fn from_unsigned(value: u16) -> Self {
                    value as $float
                }
            }

            impl FromUnsigned<u32> for $float {
                fn from_unsigned(value: u32) -> Self {
                    value as $float
                }
            }

            impl FromUnsigned<u64> for $float {
                fn from_unsigned(value: u64) -> Self {
                    value as $float
                }
            }

            impl FromUnsigned<u128> for $float {
                fn from_unsigned(value: u128) -> Self {
                    value as $float
                }
            }
        };
    }

    float!(f32, self::no_std::powf32);
    float!(f64, self::no_std::powf64);

    #[cfg(not(feature = "std"))]
    mod no_std {
        macro_rules! powf {
            ($ty:ty, $name:ident) => {
                #[inline(never)]
                pub(crate) fn $name(mut base: $ty, mut exp: i32) -> $ty {
                    if exp == 0 {
                        return 1.0;
                    }

                    while exp & 1 == 0 {
                        base = base * base;
                        exp >>= 1;
                    }

                    if exp == 1 {
                        return base;
                    }

                    let mut acc = base;

                    while exp > 1 {
                        exp >>= 1;
                        base = base * base;

                        if exp & 1 == 1 {
                            acc = acc * base;
                        }
                    }

                    acc
                }
            };
        }

        powf!(f32, powf32);
        powf!(f64, powf64);
    }
}
