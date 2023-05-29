use musli::de::NumberVisitor;
use musli::Context;

use crate::error::{Error, ErrorKind};
use crate::reader::integer::decode_signed_full;
use crate::reader::{string, Scratch, StringReference, Token};

mod private {
    pub trait Sealed {}
    impl<'de> Sealed for crate::reader::SliceParser<'de> {}
    impl<'de, R> Sealed for &mut R where R: ?Sized + super::Parser<'de> {}
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
    fn parse_string<'scratch, 'buf, C>(
        &mut self,
        cx: &mut C,
        scratch: &'scratch mut Scratch,
        validate: bool,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context<'buf, Input = Error>;

    #[doc(hidden)]
    fn read_byte<'buf, C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Error>;

    #[doc(hidden)]
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>;

    #[doc(hidden)]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>;

    #[doc(hidden)]
    fn pos(&self) -> u32;

    /// Skip over whitespace.
    #[doc(hidden)]
    fn skip_whitespace<'buf, C>(&mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>;

    /// Peek the next byte.
    #[doc(hidden)]
    fn peek_byte<'buf, C>(&mut self, cx: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Error>;

    #[doc(hidden)]
    fn consume_while<'buf, C>(&mut self, cx: &mut C, m: fn(u8) -> bool) -> Result<usize, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        let mut c = 0;

        while let Some(b) = self.peek_byte(cx)? {
            if !m(b) {
                return Ok(c);
            }

            c += 1;
            self.skip(cx, 1)?;
        }

        Ok(c)
    }

    #[doc(hidden)]
    fn peek<'buf, C>(&mut self, cx: &mut C) -> Result<Token, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        self.skip_whitespace(cx)?;

        let b = match self.peek_byte(cx)? {
            Some(b) => b,
            None => return Ok(Token::Eof),
        };

        Ok(Token::from_byte(b))
    }

    /// Parse a 32-bit floating point number.
    fn parse_f32<'buf, C>(&mut self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<'buf, Input = Error>;

    /// Parse a 64-bit floating point number.
    fn parse_f64<'buf, C>(&mut self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<'buf, Input = Error>;

    #[doc(hidden)]
    fn parse_hex_escape<'buf, C>(&mut self, cx: &mut C) -> Result<u16, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        let mut n = 0;
        let start = cx.mark();

        for _ in 0..4 {
            match string::decode_hex_val(self.read_byte(cx)?) {
                None => {
                    return Err(cx.marked_report(start, Error::new(ErrorKind::InvalidEscape)));
                }
                Some(val) => {
                    n = (n << 4) + val;
                }
            }
        }

        Ok(n)
    }

    #[doc(hidden)]
    fn parse_exact<'buf, C, const N: usize>(
        &mut self,
        cx: &mut C,
        exact: [u8; N],
        err: Error,
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        let mark = cx.mark();

        let mut bytes = [0u8; N];
        self.read(cx, &mut bytes)?;

        if bytes != exact {
            return Err(cx.marked_report(mark, err));
        }

        Ok(())
    }

    /// Parse an unknown number and try to coerce it into the best fit type
    /// through [NumberVisitor].
    #[doc(hidden)]
    fn parse_number<'buf, C, V>(&mut self, cx: &mut C, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Error>,
        V: NumberVisitor<'de, 'buf, C>,
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
    type Mut<'this> = P::Mut<'this> where Self: 'this;

    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        (**self).borrow_mut()
    }

    #[inline]
    fn parse_string<'scratch, 'buf, C>(
        &mut self,
        cx: &mut C,
        scratch: &'scratch mut Scratch,
        validate: bool,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).parse_string(cx, scratch, validate)
    }

    #[inline]
    fn read_byte<'buf, C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).read_byte(cx)
    }

    #[inline]
    fn peek<'buf, C>(&mut self, cx: &mut C) -> Result<Token, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).peek(cx)
    }

    #[inline]
    fn pos(&self) -> u32 {
        (**self).pos()
    }

    #[inline]
    fn skip_whitespace<'buf, C>(&mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).skip_whitespace(cx)
    }

    #[inline]
    fn peek_byte<'buf, C>(&mut self, cx: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).peek_byte(cx)
    }

    #[inline]
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).skip(cx, n)
    }

    #[inline]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).read(cx, buf)
    }

    #[inline]
    fn parse_f32<'buf, C>(&mut self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).parse_f32(cx)
    }

    #[inline]
    fn parse_f64<'buf, C>(&mut self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        (**self).parse_f64(cx)
    }
}
