use musli::de::EntryDecoder;
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

impl<'a, 'de, P, C> EntryDecoder<'de> for JsonObjectPairDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeKey<'this> = JsonKeyDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeValue = JsonDecoder<'a, P, C>;

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_value(mut self) -> Result<Self::DecodeValue, C::Error> {
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
