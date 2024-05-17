//! [`Parser`] trait and utilities used for musli-json.

#![allow(clippy::module_inception)]

#[cfg(test)]
mod tests;

pub(crate) mod integer;

mod into_parser;
pub use self::into_parser::IntoParser;

mod parser;
pub use self::parser::Parser;

mod slice_parser;
pub(crate) use self::slice_parser::SliceParser;

mod mut_slice_parser;
pub(crate) use self::mut_slice_parser::MutSliceParser;

pub(crate) mod string;
pub(crate) use self::string::StringReference;

mod token;
pub(crate) use self::token::Token;
