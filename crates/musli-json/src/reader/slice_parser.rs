use crate::reader::{ParseError, ParseErrorKind, Parser, Scratch, StringReference};

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
    fn parse_string<'scratch>(
        &mut self,
        scratch: &'scratch mut Scratch,
        validate: bool,
    ) -> Result<StringReference<'de, 'scratch>, ParseError> {
        scratch.bytes.clear();
        crate::reader::string::parse_string_slice_reader(self, scratch, validate)
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, ParseError> {
        let mut byte = [0];
        self.read(&mut byte[..])?;
        Ok(byte[0])
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), ParseError> {
        let outcome = self.index.wrapping_add(n);

        if outcome > self.slice.len() || outcome < self.index {
            return Err(ParseError::at(self.pos(), ParseErrorKind::BufferUnderflow));
        }

        self.index = outcome;
        Ok(())
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), ParseError> {
        let outcome = self.index.wrapping_add(buf.len());

        if outcome > self.slice.len() || outcome < self.index {
            return Err(ParseError::at(self.pos(), ParseErrorKind::BufferUnderflow));
        }

        buf.copy_from_slice(&self.slice[self.index..outcome]);
        self.index = outcome;
        Ok(())
    }

    #[inline]
    fn skip_whitespace(&mut self) -> Result<(), ParseError> {
        while self
            .slice
            .get(self.index)
            .map(u8::is_ascii_whitespace)
            .unwrap_or_default()
        {
            self.index += 1;
        }

        Ok(())
    }

    #[inline]
    fn pos(&self) -> u32 {
        self.index as u32
    }

    #[inline]
    fn peek_byte(&mut self) -> Result<Option<u8>, ParseError> {
        Ok(self.slice.get(self.index).copied())
    }
}
