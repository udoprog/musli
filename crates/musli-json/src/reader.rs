//! [`Parser`] trait and utilities used for musli-json.

pub(crate) mod integer;
mod parser;
mod slice_parser;
pub(crate) mod string;
#[cfg(test)]
mod tests;
mod token;

pub use self::parser::Parser;
pub(crate) use self::slice_parser::SliceParser;
pub(crate) use self::string::StringReference;
pub(crate) use self::token::Token;
