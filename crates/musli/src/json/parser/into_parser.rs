use core::mem::transmute;

use super::{MutSliceParser, Parser, SliceParser};

mod sealed {
    pub trait Sealed {}
    impl Sealed for &[u8] {}
    impl Sealed for &str {}
    impl Sealed for &mut &[u8] {}
    impl Sealed for &mut &str {}
}

/// Trait for types which can be converted into a [`Parser`].
pub trait IntoParser<'de>: self::sealed::Sealed {
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

impl<'a, 'de> IntoParser<'de> for &'a mut &'de [u8] {
    type Parser = MutSliceParser<'a, 'de>;

    #[inline]
    fn into_parser(self) -> Self::Parser {
        MutSliceParser::new(self)
    }
}

impl<'a, 'de> IntoParser<'de> for &'a mut &'de str {
    type Parser = MutSliceParser<'a, 'de>;

    #[inline]
    fn into_parser(self) -> Self::Parser {
        // SAFETY: Parsing ensures that the slice being processes keeps being valid UTF-8.
        MutSliceParser::new(unsafe { transmute(self) })
    }
}
