use core::mem;

use musli::de::{PackDecoder, SequenceDecoder, SizeHint};
use musli::Context;

use crate::error::{Error, ErrorKind};
use crate::reader::{Parser, Token};

use super::JsonDecoder;

pub(crate) struct JsonSequenceDecoder<P> {
    len: Option<usize>,
    first: bool,
    parser: P,
    terminated: bool,
}

impl<'de, P> JsonSequenceDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    pub(crate) fn new<C>(cx: &C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error>
    where
        C: Context<Input = Error>,
    {
        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBracket) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedOpenBracket(actual))));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            len,
            first: true,
            parser,
            terminated: false,
        })
    }
}

impl<'de, P> SequenceDecoder<'de> for JsonSequenceDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Decoder<'this> = JsonDecoder<P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn next<C>(&mut self, cx: &C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
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

    #[inline]
    fn end<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.terminated {
            let actual = self.parser.peek(cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(cx.report(Error::new(ErrorKind::ExpectedCloseBracket(actual))));
            }

            self.parser.skip(cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}

impl<'de, P> PackDecoder<'de> for JsonSequenceDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Decoder<'this> = JsonDecoder<P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn next<C>(&mut self, cx: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
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
    fn end<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.terminated {
            let actual = self.parser.peek(cx)?;

            if !matches!(actual, Token::CloseBracket) {
                return Err(cx.report(Error::new(ErrorKind::ExpectedCloseBracket(actual))));
            }

            self.parser.skip(cx, 1)?;
            self.terminated = true;
        }

        Ok(())
    }
}
