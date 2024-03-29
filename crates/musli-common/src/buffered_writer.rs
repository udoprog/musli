//! A writer which buffers the writes before it outputs it into the backing
//! storage.

use musli::{Buf, Context};

use crate::fixed::FixedBytes;
use crate::writer::Writer;

/// A writer which buffers `N` bytes inline.
///
/// Once you're done you must call [BufferedWriter::finish] to flush the
/// underlying buffer.
pub struct BufferedWriter<const N: usize, W> {
    buf: FixedBytes<N>,
    writer: W,
}

impl<const N: usize, W> BufferedWriter<N, W>
where
    W: Writer,
{
    /// Construct a new buffered writer.
    pub fn new(writer: W) -> Self {
        Self {
            buf: FixedBytes::new(),
            writer,
        }
    }

    /// Finish writing.
    pub fn finish<C>(mut self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        if !self.buf.is_empty() {
            self.writer.write_bytes(cx, self.buf.as_slice())?;
        }

        Ok(())
    }
}

impl<const N: usize, W> Writer for BufferedWriter<N, W>
where
    W: Writer,
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
        if self.buf.remaining() < bytes.len() {
            self.writer.write_bytes(cx, self.buf.as_slice())?;
            self.buf.clear();
        }

        self.buf.write_bytes(cx, bytes)
    }
}
