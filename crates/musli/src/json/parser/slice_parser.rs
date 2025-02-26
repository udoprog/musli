use crate::alloc::Vec;
use crate::json::error::ErrorMessage;
use crate::json::parser::{Parser, StringReference};
use crate::reader::SliceUnderflow;
use crate::Context;

use super::string::SliceAccess;

/// An efficient [`Parser`] wrapper around a slice.
pub struct SliceParser<'de> {
    pub(crate) slice: &'de [u8],
    pub(crate) index: usize,
}

impl<'de> SliceParser<'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub(crate) fn new(slice: &'de [u8]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'de> Parser<'de> for SliceParser<'de> {
    type Mut<'this>
        = &'this mut SliceParser<'de>
    where
        Self: 'this;

    type TryClone = SliceParser<'de>;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(SliceParser {
            slice: self.slice,
            index: self.index,
        })
    }

    #[inline]
    fn parse_string_inner<'scratch, C>(
        &mut self,
        cx: C,
        validate: bool,
        scratch: &'scratch mut Vec<u8, C::Allocator>,
        start: &C::Mark,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: Context,
    {
        let mut access = SliceAccess::new(cx, self.slice, self.index);
        let out = access.parse_string(validate, start, scratch);
        self.index = access.index;
        out
    }

    #[inline]
    fn skip_string_inner<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        let mut access = SliceAccess::new(cx, self.slice, self.index);
        let out = access.skip_string();
        self.index = access.index;
        out
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        let outcome = self.index.wrapping_add(n);

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.custom(SliceUnderflow::new(n, self.slice.len() - self.index)));
        }

        self.index = outcome;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        let outcome = self.index.wrapping_add(buf.len());

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.custom(SliceUnderflow::new(
                buf.len(),
                self.slice.len() - self.index,
            )));
        }

        buf.copy_from_slice(&self.slice[self.index..outcome]);
        self.index = outcome;
        cx.advance(buf.len());
        Ok(())
    }

    #[inline]
    fn skip_whitespace<C>(&mut self, cx: C)
    where
        C: Context,
    {
        while matches!(
            self.slice.get(self.index),
            Some(b' ' | b'\n' | b'\t' | b'\r')
        ) {
            self.index = self.index.wrapping_add(1);
            cx.advance(1);
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        self.slice.get(self.index).copied()
    }

    fn parse_f32<C>(&mut self, cx: C) -> Result<f32, C::Error>
    where
        C: Context,
    {
        let Some((value, read)) = crate::dec2flt::dec2flt(&self.slice[self.index..]) else {
            return Err(cx.message(ErrorMessage::ParseFloat));
        };

        self.index += read;
        cx.advance(read);
        Ok(value)
    }

    fn parse_f64<C>(&mut self, cx: C) -> Result<f64, C::Error>
    where
        C: Context,
    {
        let Some((value, read)) = crate::dec2flt::dec2flt(&self.slice[self.index..]) else {
            return Err(cx.message(ErrorMessage::ParseFloat));
        };

        self.index += read;
        cx.advance(read);
        Ok(value)
    }
}
