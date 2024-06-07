//! Types related to buffers.

use core::fmt::{self, Arguments};
use core::mem::ManuallyDrop;
use core::ptr;

/// An error raised when we fail to write.
#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Allocation failed")
    }
}

/// A bytes-oriented buffer.
pub struct BytesBuf<B> {
    buf: B,
    len: usize,
}

impl<B> BytesBuf<B>
where
    B: Buf<Item = u8>,
{
    pub(crate) fn from_aligned(buf: AlignedBuf<B>) -> Self {
        Self {
            buf: buf.into_inner(),
            len: 0,
        }
    }

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
    pub fn write(&mut self, bytes: &[u8]) -> bool {
        if !self.buf.reserve(bytes.len()) {
            return false;
        }

        // SAFETY: The call to reserve ensures that we have enough capacity.
        unsafe {
            self.buf
                .as_ptr_mut()
                .add(self.len)
                .copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
            self.len += bytes.len();
        }

        true
    }

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
    #[inline]
    pub fn push(&mut self, byte: u8) -> bool {
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
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: We know that the buffer is initialized up to `self.len`.
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr(), self.len) }
    }

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
    pub fn len(&self) -> usize {
        self.len
    }

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
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
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
    #[inline]
    pub fn write_buffer<U>(&mut self, other: BytesBuf<U>) -> bool
    where
        U: Buf<Item = u8>,
    {
        // Try to merge one buffer with another.
        if let Err(buf) = self.buf.try_merge(other.buf) {
            let other = BytesBuf {
                len: other.len,
                buf,
            };

            return self.write(other.as_slice());
        }

        true
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
    #[inline]
    pub fn write_fmt(&mut self, arguments: Arguments<'_>) -> Result<(), Error> {
        struct Write<'a, B>(&'a mut BytesBuf<B>);

        impl<B> fmt::Write for Write<'_, B>
        where
            B: Buf<Item = u8>,
        {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                if !self.0.write(s.as_bytes()) {
                    return Err(fmt::Error);
                }

                Ok(())
            }
        }

        let mut write = Write(self);
        fmt::write(&mut write, arguments).map_err(|_| Error)
    }
}

/// An aligned buffer.
pub struct AlignedBuf<B>
where
    B: Buf,
{
    buf: B,
    len: usize,
}

impl<B> AlignedBuf<B>
where
    B: Buf,
{
    fn into_inner(self) -> B {
        let buf = ManuallyDrop::new(self);
        unsafe { ptr::addr_of!((&buf).buf).read() }
    }

    /// Push an item into the buffer.
    pub fn push(&mut self, item: B::Item) -> bool {
        if !self.buf.reserve(1) {
            return false;
        }

        // SAFETY: The call to reserve ensures that we have enough capacity.
        unsafe {
            self.buf.as_ptr_mut().add(self.len).write(item);
            self.len += 1;
        }

        true
    }
}

impl<B> Drop for AlignedBuf<B>
where
    B: Buf,
{
    fn drop(&mut self) {
        // SAFETY: We know that the buffer is initialized up to `len`.
        unsafe {
            core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                self.buf.as_ptr_mut(),
                self.len,
            ))
        }
    }
}

impl<B> AlignedBuf<B> where B: Buf {}

/// A raw buffer allocated from a context.
///
/// Buffers are allocated through an allocator using [`Allocator::alloc`].
///
/// [`Allocator::alloc`]: crate::Allocator::alloc
pub trait Buf {
    /// An item in the buffer.
    type Item: 'static;

    /// Reserve space for the given number of items.
    fn reserve(&mut self, additional: usize) -> bool;

    /// Get a pointer into the buffer.
    fn as_ptr(&self) -> *const Self::Item;

    /// Get a mutable pointer into the buffer.
    fn as_ptr_mut(&mut self) -> *mut Self::Item;

    /// Try to merge one buffer with another.
    ///
    /// If this returns `Err(B)` if merging was not possible.
    fn try_merge<B>(&mut self, other: B) -> Result<(), B>
    where
        B: Buf<Item = Self::Item>;
}
