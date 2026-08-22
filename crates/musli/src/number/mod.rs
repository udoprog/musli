//! Parsing of numbers written out as ASCII text.
//!
//! Both [`json`] and [`sqlite_jsonb`] store their numbers this way, the latter
//! in the payload of its `INT`, `INT5`, `FLOAT` and `FLOAT5` elements, so they
//! share one parser here. Which forms that parser accepts is decided by a
//! [`Syntax`], of which [`Json`] covers [RFC 8259] and [`Json5`] the extensions
//! SQLite understands on top of it.
//!
//! The parser is explicit rather than being built on [`str::parse`], for two
//! reasons. It says which byte of a number is at fault instead of only that the
//! number is bad, and it decides exactly when a number written with a fraction
//! or an exponent still denotes a whole number, so that `1.00` and `100e-2` both
//! decode into an integer while `1.5` does not.
//!
//! Converting the decimal digits of a float into the closest binary float is
//! left to [`dec2flt`], which is correctly rounded.
//!
//! [RFC 8259]: https://datatracker.ietf.org/doc/html/rfc8259#section-6
//! [`dec2flt`]: crate::dec2flt
//! [`Syntax`]: self::syntax::Syntax
//! [`json`]: crate::json
//! [`sqlite_jsonb`]: crate::sqlite_jsonb
//! [`str::parse`]: str::parse

// Between them the two formats use everything here, so a build with both of
// them on still reports anything which has genuinely gone dead. A build with
// only one of them leaves the other's share unused, which is not worth cutting
// the module up over.
#![cfg_attr(not(all(feature = "json", feature = "sqlite-jsonb")), allow(unused))]

#[cfg(test)]
mod tests;

mod error;
pub(crate) use self::error::Error;

mod parse;
pub(crate) use self::parse::{
    Any, parse_any, parse_float, parse_signed, parse_signed_base, parse_unsigned,
    parse_unsigned_base, skip,
};

mod syntax;
pub(crate) use self::syntax::{Json, Json5};

mod traits;
pub(crate) use self::traits::{Float, Integer, Signed, Unsigned};

mod visit;
pub(crate) use self::visit::visit_any;
