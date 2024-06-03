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
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = alloc.alloc().expect("allocation failed");
    ///     assert_eq!(a.len(), 0);
    ///     a.write(b"Hello");
    ///     assert_eq!(a.len(), 5);
    /// });
    /// ```
    fn write(&mut self, bytes: &[u8]) -> bool;

    /// Write a single byte.
    ///
    /// Returns `true` if the bytes could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = alloc.alloc().expect("allocation failed");
    ///
    ///     a.push(b'H');
    ///     a.push(b'e');
    ///     a.push(b'l');
    ///     a.push(b'l');
    ///     a.push(b'o');
    ///
    ///     assert_eq!(a.as_slice(), b"Hello");
    /// });
    /// ```
    #[inline(always)]
    fn push(&mut self, byte: u8) -> bool {
        self.write(&[byte])
    }

    /// Get the initialized part of the buffer as a slice.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = alloc.alloc().expect("allocation failed");
    ///     assert_eq!(a.as_slice(), b"");
    ///     a.write(b"Hello");
    ///     assert_eq!(a.as_slice(), b"Hello");
    /// });
    /// ```
    fn as_slice(&self) -> &[u8];

    /// Get the number of initialized elements in the buffer.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = alloc.alloc().expect("allocation failed");
    ///     assert_eq!(a.len(), 0);
    ///     a.write(b"Hello");
    ///     assert_eq!(a.len(), 5);
    /// });
    /// ```
    fn len(&self) -> usize;

    /// Check if the buffer is empty.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = alloc.alloc().expect("allocation failed");
    ///     assert!(a.is_empty());
    ///     a.write(b"Hello");
    ///     assert!(!a.is_empty());
    /// });
    /// ```
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write a buffer of the same type onto the current buffer.
    ///
    /// This allows allocators to provide more efficient means of extending the
    /// current buffer with one provided from the same allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = alloc.alloc().expect("allocation failed");
    ///     let mut b = alloc.alloc().expect("allocation failed");
    ///     a.write(b"Hello");
    ///     b.write(b" World");
    ///
    ///     a.write_buffer(b);
    ///     assert_eq!(a.as_slice(), b"Hello World");
    /// });
    /// ```
    #[inline(always)]
    fn write_buffer<B>(&mut self, other: B) -> bool
    where
        B: Buf,
    {
        self.write(other.as_slice())
    }

    /// Try to write a format string into the buffer.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = alloc.alloc().expect("allocation failed");
    ///     let world = "World";
    ///
    ///     write!(a, "Hello {world}")?;
    ///
    ///     assert_eq!(a.as_slice(), b"Hello World");
    /// });
    /// # Ok::<(), musli::buf::Error>(())
    /// ```
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
