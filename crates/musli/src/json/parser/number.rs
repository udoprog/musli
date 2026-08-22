//! Number parsing for JSON, which adapts the shared [`number`] parser to the
//! [`Parser`] interface and to reporting errors through a [`Context`].
//!
//! [`number`]: crate::number

use crate::Context;
use crate::json::parser::Parser;
use crate::number::{self, Json, Signed, Unsigned};

/// Parse a number out of `p`, advancing it past the number and anchoring any
/// diagnostic at the byte the number started on.
macro_rules! parse {
    (
        $(#[$meta:meta])*
        fn $name:ident<$bound:ident> = $parse:ident;
    ) => {
        $(#[$meta])*
        #[inline]
        pub(crate) fn $name<'de, T, C, P>(cx: C, mut p: P) -> Result<T, C::Error>
        where
            T: $bound,
            P: Parser<'de>,
            C: Context,
        {
            p.skip_whitespace(cx);

            let start = cx.mark();

            match number::$parse::<Json, T>(p.remaining()) {
                Ok((value, len)) => {
                    p.skip(cx, len)?;
                    Ok(value)
                }
                Err(error) => Err(report(cx, &mut p, &start, error)),
            }
        }
    };
}

parse! {
    /// Parse an unsigned integer, ignoring any fraction or exponent.
    #[cfg_attr(feature = "parse-full", allow(unused))]
    fn parse_unsigned_base<Unsigned> = parse_unsigned_base;
}

parse! {
    /// Parse an unsigned integer which may be written with a fraction or an
    /// exponent, as long as it denotes a whole number.
    #[cfg_attr(not(feature = "parse-full"), allow(unused))]
    fn parse_unsigned_full<Unsigned> = parse_unsigned;
}

parse! {
    /// Parse a signed integer, ignoring any fraction or exponent.
    #[cfg_attr(feature = "parse-full", allow(unused))]
    fn parse_signed_base<Signed> = parse_signed_base;
}

parse! {
    /// Parse a signed integer which may be written with a fraction or an
    /// exponent, as long as it denotes a whole number.
    #[cfg_attr(not(feature = "parse-full"), allow(unused))]
    fn parse_signed_full<Signed> = parse_signed;
}

/// Parse a number without being told which type it is wanted as.
#[inline]
pub(crate) fn parse_any<'de, C, P>(cx: C, p: &mut P) -> Result<number::Any, C::Error>
where
    C: Context,
    P: ?Sized + Parser<'de>,
{
    p.skip_whitespace(cx);

    let start = cx.mark();

    match number::parse_any::<Json>(p.remaining()) {
        Ok((any, len)) => {
            p.skip(cx, len)?;
            Ok(any)
        }
        Err(error) => Err(report(cx, p, &start, error)),
    }
}

/// Skip over a well-formed number.
#[inline]
pub(crate) fn skip_number<'de, P, C>(cx: C, mut p: P) -> Result<(), C::Error>
where
    P: Parser<'de>,
    C: Context,
{
    p.skip_whitespace(cx);

    let start = cx.mark();

    match number::skip::<Json>(p.remaining()) {
        Ok(len) => p.skip(cx, len),
        Err(error) => Err(report(cx, &mut p, &start, error)),
    }
}

/// Parse a floating point number.
#[inline]
pub(crate) fn parse_float<'de, F, C, P>(cx: C, p: &mut P) -> Result<F, C::Error>
where
    F: number::Float,
    C: Context,
    P: ?Sized + Parser<'de>,
{
    let start = cx.mark();

    match number::parse_float::<Json, F>(p.remaining()) {
        Ok((value, len)) => {
            p.skip(cx, len)?;
            Ok(value)
        }
        Err(error) => Err(report(cx, p, &start, error)),
    }
}

/// Report `error` against the number which begins at `start`.
///
/// The parser is first advanced past the byte which is at fault, so that the
/// span the diagnostic carries covers the offending part of the input rather
/// than collapsing onto the byte the number began on.
#[inline]
fn report<'de, C, P>(cx: C, p: &mut P, start: &C::Mark, error: number::Error) -> C::Error
where
    C: Context,
    P: ?Sized + Parser<'de>,
{
    let n = error.at().saturating_add(1).min(p.remaining().len());

    if let Err(error) = p.skip(cx, n) {
        return error;
    }

    cx.message_at(start, error)
}
