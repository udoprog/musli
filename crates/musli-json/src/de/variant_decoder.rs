use musli::de::{Decoder, VariantDecoder};
use musli::Context;

use crate::parser::{Parser, Token};

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
    pub(crate) fn new(cx: &'a C, mut parser: P) -> Result<Self, C::Error> {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.message(format_args!("Expected open brace, was {actual}")));
        }

        parser.skip(cx, 1)?;

        Ok(Self { cx, parser })
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
    type DecodeVariant<'this> = JsonDecoder<'a, P::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeVariant<'_>, C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_value(&mut self) -> Result<bool, C::Error> {
        self.decode_value()?.skip()?;
        Ok(true)
    }

    #[inline]
    fn end(mut self) -> Result<(), C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(self
                .cx
                .message(format_args!("Expected closing brace, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(())
    }
}
