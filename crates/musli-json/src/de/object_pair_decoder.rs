use core::marker::PhantomData;

use musli::de::{MapEntryDecoder, StructFieldDecoder};
use musli::Context;

use crate::parser::{Parser, Token};

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonObjectPairDecoder<P, C: ?Sized> {
    parser: P,
    _marker: PhantomData<C>,
}

impl<P, C: ?Sized> JsonObjectPairDecoder<P, C> {
    #[inline]
    pub(super) fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: PhantomData,
        }
    }
}

impl<'de, P, C> MapEntryDecoder<'de> for JsonObjectPairDecoder<P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeMapKey<'this> = JsonKeyDecoder<P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeMapValue = JsonDecoder<P, C>;

    #[inline]
    fn decode_map_key(&mut self, _: &C) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_map_value(mut self, cx: &C) -> Result<Self::DecodeMapValue, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser))
    }

    #[inline]
    fn skip_map_value(mut self, cx: &C) -> Result<bool, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        JsonDecoder::new(self.parser.borrow_mut()).skip_any(cx)?;
        Ok(true)
    }
}

impl<'de, P, C> StructFieldDecoder<'de> for JsonObjectPairDecoder<P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeFieldName<'this> = JsonKeyDecoder<P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeFieldValue = JsonDecoder<P, C>;

    #[inline]
    fn decode_field_name(&mut self, cx: &C) -> Result<Self::DecodeFieldName<'_>, C::Error> {
        MapEntryDecoder::decode_map_key(self, cx)
    }

    #[inline]
    fn decode_field_value(self, cx: &C) -> Result<Self::DecodeFieldValue, C::Error> {
        MapEntryDecoder::decode_map_value(self, cx)
    }

    #[inline]
    fn skip_field_value(self, cx: &C) -> Result<bool, C::Error> {
        MapEntryDecoder::skip_map_value(self, cx)
    }
}
