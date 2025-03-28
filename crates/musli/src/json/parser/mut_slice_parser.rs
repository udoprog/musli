use crate::alloc::Vec;
use crate::json::error::ErrorMessage;
use crate::json::parser::{Parser, StringReference};
use crate::reader::SliceUnderflow;
use crate::Context;

use super::string::SliceAccess;

/// An efficient [`Parser`] wrapper around a mutable slice.
///
/// As the slice is being parsed, this keeps the referenced slice up-to-date.
///
/// # Implementation Note
///
/// This MUST ensure that the underlying slice remains valid UTF-8, if it is
/// valid UTF-8. We transmute a `&'a mut &'de str` in order to construct this
/// efficiently.
#[repr(transparent)]
pub struct MutSliceParser<'a, 'de> {
    slice: &'a mut &'de [u8],
}

impl<'a, 'de> MutSliceParser<'a, 'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub(crate) fn new(slice: &'a mut &'de [u8]) -> Self {
        Self { slice }
    }
}

impl<'a, 'de> Parser<'de> for MutSliceParser<'a, 'de> {
    type Mut<'this>
        = MutSliceParser<'this, 'de>
    where
        Self: 'this;

    type TryClone = MutSliceParser<'a, 'de>;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        MutSliceParser::new(self.slice)
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        None
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
        let mut access = SliceAccess::new(cx, self.slice, 0);
        let out = access.parse_string(validate, start, scratch);
        *self.slice = &self.slice[access.index..];
        out
    }

    #[inline]
    fn skip_string_inner<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        let mut access = SliceAccess::new(cx, self.slice, 0);
        let out = access.skip_string();
        *self.slice = &self.slice[access.index..];
        out
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        if self.slice.len() < n {
            return Err(cx.custom(SliceUnderflow::new(n, self.slice.len())));
        }

        *self.slice = &self.slice[n..];
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        if self.slice.len() < buf.len() {
            return Err(cx.custom(SliceUnderflow::new(buf.len(), self.slice.len())));
        }

        let (head, tail) = self.slice.split_at(buf.len());
        *self.slice = tail;
        buf.copy_from_slice(head);
        cx.advance(buf.len());
        Ok(())
    }

    #[inline]
    fn skip_whitespace<C>(&mut self, cx: C)
    where
        C: Context,
    {
        let n = 0;

        let n = 'out: {
            for (index, &b) in self.slice[n..].iter().enumerate() {
                if matches!(b, b' ' | b'\n' | b'\t' | b'\r') {
                    continue;
                }

                break 'out index;
            }

            self.slice.len()
        };

        *self.slice = &self.slice[n..];
        cx.advance(n);
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        self.slice.first().copied()
    }

    fn parse_f32<C>(&mut self, cx: C) -> Result<f32, C::Error>
    where
        C: Context,
    {
        let Some((value, read)) = crate::dec2flt::dec2flt(self.slice) else {
            return Err(cx.message(ErrorMessage::ParseFloat));
        };

        *self.slice = &self.slice[read..];
        cx.advance(read);
        Ok(value)
    }

    fn parse_f64<C>(&mut self, cx: C) -> Result<f64, C::Error>
    where
        C: Context,
    {
        let Some((value, read)) = crate::dec2flt::dec2flt(self.slice) else {
            return Err(cx.message(ErrorMessage::ParseFloat));
        };

        *self.slice = &self.slice[read..];
        cx.advance(read);
        Ok(value)
    }
}
