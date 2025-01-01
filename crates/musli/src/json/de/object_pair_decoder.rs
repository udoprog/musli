use crate::de::EntryDecoder;
use crate::json::parser::{Parser, Token};
use crate::Context;

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonObjectPairDecoder<P, C> {
    cx: C,
    parser: P,
}

impl<P, C> JsonObjectPairDecoder<P, C> {
    #[inline]
    pub(super) fn new(cx: C, parser: P) -> Self {
        Self { cx, parser }
    }
}

impl<'de, P, C> EntryDecoder<'de> for JsonObjectPairDecoder<P, C>
where
    P: Parser<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeKey<'this>
        = JsonKeyDecoder<P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeValue = JsonDecoder<P, C>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_value(mut self) -> Result<Self::DecodeValue, C::Error> {
        let actual = self.parser.lex(self.cx);

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser))
    }
}
