use core::marker::PhantomData;
use core::mem;

use crate::de::{Decoder, SequenceDecoder, SizeHint};
use crate::json::parser::{Parser, Token};
use crate::Context;

use super::JsonDecoder;

#[must_use = "Must call skip_sequence_remaining"]
pub(crate) struct JsonSequenceDecoder<P, C, M> {
    cx: C,
    len: Option<usize>,
    first: bool,
    parser: P,
    finalized: bool,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonSequenceDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error> {
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
            _marker: PhantomData,
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

impl<'de, P, C, M> SequenceDecoder<'de> for JsonSequenceDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type DecodeNext<'this>
        = JsonDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

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
