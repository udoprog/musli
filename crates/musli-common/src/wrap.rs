#[cfg(feature = "std")]
use musli::{Buf, Context};

/// Wrap a type so that it implements [Reader] and [Writer].
///
/// [Reader]: crate::reader::Reader
/// [Writer]: crate::writer::Writer
///
/// See [`wrap()`].
pub struct Wrap<T> {
    #[cfg_attr(not(feature = "std"), allow(unused))]
    inner: T,
}

/// Wrap a type so that it implements [Reader] and [Writer].
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
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_buffer<C, B>(&mut self, cx: &C, buffer: B) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
        B: Buf,
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
