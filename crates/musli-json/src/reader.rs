//! [Parser] trait and utilities used for musli-json.

pub(crate) mod integer;
mod parse_error;
mod parser;
mod scratch;
mod slice_parser;
pub(crate) mod string;
#[cfg(test)]
mod tests;
mod token;

pub use self::parse_error::ParseError;
pub use self::parser::Parser;
pub use self::scratch::Scratch;
pub(crate) use self::slice_parser::SliceParser;
pub use self::string::StringReference;
pub(crate) use self::token::Token;
