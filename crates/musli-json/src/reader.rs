//! [Parser] trait and utilities used for musli-json.

pub(crate) mod integer;
mod parser;
mod slice_parser;
pub(crate) mod string;
#[cfg(test)]
mod tests;
mod token;

pub(crate) use self::parser::Parser;
pub(crate) use self::slice_parser::SliceParser;
pub use self::string::StringReference;
pub(crate) use self::token::Token;
