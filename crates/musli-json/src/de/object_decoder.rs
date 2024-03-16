use core::mem;

use musli::de::{MapDecoder, MapPairsDecoder, SizeHint, StructDecoder, StructPairsDecoder};
use musli::Context;

use crate::error::ErrorKind;
use crate::reader::{Parser, Token};

use super::{JsonDecoder, JsonKeyDecoder, JsonObjectPairDecoder};

pub(crate) struct JsonObjectDecoder<P> {
    first: bool,
    completed: bool,
    len: Option<usize>,
    parser: P,
}

impl<'de, P> JsonObjectDecoder<P>
where
    P: Parser<'de>,
{
    #[inline]
    pub(super) fn new<C>(cx: &C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error>
    where
        C: Context,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.custom(ErrorKind::ExpectedOpenBrace(actual)));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            first: true,
            completed: false,
            len,
            parser,
        })
    }

    fn parse_map_key<C>(&mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context,
    {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(cx)?;

            if token.is_string() {
                return Ok(true);
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(cx, 1)?;
                }
                Token::CloseBrace => {
                    self.parser.skip(cx, 1)?;
                    return Ok(false);
                }
                token => {
                    return Err(cx.message(format_args!(
                        "Expected value, or closing brace `}}` but found {token:?}"
                    )));
                }
            }
        }
    }
}

#[musli::map_decoder]
impl<'de, C, P> MapDecoder<'de, C> for JsonObjectDecoder<P>
where
    C: Context,
    P: Parser<'de>,
{
    type Entry<'this> = JsonObjectPairDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type MapPairs = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn into_map_pairs(self, _: &C) -> Result<Self::MapPairs, C::Error> {
        Ok(self)
    }

    #[inline]
    fn entry(&mut self, cx: &C) -> Result<Option<Self::Entry<'_>>, C::Error> {
        if !self.parse_map_key(cx)? {
            return Ok(None);
        }

        Ok(Some(JsonObjectPairDecoder::new(self.parser.borrow_mut())))
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, C, P> MapPairsDecoder<'de, C> for JsonObjectDecoder<P>
where
    C: Context,
    P: Parser<'de>,
{
    type MapPairsKey<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type MapPairsValue<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, cx: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error> {
        if !self.parse_map_key(cx)? {
            self.completed = true;
            return Ok(None);
        }

        Ok(Some(JsonKeyDecoder::new(self.parser.borrow_mut())))
    }

    #[inline]
    fn map_pairs_value(&mut self, cx: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_map_pairs_value(&mut self, cx: &C) -> Result<bool, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        JsonDecoder::new(self.parser.borrow_mut()).skip_any(cx)?;
        Ok(true)
    }

    #[inline]
    fn end(mut self, cx: &C) -> Result<(), C::Error> {
        if !self.completed {
            while self.parse_map_key(cx)? {
                JsonKeyDecoder::new(self.parser.borrow_mut()).skip_any(cx)?;
                self.skip_map_pairs_value(cx)?;
            }
        }

        Ok(())
    }
}

#[musli::struct_decoder]
impl<'de, C, P> StructDecoder<'de, C> for JsonObjectDecoder<P>
where
    C: Context,
    P: Parser<'de>,
{
    type Field<'this> = JsonObjectPairDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type StructPairs = Self;

    #[inline]
    fn size_hint(&self, cx: &C) -> SizeHint {
        MapDecoder::size_hint(self, cx)
    }

    #[inline]
    fn into_struct_pairs(self, _: &C) -> Result<Self::StructPairs, C::Error> {
        Ok(self)
    }

    #[inline]
    fn field(&mut self, cx: &C) -> Result<Option<Self::Field<'_>>, C::Error> {
        MapDecoder::entry(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapDecoder::end(self, cx)
    }
}

impl<'de, C, P> StructPairsDecoder<'de, C> for JsonObjectDecoder<P>
where
    C: Context,
    P: Parser<'de>,
{
    type FieldName<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type FieldValue<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error> {
        if !self.parse_map_key(cx)? {
            return Err(cx.message("Expected map key, but found closing brace `}`"));
        }

        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn field_value(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_field_value(&mut self, cx: &C) -> Result<bool, C::Error> {
        MapPairsDecoder::skip_map_pairs_value(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapPairsDecoder::end(self, cx)
    }
}
