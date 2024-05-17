use core::mem;

use crate::de::{Decoder, SequenceDecoder, SizeHint};
use crate::json::parser::{Parser, Token};
use crate::Context;

use super::JsonDecoder;

#[must_use = "Must call skip_sequence_remaining"]
pub(crate) struct JsonSequenceDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    len: Option<usize>,
    first: bool,
    parser: P,
    finalized: bool,
}

impl<'a, 'de, P, C> JsonSequenceDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &'a C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error> {
        let actual = parser.lex(cx);

        if !matches!(actual, Token::OpenBracket) {
            return Err(cx.message(format_args!("Expected opening bracket, was {actual}")));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            cx,
            len,
            first: true,
            parser,
            finalized: false,
        })
    }

    fn parse_next_value(&mut self) -> Result<bool, C::Error> {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.lex(self.cx);

            if token.is_value() {
                return Ok(true);
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(self.cx, 1)?;
                }
                Token::CloseBracket => {
                    return Ok(false);
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
    pub(super) fn skip_sequence_remaining(mut self) -> Result<(), C::Error> {
        if self.finalized {
            return Ok(());
        }

        while let Some(decoder) = SequenceDecoder::try_decode_next(&mut self)? {
            decoder.skip()?;
        }

        let actual = self.parser.lex(self.cx);

        if !matches!(actual, Token::CloseBracket) {
            return Err(self
                .cx
                .message(format_args!("Expected closing bracket, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        self.finalized = true;
        Ok(())
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
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if !self.parse_next_value()? {
            return Ok(None);
        }

        Ok(Some(JsonDecoder::new(self.cx, self.parser.borrow_mut())))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        if !self.parse_next_value()? {
            return Err(self.cx.message(format_args!("Encountered short array")));
        }

        Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()))
    }
}
