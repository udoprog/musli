use crate::alloc::Vec;
use crate::de::Visitor;
use crate::json::parser::integer::decode_signed_full;
use crate::json::parser::{StringReference, Token};
use crate::Context;

mod private {
    pub trait Sealed {}
    impl Sealed for crate::json::parser::SliceParser<'_> {}
    impl Sealed for crate::json::parser::MutSliceParser<'_, '_> {}
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

    /// Reborrow the current parser.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Must parse the string from the input buffer and validate that it is
    /// valid UTF-8.
    #[doc(hidden)]
    fn parse_string<'scratch, C>(
        &mut self,
        cx: C,
        validate: bool,
        scratch: &'scratch mut Vec<u8, C::Allocator>,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context;

    /// Skip a string.
    #[doc(hidden)]
    fn skip_string<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context;

    #[doc(hidden)]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        let mut byte = [0];
        self.read(cx, &mut byte[..])?;
        Ok(byte[0])
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
    fn parse_f32<C>(&mut self, cx: C) -> Result<f32, C::Error>
    where
        C: Context;

    /// Parse a 64-bit floating point number.
    fn parse_f64<C>(&mut self, cx: C) -> Result<f64, C::Error>
    where
        C: Context;

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
            return Err(cx.marked_message(&mark, format_args!("Expected `{exact}`")));
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
        let signed = decode_signed_full::<i128, _, _>(cx, self)?;

        if signed.is_negative {
            let value = match signed.compute() {
                Ok(value) => value,
                Err(..) => {
                    let value = signed.compute_float();
                    return visitor.visit_f64(cx, value);
                }
            };

            if value >= i8::MIN as i128 && value <= i8::MAX as i128 {
                return visitor.visit_i8(cx, value as i8);
            }

            if value >= i16::MIN as i128 && value <= i16::MAX as i128 {
                return visitor.visit_i16(cx, value as i16);
            }

            if value >= i32::MIN as i128 && value <= i32::MAX as i128 {
                return visitor.visit_i32(cx, value as i32);
            }

            if value >= i64::MIN as i128 && value <= i64::MAX as i128 {
                return visitor.visit_i64(cx, value as i64);
            }

            if value >= isize::MIN as i128 && value <= isize::MAX as i128 {
                return visitor.visit_isize(cx, value as isize);
            }

            visitor.visit_i128(cx, value)
        } else {
            let value = match signed.unsigned.compute() {
                Ok(value) => value,
                Err(..) => {
                    let value = signed.unsigned.compute_float();
                    return visitor.visit_f64(cx, value);
                }
            };

            if value <= u8::MAX as u128 {
                return visitor.visit_u8(cx, value as u8);
            }

            if value <= u16::MAX as u128 {
                return visitor.visit_u16(cx, value as u16);
            }

            if value <= u32::MAX as u128 {
                return visitor.visit_u32(cx, value as u32);
            }

            if value <= u64::MAX as u128 {
                return visitor.visit_u64(cx, value as u64);
            }

            if value <= usize::MAX as u128 {
                return visitor.visit_usize(cx, value as usize);
            }

            visitor.visit_u128(cx, value)
        }
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

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        (**self).borrow_mut()
    }

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
        (**self).parse_string(cx, validate, scratch)
    }

    #[inline]
    fn skip_string<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).skip_string(cx)
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
