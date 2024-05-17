use super::{Parser, SliceParser};

/// Trait for types which can be converted into a [`Parser`].
pub trait IntoParser<'de> {
    /// The parser type being converted into.
    type Parser: Parser<'de>;

    /// Convert into a parser.
    fn into_parser(self) -> Self::Parser;
}

impl<'de> IntoParser<'de> for &'de [u8] {
    type Parser = SliceParser<'de>;

    #[inline]
    fn into_parser(self) -> Self::Parser {
        SliceParser::new(self)
    }
}

impl<'de> IntoParser<'de> for &'de str {
    type Parser = SliceParser<'de>;

    #[inline]
    fn into_parser(self) -> Self::Parser {
        SliceParser::new(self.as_bytes())
    }
}
