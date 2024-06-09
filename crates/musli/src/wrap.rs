//! Wrapper for integrating musli with I/O types like [std::io].
//!
//! The main methods in this module is the [`wrap`] function which constructs an
//! adapter around an I/O type to work with musli.

#[cfg(feature = "std")]
use crate::buf::BufVec;
#[cfg(feature = "std")]
use crate::Context;

/// Wrap a type so that it implements [`Reader`] and [`Writer`].
///
/// See [`wrap()`].
///
/// [`Reader`]: crate::reader::Reader
/// [`Writer`]: crate::writer::Writer
pub struct Wrap<T> {
    #[cfg_attr(not(feature = "std"), allow(unused))]
    inner: T,
}

/// Wrap a type so that it implements [`Reader`] and [`Writer`].
///
/// [`Reader`]: crate::reader::Reader
/// [`Writer`]: crate::writer::Writer
pub fn wrap<T>(inner: T) -> Wrap<T> {
    Wrap { inner }
}

#[cfg(feature = "std")]
impl<W> crate::writer::Writer for Wrap<W>
where
    W: std::io::Write,
{
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: &C, buffer: BufVec<'_, C::Allocator, u8>) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, buffer.as_slice())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.inner.write_all(bytes).map_err(cx.map())?;
        cx.advance(bytes.len());
        Ok(())
    }
}
