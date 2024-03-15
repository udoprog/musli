use musli::de::VariantDecoder;
use musli::Context;

use crate::error::{Error, ErrorKind};
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
        C: Context<Input = Error>,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedOpenBrace(actual))));
        }

        parser.skip(cx, 1)?;
        Ok(Self { parser })
    }
}

impl<'de, P> VariantDecoder<'de> for JsonVariantDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Tag<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;

    type Variant<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn variant<C>(&mut self, cx: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedColon(actual))));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_variant<C>(&mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let this = self.variant(cx)?;
        JsonDecoder::new(this.parser).skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedCloseBrace(actual))));
        }

        self.parser.skip(cx, 1)?;
        Ok(())
    }
}
