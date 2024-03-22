use core::marker::PhantomData;
use core::mem;

use musli::de::{PackDecoder, SequenceDecoder, SizeHint};
use musli::Context;

use crate::parser::{Parser, Token};

use super::JsonDecoder;

pub(crate) struct JsonSequenceDecoder<P, C: ?Sized> {
    len: Option<usize>,
    first: bool,
    parser: P,
    terminated: bool,
    _marker: PhantomData<C>,
}

impl<'de, P, C> JsonSequenceDecoder<P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    #[inline]
    pub(crate) fn new(cx: &C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error> {
        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBracket) {
            return Err(cx.message(format_args!("Expected opening bracket, was {actual}")));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            len,
            first: true,
            parser,
            terminated: false,
            _marker: PhantomData,
        })
    }
}

impl<'de, P, C> SequenceDecoder<'de> for JsonSequenceDecoder<P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeNext<'this> = JsonDecoder<P::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn decode_next(&mut self, cx: &C) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if self.terminated {
            return Ok(None);
        }

        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_value() {
                return Ok(Some(JsonDecoder::new(self.parser.borrow_mut())));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(cx, 1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(cx, 1)?;
                    self.terminated = true;
                    return Ok(None);
                }
                _ => {
                    return Err(cx.message(format_args!(
                        "Expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }
}

impl<'de, P, C> PackDecoder<'de> for JsonSequenceDecoder<P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeNext<'this> = JsonDecoder<P::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn decode_next(&mut self, cx: &C) -> Result<Self::DecodeNext<'_>, C::Error> {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_value() {
                return Ok(JsonDecoder::new(self.parser.borrow_mut()));
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(cx, 1)?;
                }
                Token::CloseBracket => {
                    self.parser.skip(cx, 1)?;
                    self.terminated = true;

                    return Err(
                        cx.message(format_args!("Encountered short array, but found {token}"))
                    );
                }
                _ => {
                    return Err(cx.message(format_args!(
                        "Expected value or closing bracket `]`, but found {token}"
                    )));
                }
            }
        }
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        if !self.terminated {
            let actual = self.parser.peek(cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(cx.message(format_args!("Expected closing bracket, was {actual}")));
            }

            self.parser.skip(cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}
