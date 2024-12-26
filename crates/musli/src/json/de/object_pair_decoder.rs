use crate::de::EntryDecoder;
use crate::json::parser::{Parser, Token};
use crate::Context;

use super::{JsonDecoder, JsonKeyDecoder};

pub(crate) struct JsonObjectPairDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    parser: P,
}

impl<'a, P, C: ?Sized> JsonObjectPairDecoder<'a, P, C> {
    #[inline(always)]
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
    type DecodeKey<'this>
        = JsonKeyDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeValue = JsonDecoder<'a, P, C>;

    #[inline(always)]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, C::Error> {
        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline(always)]
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
