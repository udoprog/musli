use crate::Context;
use crate::alloc::Vec;
use crate::json::parser::{Parser, SliceParser, StringReference};
use crate::reader::SliceUnderflow;

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
pub struct MutSliceParser<'a, 'de, const UTF8: bool = false> {
    slice: &'a mut &'de [u8],
}

impl<'a, 'de> MutSliceParser<'a, 'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub(crate) fn new(slice: &'a mut &'de [u8]) -> Self {
        Self { slice }
    }
}

impl<'a, 'de> MutSliceParser<'a, 'de, true> {
    /// Construct a new instance around a slice which is known to be valid
    /// UTF-8.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the slice contains valid UTF-8. Parsing
    /// keeps it valid UTF-8, since every update to the slice goes through
    /// [`MutSliceParser::set_tail`], which never leaves it starting in the
    /// middle of a character.
    #[inline]
    pub(crate) unsafe fn new_utf8(slice: &'a mut &'de [u8]) -> Self {
        Self { slice }
    }
}

impl<'a, 'de, const UTF8: bool> MutSliceParser<'a, 'de, UTF8> {
    /// Update the slice to `tail`, which must be a suffix of the current
    /// slice.
    ///
    /// When the slice is UTF-8, error paths can stop in the middle of a
    /// multibyte character, such as when a literal like `true` is compared
    /// against the next four bytes. Since the slice might have been
    /// transmuted from a `&mut &str`, it is then advanced to the next
    /// character boundary so that it stays valid UTF-8.
    #[inline(always)]
    fn set_tail<C>(&mut self, cx: C, tail: &'de [u8])
    where
        C: Context,
    {
        if UTF8 && matches!(tail.first(), Some(&b) if is_continuation(b)) {
            self.set_tail_realign(cx, tail);
        } else {
            *self.slice = tail;
        }
    }

    #[cold]
    #[inline(never)]
    fn set_tail_realign<C>(&mut self, cx: C, tail: &'de [u8])
    where
        C: Context,
    {
        let n = tail
            .iter()
            .position(|&b| !is_continuation(b))
            .unwrap_or(tail.len());

        *self.slice = &tail[n..];
        cx.advance(n);
    }
}

/// Test if `b` is a UTF-8 continuation byte, which is never at a character
/// boundary.
#[inline(always)]
fn is_continuation(b: u8) -> bool {
    (b as i8) < -0x40
}

impl<'a, 'de, const UTF8: bool> Parser<'de> for MutSliceParser<'a, 'de, UTF8> {
    type Mut<'this>
        = MutSliceParser<'this, 'de, UTF8>
    where
        Self: 'this;

    type TryClone = SliceParser<'de, UTF8>;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        MutSliceParser { slice: self.slice }
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(SliceParser {
            slice: self.slice,
            index: 0,
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
        let slice: &'de [u8] = self.slice;
        let mut access = SliceAccess::<_, UTF8>::new(cx, slice, 0);
        let out = access.parse_string(validate, start, scratch);
        let tail = &slice[access.index..];

        // A successfully parsed string ends after its closing quote, so only
        // errors can stop inside of a character.
        if out.is_ok() {
            *self.slice = tail;
        } else {
            self.set_tail(cx, tail);
        }

        out
    }

    #[inline]
    fn skip_string_inner<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        let slice: &'de [u8] = self.slice;
        let mut access = SliceAccess::<_, UTF8>::new(cx, slice, 0);
        let out = access.skip_string();
        let tail = &slice[access.index..];

        // A successfully skipped string ends after its closing quote, so only
        // errors can stop inside of a character.
        if out.is_ok() {
            *self.slice = tail;
        } else {
            self.set_tail(cx, tail);
        }

        out
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        let slice: &'de [u8] = self.slice;

        let Some((&b, tail)) = slice.split_first() else {
            return Err(cx.custom(SliceUnderflow::new(1, 0)));
        };

        cx.advance(1);
        self.set_tail(cx, tail);
        Ok(b)
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        if self.slice.len() < n {
            return Err(cx.custom(SliceUnderflow::new(n, self.slice.len())));
        }

        let slice: &'de [u8] = self.slice;
        cx.advance(n);
        self.set_tail(cx, &slice[n..]);
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

        let slice: &'de [u8] = self.slice;
        let (head, tail) = slice.split_at(buf.len());
        buf.copy_from_slice(head);
        cx.advance(buf.len());
        self.set_tail(cx, tail);
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

    #[inline]
    fn remaining(&self) -> &[u8] {
        self.slice
    }
}
