use core::fmt;
use core::ops::Deref;
use core::str;

use crate::Allocator;

use super::{AllocError, Vec};

/// Wrapper around a buffer that is guaranteed to be a valid utf-8 string.
pub struct String<A>
where
    A: Allocator,
{
    buf: Vec<u8, A>,
}

#[cfg(feature = "alloc")]
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<String<crate::alloc::System>>();
};

/// Collect a string into a string buffer.
#[inline]
pub(crate) fn collect_string<A, T>(alloc: A, value: T) -> Result<String<A>, AllocError>
where
    A: Allocator,
    T: fmt::Display,
{
    use core::fmt::Write;

    let mut string = String::new_in(alloc);

    if write!(string, "{value}").is_err() {
        return Err(AllocError);
    }

    Ok(string)
}

impl<A> String<A>
where
    A: Allocator,
{
    /// Construct a new string buffer in the provided allocator.
    #[inline]
    pub(crate) fn new_in(alloc: A) -> Self {
        Self {
            buf: Vec::new_in(alloc),
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        // SAFETY: Interactions ensure that data is valid utf-8.
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }

    #[inline]
    fn push(&mut self, c: char) -> Result<(), AllocError> {
        self.buf
            .extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes())
    }

    #[inline]
    fn push_str(&mut self, s: &str) -> Result<(), AllocError> {
        self.buf.extend_from_slice(s.as_bytes())
    }
}

impl<A> fmt::Write for String<A>
where
    A: Allocator,
{
    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c).map_err(|_| fmt::Error)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
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
