use musli::Context;

use crate::error::{Error, ErrorKind};
use crate::reader::{Parser, Scratch, StringReference, Token};

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
    fn parse_string<'scratch, 'buf, C>(
        &mut self,
        cx: &mut C,
        scratch: &'scratch mut Scratch,
        validate: bool,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        let start = cx.mark();
        let actual = self.peek(cx)?;

        if !matches!(actual, Token::String) {
            return Err(cx.marked_report(start, Error::new(ErrorKind::ExpectedString(actual))));
        }

        self.skip(cx, 1)?;
        scratch.bytes.clear();
        let out =
            crate::reader::string::parse_string_slice_reader(cx, self, scratch, validate, start);
        out
    }

    #[inline]
    fn read_byte<'buf, C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        let mut byte = [0];
        self.read(cx, &mut byte[..])?;
        Ok(byte[0])
    }

    #[inline]
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        let outcome = self.index.wrapping_add(n);

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.report(Error::new(ErrorKind::BufferUnderflow)));
        }

        self.index = outcome;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        let outcome = self.index.wrapping_add(buf.len());

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.report(Error::new(ErrorKind::BufferUnderflow)));
        }

        buf.copy_from_slice(&self.slice[self.index..outcome]);
        self.index = outcome;
        cx.advance(buf.len());
        Ok(())
    }

    #[inline]
    fn skip_whitespace<'buf, C>(&mut self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        while matches!(
            self.slice.get(self.index),
            Some(b' ' | b'\n' | b'\t' | b'\r')
        ) {
            self.index = self.index.wrapping_add(1);
            cx.advance(1);
        }

        Ok(())
    }

    #[inline]
    fn pos(&self) -> u32 {
        self.index as u32
    }

    #[inline]
    fn peek_byte<'buf, C>(&mut self, _: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        Ok(self.slice.get(self.index).copied())
    }

    #[inline]
    fn parse_f32<'buf, C>(&mut self, cx: &mut C) -> Result<f32, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        use lexical::parse_float_options::JSON;
        const FORMAT: u128 = lexical::format::STANDARD;

        let (value, read) = match lexical::parse_partial_with_options::<f32, _, FORMAT>(
            &self.slice[self.index..],
            &JSON,
        ) {
            Ok(out) => out,
            Err(error) => {
                return Err(cx.report(Error::new(ErrorKind::ParseFloat(error))));
            }
        };

        self.index += read;
        cx.advance(read);
        Ok(value)
    }

    #[inline]
    fn parse_f64<'buf, C>(&mut self, cx: &mut C) -> Result<f64, C::Error>
    where
        C: Context<'buf, Input = Error>,
    {
        use lexical::parse_float_options::JSON;
        const FORMAT: u128 = lexical::format::STANDARD;

        let (value, read) = match lexical::parse_partial_with_options::<f64, _, FORMAT>(
            &self.slice[self.index..],
            &JSON,
        ) {
            Ok(out) => out,
            Err(error) => {
                return Err(cx.report(Error::new(ErrorKind::ParseFloat(error))));
            }
        };

        self.index += read;
        cx.advance(read);
        Ok(value)
    }
}
