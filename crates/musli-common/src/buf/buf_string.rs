use core::fmt;
use core::ops::Deref;
use core::str;

use musli::Buf;

use crate::fixed::CapacityError;

/// A string wrapped around a context buffer.
pub struct BufString<B> {
    buf: B,
}

impl<B> BufString<B>
where
    B: Buf,
{
    /// Construct a new fixed string.
    pub const fn new(buf: B) -> BufString<B> {
        BufString { buf }
    }

    fn as_str(&self) -> &str {
        // SAFETY: Interactions ensure that data is valid utf-8.
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }

    fn try_push(&mut self, c: char) -> Result<(), CapacityError> {
        if !self.buf.write(c.encode_utf8(&mut [0; 4]).as_bytes()) {
            return Err(CapacityError);
        }

        Ok(())
    }

    fn try_push_str(&mut self, s: &str) -> Result<(), CapacityError> {
        if !self.buf.write(s.as_bytes()) {
            return Err(CapacityError);
        }

        Ok(())
    }
}

impl<B> fmt::Write for BufString<B>
where
    B: Buf,
{
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.try_push(c).map_err(|_| fmt::Error)
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_push_str(s).map_err(|_| fmt::Error)
    }
}

impl<B> Deref for BufString<B>
where
    B: Buf,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }
}

impl<B> fmt::Display for BufString<B>
where
    B: Buf,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<B> AsRef<str> for BufString<B>
where
    B: Buf,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
