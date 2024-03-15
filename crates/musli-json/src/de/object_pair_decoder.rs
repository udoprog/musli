use musli::de::{MapEntryDecoder, StructFieldDecoder};
use musli::Context;

use crate::reader::{Parser, Token};
use crate::Error;

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonObjectPairDecoder<P> {
    parser: P,
}

impl<P> JsonObjectPairDecoder<P> {
    #[inline]
    pub(super) fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<'de, P> MapEntryDecoder<'de> for JsonObjectPairDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type MapKey<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;

    type MapValue = JsonDecoder<P>;

    #[inline]
    fn map_key<C>(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn map_value<C>(mut self, cx: &C) -> Result<Self::MapValue, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser))
    }

    #[inline]
    fn skip_map_value<C>(mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        JsonDecoder::new(self.parser.borrow_mut()).skip_any(cx)?;
        Ok(true)
    }
}

impl<'de, P> StructFieldDecoder<'de> for JsonObjectPairDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type FieldName<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;

    type FieldValue = JsonDecoder<P>;

    #[inline]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEntryDecoder::map_key(self, cx)
    }

    #[inline]
    fn field_value<C>(self, cx: &C) -> Result<Self::FieldValue, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEntryDecoder::map_value(self, cx)
    }

    #[inline]
    fn skip_field_value<C>(self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapEntryDecoder::skip_map_value(self, cx)
    }
}
