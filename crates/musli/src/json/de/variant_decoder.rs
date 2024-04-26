use crate::de::VariantDecoder;
use crate::json::parser::{Parser, Token};
use crate::Context;

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonVariantDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    parser: P,
}

impl<'a, 'de, P, C> JsonVariantDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &'a C, mut parser: P) -> Result<Self, C::Error> {
        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.message(format_args!("Expected open brace, was {actual}")));
        }

        parser.skip(cx, 1)?;
        Ok(Self { cx, parser })
    }

    #[inline]
    pub(super) fn end(mut self) -> Result<(), C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(self.cx.message(format_args!(
                "Expected closing brace for variant, was {actual}"
            )));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(())
    }
}

impl<'a, 'de, P, C> VariantDecoder<'de> for JsonVariantDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeTag<'this> = JsonKeyDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeValue<'this> = JsonDecoder<'a, P::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()))
    }
}
