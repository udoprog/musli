use core::mem::{replace, take};

use musli::de::{Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, SizeHint};
use musli::Context;

use crate::parser::{Parser, Token};

use super::{JsonDecoder, JsonKeyDecoder, JsonObjectPairDecoder};

#[must_use = "Must call skip_object_remaining to complete decoding"]
pub(crate) struct JsonObjectDecoder<'a, P, C: ?Sized> {
    cx: &'a C,
    first: bool,
    len: Option<usize>,
    parser: P,
    finalized: bool,
}

impl<'a, 'de, P, C> JsonObjectDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    pub(super) fn new_in(
        cx: &'a C,
        first: bool,
        len: Option<usize>,
        parser: P,
    ) -> Result<Self, C::Error> {
        Ok(Self {
            cx,
            first,
            len,
            parser,
            finalized: false,
        })
    }

    #[inline]
    pub(super) fn new(cx: &'a C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error> {
        let actual = parser.peek(cx)?;

        if !matches!(actual, Token::OpenBrace) {
            return Err(cx.message(format_args!("Expected opening brace, was {actual}")));
        }

        parser.skip(cx, 1)?;

        Ok(Self {
            cx,
            first: true,
            len,
            parser,
            finalized: false,
        })
    }

    fn parse_map_key(&mut self) -> Result<bool, C::Error> {
        if self.finalized {
            return Ok(false);
        }

        let first = take(&mut self.first);

        loop {
            let token = self.parser.peek(self.cx)?;

            match token {
                Token::String => {
                    return Ok(true);
                }
                Token::Comma if !first => {
                    self.parser.skip(self.cx, 1)?;
                }
                Token::CloseBrace => {
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

    /// Parse end of object.
    #[inline]
    pub(super) fn skip_object_remaining(mut self) -> Result<(), C::Error> {
        // Someone else is responsible for finalizing this decoder.
        if self.finalized {
            return Ok(());
        }

        while let Some(mut entry) = self.decode_entry()? {
            entry.decode_map_key()?.skip()?;
            entry.decode_map_value()?.skip()?;
        }

        let actual = self.parser.peek(self.cx)?;

        if !matches!(actual, Token::CloseBrace) {
            return Err(self
                .cx
                .message(format_args!("Expected closing brace `}}`, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        self.finalized = true;
        Ok(())
    }
}

impl<'a, 'de, P, C> MapDecoder<'de> for JsonObjectDecoder<'a, P, C>
where
    P: Parser<'de>,
    C: ?Sized + Context,
{
    type Cx = C;
    type DecodeEntry<'this> = JsonObjectPairDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this> = JsonObjectDecoder<'a, P::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::from(self.len)
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
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, <Self::Cx as Context>::Error> {
        if replace(&mut self.finalized, true) {
            return Err(self
                .cx
                .message("Cannot decode remaining entries after finalizing"));
        }

        JsonObjectDecoder::new_in(self.cx, self.first, self.len, self.parser.borrow_mut())
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
        if !self.parse_map_key()? {
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
    fn end_map_entries(self) -> Result<(), C::Error> {
        self.skip_object_remaining()
    }
}
