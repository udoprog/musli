//! Types related to buffers.

use core::fmt::{self, Arguments};

/// An error raised when we fail to write.
#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Allocation failed")
    }
}

/// A buffer allocated from a context.
///
/// Buffers are allocated through an allocator using [`Allocator::alloc`].
///
/// [`Allocator::alloc`]: crate::Allocator::alloc
pub trait Buf {
    /// Write the given number of bytes.
    ///
    /// Returns `true` if the bytes could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    fn write(&mut self, bytes: &[u8]) -> bool;

    /// Write a buffer of the same type onto the current buffer.
    ///
    /// This allows allocators to provide more efficient means of extending the
    /// current buffer with one provided from the same allocator.
    #[inline(always)]
    fn write_buffer<B>(&mut self, other: B) -> bool
    where
        B: Buf,
    {
        self.write(other.as_slice())
    }

    /// Write a single byte.
    ///
    /// Returns `true` if the bytes could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    #[inline(always)]
    fn push(&mut self, byte: u8) -> bool {
        self.write(&[byte])
    }

    /// Get the length of the buffer in bytes.
    fn len(&self) -> usize;

    /// Test if the buffer is empty.
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the buffer as its initialized slice.
    fn as_slice(&self) -> &[u8];

    /// Try to write a format string into the buffer.
    fn write_fmt(&mut self, arguments: Arguments<'_>) -> Result<(), Error>;
}

impl Buf for [u8] {
    #[inline(always)]
    fn write(&mut self, _: &[u8]) -> bool {
        false
    }

    #[inline(always)]
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        <[_]>::is_empty(self)
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        self
    }

    #[inline]
    fn write_fmt(&mut self, _: Arguments<'_>) -> Result<(), Error> {
        Err(Error)
    }
}
