use musli::Context;

use crate::reader::{ParseError, ParseErrorKind, Parser, Scratch, StringReference, Token};

/// An efficient [Reader] wrapper around a slice.
pub struct SliceParser<'de> {
    pub(crate) slice: &'de [u8],
    pub(crate) index: usize,
}

impl<'de> SliceParser<'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub fn new(slice: &'de [u8]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'de> Parser<'de> for SliceParser<'de> {
    type Mut<'this> = &'this mut SliceParser<'de> where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn parse_string<'scratch, C>(
        &mut self,
        cx: &mut C,
        scratch: &'scratch mut Scratch,
        validate: bool,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context<Input = ParseError>,
    {
        let start = self.pos();
        let actual = self.peek(cx)?;

        if !matches!(actual, Token::String) {
            return Err(cx.report(ParseError::at(
                start,
                ParseErrorKind::ExpectedString(actual),
            )));
        }

        self.skip(cx, 1)?;
        scratch.bytes.clear();
        crate::reader::string::parse_string_slice_reader(cx, self, scratch, validate, start)
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<Input = ParseError>,
    {
        let mut byte = [0];
        self.read(cx, &mut byte[..])?;
        Ok(byte[0])
    }

    #[inline]
    fn skip<C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<Input = ParseError>,
    {
        let outcome = self.index.wrapping_add(n);

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.report(ParseError::at(self.pos(), ParseErrorKind::BufferUnderflow)));
        }

        self.index = outcome;
        Ok(())
    }

    #[inline]
    fn read<C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<Input = ParseError>,
    {
        let outcome = self.index.wrapping_add(buf.len());

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.report(ParseError::at(self.pos(), ParseErrorKind::BufferUnderflow)));
        }

        buf.copy_from_slice(&self.slice[self.index..outcome]);
        self.index = outcome;
        Ok(())
    }

    #[inline]
    fn skip_whitespace<C>(&mut self, _: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = ParseError>,
    {
        while matches!(
            self.slice.get(self.index),
            Some(b' ' | b'\n' | b'\t' | b'\r')
        ) {
            self.index += 1;
        }

        Ok(())
    }

    #[inline]
    fn pos(&self) -> u32 {
        self.index as u32
    }

    #[inline]
    fn peek_byte<C>(&mut self, _: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<Input = ParseError>,
    {
        Ok(self.slice.get(self.index).copied())
    }

    #[inline]
    fn parse_f32<C>(&mut self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<Input = ParseError>,
    {
        use lexical::parse_float_options::JSON;
        const FORMAT: u128 = lexical::format::STANDARD;

        let (value, read) = match lexical::parse_partial_with_options::<f32, _, FORMAT>(
            &self.slice[self.index..],
            &JSON,
        ) {
            Ok(out) => out,
            Err(error) => {
                return Err(cx.report(ParseError::at(
                    self.pos(),
                    ParseErrorKind::ParseFloat(error),
                )));
            }
        };

        self.index += read;
        Ok(value)
    }

    #[inline]
    fn parse_f64<C>(&mut self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<Input = ParseError>,
    {
        use lexical::parse_float_options::JSON;
        const FORMAT: u128 = lexical::format::STANDARD;

        let (value, read) = match lexical::parse_partial_with_options::<f64, _, FORMAT>(
            &self.slice[self.index..],
            &JSON,
        ) {
            Ok(out) => out,
            Err(error) => {
                return Err(cx.report(ParseError::at(
                    self.pos(),
                    ParseErrorKind::ParseFloat(error),
                )));
            }
        };

        self.index += read;
        Ok(value)
    }
}
