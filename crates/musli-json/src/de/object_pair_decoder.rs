use musli::de::{MapEntryDecoder, StructFieldDecoder};
use musli::Context;

use crate::parser::{Parser, Token};

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonObjectPairDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    parser: P,
}

impl<'a, P, C: ?Sized> JsonObjectPairDecoder<'a, P, C> {
    #[inline]
    pub(super) fn new(cx: &'a C, parser: P) -> Self {
        Self { cx, parser }
    }
}

impl<'a, 'de, P, C> MapEntryDecoder<'de> for JsonObjectPairDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeMapKey<'this> = JsonKeyDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeMapValue = JsonDecoder<'a, P, C>;

    #[inline]
    fn decode_map_key(&mut self) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_map_value(mut self) -> Result<Self::DecodeMapValue, C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser))
    }
}

impl<'a, 'de, P, C> StructFieldDecoder<'de> for JsonObjectPairDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeFieldName<'this> = JsonKeyDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeFieldValue = JsonDecoder<'a, P, C>;

    #[inline]
    fn decode_field_name(&mut self) -> Result<Self::DecodeFieldName<'_>, C::Error> {
        MapEntryDecoder::decode_map_key(self)
    }

    #[inline]
    fn decode_field_value(self) -> Result<Self::DecodeFieldValue, C::Error> {
        MapEntryDecoder::decode_map_value(self)
    }
}
