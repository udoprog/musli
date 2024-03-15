use core::mem;

use musli::de::{MapDecoder, MapPairsDecoder, SizeHint, StructDecoder, StructPairsDecoder};
use musli::Context;

use crate::error::{Error, ErrorKind};
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
        C: Context<Input = Error>,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.report(Error::new(ErrorKind::ExpectedOpenBrace(actual))));
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
        C: Context<Input = Error>,
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

impl<'de, P> MapDecoder<'de> for JsonObjectDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Entry<'this> = JsonObjectPairDecoder<P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn entry<C>(&mut self, cx: &C) -> Result<Option<Self::Entry<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.parse_map_key(cx)? {
            return Ok(None);
        }

        Ok(Some(JsonObjectPairDecoder::new(self.parser.borrow_mut())))
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(())
    }
}

impl<'de, P> MapPairsDecoder<'de> for JsonObjectDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type MapPairsKey<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;

    type MapPairsValue<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn map_pairs_key<C>(&mut self, cx: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.parse_map_key(cx)? {
            self.completed = true;
            return Ok(None);
        }

        Ok(Some(JsonKeyDecoder::new(self.parser.borrow_mut())))
    }

    #[inline]
    fn map_pairs_value<C>(&mut self, cx: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_map_pairs_value<C>(&mut self, cx: &C) -> Result<bool, C::Error>
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

    #[inline]
    fn end<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.completed {
            while self.parse_map_key(cx)? {
                JsonKeyDecoder::new(self.parser.borrow_mut()).skip_any(cx)?;
                self.skip_map_pairs_value(cx)?;
            }
        }

        Ok(())
    }
}

impl<'de, P> StructPairsDecoder<'de> for JsonObjectDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type FieldName<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;

    type FieldValue<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.parse_map_key(cx)? {
            return Err(cx.message("Expected map key, but found closing brace `}`"));
        }

        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn field_value<C>(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_field_value<C>(&mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapPairsDecoder::skip_map_pairs_value(self, cx)
    }

    #[inline]
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapPairsDecoder::end(self, cx)
    }
}

impl<'de, P> StructDecoder<'de> for JsonObjectDecoder<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    type Field<'this> = JsonObjectPairDecoder<P::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        MapDecoder::size_hint(self)
    }

    #[inline]
    fn field<C>(&mut self, cx: &C) -> Result<Option<Self::Field<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapDecoder::entry(self, cx)
    }

    #[inline]
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        MapDecoder::end(self, cx)
    }
}
