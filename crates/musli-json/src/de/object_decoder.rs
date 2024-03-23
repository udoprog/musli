use core::mem;

use musli::de::{MapDecoder, MapEntriesDecoder, SizeHint, StructDecoder, StructFieldsDecoder};
use musli::Context;

use crate::parser::{Parser, Token};

use super::{JsonDecoder, JsonKeyDecoder, JsonObjectPairDecoder};

pub(crate) struct JsonObjectDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    first: bool,
    completed: bool,
    len: Option<usize>,
    parser: P,
}

impl<'a, 'de, P, C> JsonObjectDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    #[inline]
    pub(super) fn new(cx: &'a C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error> {
        parser.skip_whitespace(cx)?;

        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.message(format_args!("Expected opening brace, was {actual}")));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            cx,
            first: true,
            completed: false,
            len,
            parser,
        })
    }

    fn parse_map_key(&mut self) -> Result<bool, C::Error> {
        let first = mem::take(&mut self.first);

        loop {
            let token = self.parser.peek(self.cx)?;

            if token.is_string() {
                return Ok(true);
            }

            match token {
                Token::Comma if !first => {
                    self.parser.skip(self.cx, 1)?;
                }
                Token::CloseBrace => {
                    self.parser.skip(self.cx, 1)?;
                    return Ok(false);
                }
                token => {
                    return Err(self.cx.message(format_args!(
                        "Expected value, or closing brace `}}` but found {token:?}"
                    )));
                }
            }
        }
    }
}

#[musli::map_decoder]
impl<'a, 'de, P, C> MapDecoder<'de> for JsonObjectDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeEntry<'this> = JsonObjectPairDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type IntoMapEntries = Self;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
    }

    #[inline]
    fn into_map_entries(self) -> Result<Self::IntoMapEntries, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        if !self.parse_map_key()? {
            return Ok(None);
        }

        Ok(Some(JsonObjectPairDecoder::new(
            self.cx,
            self.parser.borrow_mut(),
        )))
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'a, 'de, P, C> MapEntriesDecoder<'de> for JsonObjectDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeMapEntryKey<'this> = JsonKeyDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeMapEntryValue<'this> = JsonDecoder<'a, P::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn decode_map_entry_key(&mut self) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        if self.completed {
            return Ok(None);
        }

        if !self.parse_map_key()? {
            self.completed = true;
            return Ok(None);
        }

        Ok(Some(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut())))
    }

    #[inline]
    fn decode_map_entry_value(&mut self) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_map_entry_value(&mut self) -> Result<bool, C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        JsonDecoder::new(self.cx, self.parser.borrow_mut()).skip_any()?;
        Ok(true)
    }
}

#[musli::struct_decoder]
impl<'a, 'de, P, C> StructDecoder<'de> for JsonObjectDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeField<'this> = JsonObjectPairDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type IntoStructFields = Self;

    #[inline]
    fn cx(&self) -> &Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        MapDecoder::size_hint(self)
    }

    #[inline]
    fn into_struct_fields(self) -> Result<Self::IntoStructFields, C::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_field(&mut self) -> Result<Option<Self::DecodeField<'_>>, C::Error> {
        MapDecoder::decode_entry(self)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        MapDecoder::end(self)
    }
}

impl<'a, 'de, P, C> StructFieldsDecoder<'de> for JsonObjectDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeStructFieldName<'this> = JsonKeyDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeStructFieldValue<'this> = JsonDecoder<'a, P::Mut<'this>, C> where Self: 'this;

    #[inline]
    fn decode_struct_field_name(&mut self) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        if !self.parse_map_key()? {
            return Err(self
                .cx
                .message("Expected map key, but found closing brace `}`"));
        }

        Ok(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn decode_struct_field_value(&mut self) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn skip_struct_field_value(&mut self) -> Result<bool, C::Error> {
        MapEntriesDecoder::skip_map_entry_value(self)
    }

    #[inline]
    fn end(self) -> Result<(), C::Error> {
        MapEntriesDecoder::end(self)
    }
}
