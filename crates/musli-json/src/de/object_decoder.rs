use core::mem;

use musli::de::{MapDecoder, MapEntriesDecoder, SizeHint, StructDecoder, StructFieldsDecoder};
use musli::Context;

use crate::parser::{Parser, Token};

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
        C: ?Sized + Context,
    {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.message(format_args!("Expected opening brace, was {actual}")));
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
        C: ?Sized + Context,
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
    C: ?Sized + Context,
    P: Parser<'de>,
{
    type DecodeEntry<'this> = JsonObjectPairDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type IntoMapEntries = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn into_map_entries(self, _: &C) -> Result<Self::IntoMapEntries, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_entry(&mut self, cx: &C) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
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

impl<'de, C, P> MapEntriesDecoder<'de, C> for JsonObjectDecoder<P>
where
    C: ?Sized + Context,
    P: Parser<'de>,
{
    type DecodeMapEntryKey<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type DecodeMapEntryValue<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn decode_map_entry_key(
        &mut self,
        cx: &C,
    ) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        if self.completed {
            return Ok(None);
        }

        if !self.parse_map_key(cx)? {
            self.completed = true;
            return Ok(None);
        }

        Ok(Some(JsonKeyDecoder::new(self.parser.borrow_mut())))
    }

    #[inline]
    fn decode_map_entry_value(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_map_entry_value(&mut self, cx: &C) -> Result<bool, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        JsonDecoder::new(self.parser.borrow_mut()).skip_any(cx)?;
        Ok(true)
    }
}

#[musli::struct_decoder]
impl<'de, C, P> StructDecoder<'de, C> for JsonObjectDecoder<P>
where
    C: ?Sized + Context,
    P: Parser<'de>,
{
    type DecodeField<'this> = JsonObjectPairDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type IntoStructFields = Self;

    #[inline]
    fn size_hint(&self, cx: &C) -> SizeHint {
        MapDecoder::size_hint(self, cx)
    }

    #[inline]
    fn into_struct_fields(self, _: &C) -> Result<Self::IntoStructFields, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_field(&mut self, cx: &C) -> Result<Option<Self::DecodeField<'_>>, C::Error> {
        MapDecoder::decode_entry(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapDecoder::end(self, cx)
    }
}

impl<'de, C, P> StructFieldsDecoder<'de, C> for JsonObjectDecoder<P>
where
    C: ?Sized + Context,
    P: Parser<'de>,
{
    type DecodeStructFieldName<'this> = JsonKeyDecoder<P::Mut<'this>>
    where
        Self: 'this;
    type DecodeStructFieldValue<'this> = JsonDecoder<P::Mut<'this>> where Self: 'this;

    #[inline]
    fn decode_struct_field_name(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        if !self.parse_map_key(cx)? {
            return Err(cx.message("Expected map key, but found closing brace `}`"));
        }

        Ok(JsonKeyDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_struct_field_value(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        let actual = self.parser.peek(cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(cx.message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(cx, 1)?;
        Ok(JsonDecoder::new(self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_struct_field_value(&mut self, cx: &C) -> Result<bool, C::Error> {
        MapEntriesDecoder::skip_map_entry_value(self, cx)
    }

    #[inline]
    fn end(self, cx: &C) -> Result<(), C::Error> {
        MapEntriesDecoder::end(self, cx)
    }
}
