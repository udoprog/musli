//! Helpers for integrating musli with I/O types like [std::io] and
//! [std::io::Write].
//!
//! The central function in this module is the [wrap] function which constructs
//! an adapter around an I/O type to work with musli.

#[cfg(feature = "std")]
use musli::context::Buffer;
#[cfg(feature = "std")]
use musli::Context;

/// Wrapper constructed with [wrap].
pub struct Wrap<T> {
    #[cfg_attr(not(feature = "std"), allow(unused))]
    inner: T,
}

/// Wrap a type so that it implements [Reader] or [Writer] as appropriate.
///
/// [Reader]: crate::reader::Reader
/// [Writer]: crate::writer::Writer
pub fn wrap<T>(inner: T) -> Wrap<T> {
    Wrap { inner }
}

#[cfg(feature = "std")]
impl<W> crate::writer::Writer for Wrap<W>
where
    W: std::io::Write,
{
    type Error = std::io::Error;
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_buffer<C, B>(&mut self, cx: &C, buffer: B) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
        B: Buffer,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, unsafe { buffer.as_slice() })
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.inner.write_all(bytes).map_err(|err| cx.report(err))?;
        cx.advance(bytes.len());
        Ok(())
    }
}
