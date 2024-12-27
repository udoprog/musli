use core::fmt;
use core::ops::Deref;
use core::str;

use crate::fixed::CapacityError;
use crate::Context;

use super::{Allocator, Vec};

/// Wrapper around a buffer that is guaranteed to be a valid utf-8 string.
pub struct String<A>
where
    A: Allocator,
{
    buf: Vec<u8, A>,
}

/// Collect a string into a string buffer.
pub(crate) fn collect_string<'cx, C, T>(
    cx: &'cx C,
    value: &T,
) -> Result<String<C::Allocator>, C::Error>
where
    C: 'cx + ?Sized + Context,
    T: ?Sized + fmt::Display,
{
    use core::fmt::Write;

    let mut string = String::new_in(cx.alloc());

    if write!(string, "{value}").is_err() {
        return Err(cx.message("Failed to write to string"));
    }

    Ok(string)
}

impl<A> String<A>
where
    A: Allocator,
{
    /// Construct a new string buffer in the provided allocator.
    pub(crate) fn new_in(alloc: A) -> Self {
        Self {
            buf: Vec::new_in(alloc),
        }
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

impl<A> fmt::Write for String<A>
where
    A: Allocator,
{
    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.try_push(c).map_err(|_| fmt::Error)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_push_str(s).map_err(|_| fmt::Error)
    }
}

impl<A> Deref for String<A>
where
    A: Allocator,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }
}

impl<A> fmt::Display for String<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<A> fmt::Debug for String<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<A> AsRef<str> for String<A>
where
    A: Allocator,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
