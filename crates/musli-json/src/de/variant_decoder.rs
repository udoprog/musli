use musli::de::VariantDecoder;
use musli::Context;

use crate::error::ErrorKind;
use crate::reader::{Parser, Token};

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonVariantDecoder<P> {
    parser: P,
}

impl<'de, P> JsonVariantDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    pub(crate) fn new<C>(cx: &C, mut parser: P) -> Result<Self, C::Error>
    where
        C: Context,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.custom(ErrorKind::ExpectedOpenBrace(actual)));
        }

        parser.skip(cx, 1)?;
        Ok(Self { parser })
    }
}

impl<'de, C, P> VariantDecoder<'de, C> for JsonVariantDecoder<P>
where
    C: Context,
    P: Parser<'de>,
{
    type Tag<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type Variant<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn tag(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn variant(&mut self, cx: &C) -> Result<Self::Variant<'_>, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.custom(ErrorKind::ExpectedColon(actual)));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_variant(&mut self, cx: &C) -> Result<bool, C::Error> {
        let this = self.variant(cx)?;
        JsonDecoder::new(this.parser).skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(cx.custom(ErrorKind::ExpectedCloseBrace(actual)));
        }

        self.parser.skip(cx, 1)?;
        Ok(())
    }
}
