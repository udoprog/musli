use crate::reader::{string, ParseError, ParseErrorKind, Scratch, StringReference, Token};

mod private {
    pub trait Sealed {}
    impl<'de> Sealed for crate::reader::SliceParser<'de> {}
    impl<'de, R> Sealed for &mut R where R: super::Parser<'de> {}
}

/// Parser trait for this crate.
pub trait Parser<'de>: private::Sealed {
    /// Reborrowed type.
    ///
    /// Why oh why would we want to do this over having a simple `&'this mut P`?
    ///
    /// We want to avoid recursive types, which will blow up the compiler. And
    /// the above is a typical example of when that can go wrong. This ensures
    /// that each call to `borrow_mut` dereferences the [Parser] at each step to
    /// avoid constructing a large muted type, like `&mut &mut &mut
    /// SliceParser<'de>`.
    type Mut<'this>: Parser<'de>
    where
        Self: 'this;

    /// Reborrow the current parser.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Must parse the string from the input buffer and validate that it is
    /// valid UTF-8.
    #[doc(hidden)]
    fn parse_string<'scratch>(
        &mut self,
        scratch: &'scratch mut Scratch,
        validate: bool,
    ) -> Result<StringReference<'de, 'scratch>, ParseError>;

    #[doc(hidden)]
    fn read_byte(&mut self) -> Result<u8, ParseError>;

    #[doc(hidden)]
    fn skip(&mut self, n: usize) -> Result<(), ParseError>;

    #[doc(hidden)]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), ParseError>;

    #[doc(hidden)]
    fn peek(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace()?;

        let b = match self.peek_byte()? {
            Some(b) => b,
            None => return Ok(Token::Eof),
        };

        Ok(Token::from_byte(b))
    }

    #[doc(hidden)]
    fn parse_hex_escape(&mut self) -> Result<u16, ParseError> {
        let mut n = 0;
        let start = self.pos();

        for _ in 0..4 {
            match string::decode_hex_val(self.read_byte()?) {
                None => {
                    return Err(ParseError::spanned(
                        start,
                        self.pos(),
                        ParseErrorKind::InvalidEscape,
                    ))
                }
                Some(val) => {
                    n = (n << 4) + val;
                }
            }
        }

        Ok(n)
    }

    #[doc(hidden)]
    fn parse_exact<const N: usize>(
        &mut self,
        exact: [u8; N],
        err: impl FnOnce(u32) -> ParseError,
    ) -> Result<(), ParseError> {
        let pos = self.pos();

        let mut bytes = [0u8; N];
        self.read(&mut bytes)?;

        if bytes != exact {
            return Err(err(pos));
        }

        Ok(())
    }

    #[doc(hidden)]
    fn pos(&self) -> u32;

    /// Skip over whitespace.
    #[doc(hidden)]
    fn skip_whitespace(&mut self) -> Result<(), ParseError>;

    /// Peek the next byte.
    #[doc(hidden)]
    fn peek_byte(&mut self) -> Result<Option<u8>, ParseError>;
}

impl<'de, P> Parser<'de> for &mut P
where
    P: Parser<'de>,
{
    type Mut<'this> = P::Mut<'this> where Self: 'this;

    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        (**self).borrow_mut()
    }

    #[inline]
    fn parse_string<'scratch>(
        &mut self,
        scratch: &'scratch mut Scratch,
        validate: bool,
    ) -> Result<StringReference<'de, 'scratch>, ParseError> {
        (**self).parse_string(scratch, validate)
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, ParseError> {
        (**self).read_byte()
    }

    #[inline]
    fn peek(&mut self) -> Result<Token, ParseError> {
        (**self).peek()
    }

    #[inline]
    fn pos(&self) -> u32 {
        (**self).pos()
    }

    #[inline]
    fn skip_whitespace(&mut self) -> Result<(), ParseError> {
        (**self).skip_whitespace()
    }

    #[inline]
    fn peek_byte(&mut self) -> Result<Option<u8>, ParseError> {
        (**self).peek_byte()
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), ParseError> {
        (**self).skip(n)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), ParseError> {
        (**self).read(buf)
    }
}
