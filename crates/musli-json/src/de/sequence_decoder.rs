use core::mem;

use musli::de::{PackDecoder, SequenceDecoder, SizeHint};
use musli::Context;

use crate::parser::{Parser, Token};

use super::JsonDecoder;

pub(crate) struct JsonSequenceDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    len: Option<usize>,
    first: bool,
    parser: P,
    terminated: bool,
}

impl<'a, 'de, P, C> JsonSequenceDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    #[inline]
    pub(crate) fn new(cx: &'a C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error> {
        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBracket) {
            return Err(cx.message(format_args!("Expected opening bracket, was {actual}")));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            cx,
            len,
            first: true,
            parser,
            terminated: false,
        })
    }
}

impl<'a, 'de, P, C> SequenceDecoder<'de> for JsonSequenceDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeNext<'this> = JsonDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if self.terminated {
            return Ok(None);
        }

        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(self.cx)?;

            if token.is_value() {
                return Ok(Some(JsonDecoder::new(self.cx, self.parser.borrow_mut())));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(self.cx, 1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(self.cx, 1)?;
                    self.terminated = true;
                    return Ok(None);
                }
                _ => {
                    return Err(self.cx.message(format_args!(
                        "Expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }
}

impl<'a, 'de, P, C> PackDecoder<'de> for JsonSequenceDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeNext<'this> = JsonDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(self.cx)?;

            if token.is_value() {
                return Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(self.cx, 1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(self.cx, 1)?;
                    self.terminated = true;

                    return Err(self
                        .cx
                        .message(format_args!("Encountered short array, but found {token}")));
                }
                _ => {
                    return Err(self.cx.message(format_args!(
                        "Expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end(mut self) -> Result<(), C::Error> {
        if !self.terminated {
            let actual = self.parser.peek(self.cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(self
                    .cx
                    .message(format_args!("Expected closing bracket, was {actual}")));
            }

            self.parser.skip(self.cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}
