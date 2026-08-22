use crate::Context;
use crate::alloc::Vec;
use crate::de::Visitor;
use crate::json::parser::number::{parse_any, parse_float};
use crate::json::parser::{StringReference, Token};

mod private {
    pub trait Sealed {}
    impl<const UTF8: bool> Sealed for crate::json::parser::SliceParser<'_, UTF8> {}
    impl<const UTF8: bool> Sealed for crate::json::parser::MutSliceParser<'_, '_, UTF8> {}
    impl<'de, R> Sealed for &mut R where R: ?Sized + super::Parser<'de> {}
}

/// Trait governing how JSON is parsed depending on the kind of buffer provided.
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

    /// The type of parser that can be cloned into.
    type TryClone: Parser<'de>;

    /// Reborrow the current parser.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Try to clone the parser.
    fn try_clone(&self) -> Option<Self::TryClone>;

    #[doc(hidden)]
    #[inline]
    fn parse_string<'scratch, C>(
        &mut self,
        cx: C,
        validate: bool,
        scratch: &'scratch mut Vec<u8, C::Allocator>,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context,
    {
        let start = cx.mark();
        let actual = self.lex(cx);

        if !matches!(actual, Token::String) {
            return Err(cx.message_at(&start, format_args!("Expected string, found {actual}")));
        }

        self.skip(cx, 1)?;
        self.parse_string_inner(cx, validate, scratch, &start)
    }

    #[doc(hidden)]
    fn parse_string_inner<'scratch, C>(
        &mut self,
        cx: C,
        validate: bool,
        scratch: &'scratch mut Vec<u8, C::Allocator>,
        start: &C::Mark,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context;

    /// Skip a string.
    #[doc(hidden)]
    fn skip_string_inner<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context;

    #[doc(hidden)]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        let mut byte = [0];
        self.read(cx, &mut byte[..])?;
        let [b] = byte;
        Ok(b)
    }

    #[doc(hidden)]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context;

    #[doc(hidden)]
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context;

    /// Skip over whitespace.
    #[doc(hidden)]
    fn skip_whitespace<C>(&mut self, cx: C)
    where
        C: Context;

    #[doc(hidden)]
    fn consume_while<C>(&mut self, cx: C, m: fn(u8) -> bool) -> Result<usize, C::Error>
    where
        C: Context,
    {
        let mut c = 0;

        while let Some(b) = self.peek() {
            if !m(b) {
                return Ok(c);
            }

            c += 1;
            self.skip(cx, 1)?;
        }

        Ok(c)
    }

    /// Peek the next byte.
    #[doc(hidden)]
    fn peek(&mut self) -> Option<u8>;

    /// Access the not yet consumed portion of the input.
    ///
    /// This permits parsing routines which are hot, such as number parsing, to
    /// work over a plain slice instead of going through the parser one byte at
    /// a time.
    #[doc(hidden)]
    fn remaining(&self) -> &[u8];

    /// Test if the input has been fully consumed, ignoring trailing whitespace
    /// since that is not part of a JSON value.
    #[doc(hidden)]
    fn is_exhausted<C>(&mut self, cx: C) -> bool
    where
        C: Context,
    {
        self.skip_whitespace(cx);
        self.peek().is_none()
    }

    #[doc(hidden)]
    fn lex<C>(&mut self, cx: C) -> Token
    where
        C: Context,
    {
        self.skip_whitespace(cx);

        let Some(b) = self.peek() else {
            return Token::Eof;
        };

        Token::from_byte(b)
    }

    /// Parse a 32-bit floating point number.
    #[inline]
    fn parse_f32<C>(&mut self, cx: C) -> Result<f32, C::Error>
    where
        C: Context,
    {
        parse_float(cx, self)
    }

    /// Parse a 64-bit floating point number.
    #[inline]
    fn parse_f64<C>(&mut self, cx: C) -> Result<f64, C::Error>
    where
        C: Context,
    {
        parse_float(cx, self)
    }

    #[doc(hidden)]
    fn parse_exact<C>(&mut self, cx: C, exact: &str) -> Result<(), C::Error>
    where
        C: Context,
    {
        debug_assert!(exact.len() <= 5);

        let mark = cx.mark();

        let mut bytes = [0u8; 8];
        let bytes = &mut bytes[..exact.len()];

        self.read(cx, bytes)?;

        if bytes != exact.as_bytes() {
            return Err(cx.message_at(&mark, format_args!("Expected `{exact}`")));
        }

        Ok(())
    }

    /// Parse an unknown number and try to coerce it into the best fit type
    /// through [Visitor].
    #[doc(hidden)]
    fn parse_number<C, V>(&mut self, cx: C, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: Visitor<'de, C, Error = C::Error, Allocator = C::Allocator>,
    {
        let any = parse_any(cx, self)?;
        crate::number::visit_any(cx, any, visitor)
    }
}

impl<'de, P> Parser<'de> for &mut P
where
    P: ?Sized + Parser<'de>,
{
    type Mut<'this>
        = P::Mut<'this>
    where
        Self: 'this;

    type TryClone = P::TryClone;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        (**self).borrow_mut()
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        (**self).try_clone()
    }

    #[inline]
    fn parse_string_inner<'scratch, C>(
        &mut self,
        cx: C,
        validate: bool,
        scratch: &'scratch mut Vec<u8, C::Allocator>,
        start: &C::Mark,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context,
    {
        (**self).parse_string_inner(cx, validate, scratch, start)
    }

    #[inline]
    fn skip_string_inner<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).skip_string_inner(cx)
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        (**self).read_byte(cx)
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        (**self).peek()
    }

    #[inline]
    fn remaining(&self) -> &[u8] {
        (**self).remaining()
    }

    #[inline]
    fn lex<C>(&mut self, cx: C) -> Token
    where
        C: Context,
    {
        (**self).lex(cx)
    }

    #[inline]
    fn skip_whitespace<C>(&mut self, cx: C)
    where
        C: Context,
    {
        (**self).skip_whitespace(cx);
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).skip(cx, n)
    }

    #[inline]
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).read(cx, buf)
    }

    #[inline]
    fn parse_f32<C>(&mut self, cx: C) -> Result<f32, C::Error>
    where
        C: Context,
    {
        (**self).parse_f32(cx)
    }

    #[inline]
    fn parse_f64<C>(&mut self, cx: C) -> Result<f64, C::Error>
    where
        C: Context,
    {
        (**self).parse_f64(cx)
    }
}
