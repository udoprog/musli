use musli::context::Buffer;
use musli::Context;

use crate::error::{Error, ErrorKind};
use crate::reader::{Parser, StringReference, Token};

use lexical::parse_float_options::JSON;

const FORMAT: u128 = lexical::format::STANDARD;

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
    fn parse_string<'scratch, C, S>(
        &mut self,
        cx: &C,
        validate: bool,
        scratch: &'scratch mut S,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context<Input = Error>,
        S: ?Sized + Buffer,
    {
        let start = cx.mark();
        let actual = self.peek(cx)?;

        if !matches!(actual, Token::String) {
            return Err(cx.marked_report(start, Error::new(ErrorKind::ExpectedString(actual))));
        }

        self.skip(cx, 1)?;
        let out =
            crate::reader::string::parse_string_slice_reader(cx, self, validate, start, scratch);
        out
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: Context<Input = Error>,
    {
        let mut byte = [0];
        self.read(cx, &mut byte[..])?;
        Ok(byte[0])
    }

    #[inline]
    fn skip<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
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
    fn read<C>(&mut self, cx: &C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
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
    fn skip_whitespace<C>(&mut self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Error>,
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
    fn peek_byte<C>(&mut self, _: &C) -> Result<Option<u8>, C::Error>
    where
        C: Context<Input = Error>,
    {
        Ok(self.slice.get(self.index).copied())
    }

    fn parse_f32<C>(&mut self, cx: &C) -> Result<f32, C::Error>
    where
        C: Context<Input = Error>,
    {
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

    fn parse_f64<C>(&mut self, cx: &C) -> Result<f64, C::Error>
    where
        C: Context<Input = Error>,
    {
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
