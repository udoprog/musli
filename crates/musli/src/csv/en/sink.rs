use crate::alloc::Vec;
use crate::{Context, Writer};

/// A writer which either writes to an underlying writer, or discards what is
/// being written.
///
/// This is used when a record is encoded more than once, such as when a header
/// row is being written, to ignore the parts of the record which are not
/// relevant to the current pass.
pub(crate) enum Sink<W> {
    /// Write to the underlying writer.
    Real(W),
    /// Discard anything written.
    Discard,
}

impl<W> Writer for Sink<W>
where
    W: Writer,
{
    type Ok = ();
    type Mut<'this>
        = Sink<W::Mut<'this>>
    where
        Self: 'this;

    #[inline]
    fn finish<C>(&mut self, _: C) -> Result<Self::Ok, C::Error>
    where
        C: Context,
    {
        Ok(())
    }

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        match self {
            Sink::Real(writer) => Sink::Real(writer.borrow_mut()),
            Sink::Discard => Sink::Discard,
        }
    }

    #[inline]
    fn extend<C>(&mut self, cx: C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: Context,
    {
        match self {
            Sink::Real(writer) => writer.extend(cx, buffer),
            Sink::Discard => Ok(()),
        }
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        match self {
            Sink::Real(writer) => writer.write_bytes(cx, bytes),
            Sink::Discard => Ok(()),
        }
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: C, b: u8) -> Result<(), C::Error>
    where
        C: Context,
    {
        match self {
            Sink::Real(writer) => writer.write_byte(cx, b),
            Sink::Discard => Ok(()),
        }
    }
}
