use core::mem::{replace, take};

use crate::de::{Decoder, EntriesDecoder, EntryDecoder, MapDecoder, SizeHint};
use crate::json::parser::{Parser, Token};
use crate::Context;

use super::{JsonDecoder, JsonKeyDecoder, JsonObjectPairDecoder};

#[must_use = "Must call skip_object_remaining to complete decoding"]
pub(crate) struct JsonObjectDecoder<P, C> {
    cx: C,
    first: bool,
    len: Option<usize>,
    parser: P,
    finalized: bool,
}

impl<'de, P, C> JsonObjectDecoder<P, C>
where
    P: Parser<'de>,
    C: Context,
{
    pub(super) fn new_in(
        cx: C,
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
    pub(super) fn new(cx: C, len: Option<usize>, mut parser: P) -> Result<Self, C::Error> {
        let actual = parser.lex(cx);

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
            let token = self.parser.lex(self.cx);

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
            entry.decode_key()?.skip()?;
            entry.decode_value()?.skip()?;
        }

        let actual = self.parser.lex(self.cx);

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

impl<'de, P, C> MapDecoder<'de> for JsonObjectDecoder<P, C>
where
    P: Parser<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeEntry<'this>
        = JsonObjectPairDecoder<P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = JsonObjectDecoder<P::Mut<'this>, C>
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

impl<'de, P, C> EntriesDecoder<'de> for JsonObjectDecoder<P, C>
where
    P: Parser<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeEntryKey<'this>
        = JsonKeyDecoder<P::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = JsonDecoder<P::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, C::Error> {
        if !self.parse_map_key()? {
            return Ok(None);
        }

        Ok(Some(JsonKeyDecoder::new(self.cx, self.parser.borrow_mut())))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, C::Error> {
        let actual = self.parser.lex(self.cx);

        if !matches!(actual, Token::Colon) {
            return Err(self
                .cx
                .message(format_args!("Expected colon `:`, was {actual}")));
        }

        self.parser.skip(self.cx, 1)?;
        Ok(JsonDecoder::new(self.cx, self.parser.borrow_mut()))
    }

    #[inline]
    fn end_entries(self) -> Result<(), C::Error> {
        self.skip_object_remaining()
    }
}
