//! Types related to buffers.

use core::fmt;

/// An error raised when we fail to write.
#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Allocation failed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// A raw buffer allocated from a context.
///
/// Buffers are allocated through an allocator using [`Allocator::alloc`].
///
/// [`Allocator::alloc`]: crate::Allocator::alloc
pub trait Buf {
    /// An item in the buffer.
    type Item: 'static;

    /// Resize the buffer.
    fn resize(&mut self, len: usize, additional: usize) -> bool;

    /// Get a pointer into the buffer.
    fn as_ptr(&self) -> *const Self::Item;

    /// Get a mutable pointer into the buffer.
    fn as_ptr_mut(&mut self) -> *mut Self::Item;

    /// Try to merge one buffer with another.
    ///
    /// The two length parameters refers to the initialized length of the two
    /// buffers.
    ///
    /// If this returns `Err(B)` if merging was not possible.
    fn try_merge<B>(&mut self, this_len: usize, other: B, other_len: usize) -> Result<(), B>
    where
        B: Buf<Item = Self::Item>;
}
