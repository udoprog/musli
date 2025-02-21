use core::marker::PhantomData;

use crate::de::VariantDecoder;
use crate::json::parser::{Parser, Token};
use crate::Context;

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonVariantDecoder<P, C, M> {
    cx: C,
    parser: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonVariantDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, mut parser: P) -> Result<Self, C::Error> {
        let actual = parser.lex(cx);

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.message(format_args!("Expected open brace, was {actual}")));
        }

        parser.skip(cx, 1)?;
        Ok(Self {
            cx,
            parser,
            _marker: PhantomData,
        })
    }

    #[inline]
    pub(super) fn end(mut self) -> Result<(), C::Error> {
        let actual = self.parser.lex(self.cx);

        if !matches!(actual, Token::CloseBrace) {
            return Err(self.cx.message(format_args!(
                "Expected closing brace for variant, was {actual}"
            )));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(())
    }
}

impl<'de, P, C, M> VariantDecoder<'de> for JsonVariantDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type DecodeTag<'this>
        = JsonKeyDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = JsonDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        let actual = self.parser.lex(self.cx);

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()))
    }
}
