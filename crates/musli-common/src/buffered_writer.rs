//! A writer which buffers the writes before it outputs it into the backing
//! storage.

use crate::fixed_bytes::FixedBytes;
use crate::writer::Writer;

/// A writer which buffers `N` bytes inline.
///
/// Once you're done you must call [BufferedWriter::finish] to flush the
/// underlying buffer.
pub struct BufferedWriter<const N: usize, W>
where
    W: Writer,
{
    buf: FixedBytes<N, W::Error>,
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
    pub fn finish(mut self) -> Result<(), W::Error> {
        if !self.buf.is_empty() {
            self.writer.write_bytes(self.buf.as_slice())?;
        }

        Ok(())
    }
}

impl<const N: usize, W> Writer for BufferedWriter<N, W>
where
    W: Writer,
{
    type Error = W::Error;
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        if self.buf.remaining() < bytes.len() {
            self.writer.write_bytes(self.buf.as_slice())?;
            self.buf.clear();
        }

        self.buf.write_bytes(bytes)
    }
}
