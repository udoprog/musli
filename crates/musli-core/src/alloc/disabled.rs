use core::marker::PhantomData;
use core::ptr;

use super::{Alloc, AllocError, Allocator};

/// An empty buffer.
pub struct EmptyBuf<T> {
    _marker: PhantomData<T>,
}

impl<T> Alloc<T> for EmptyBuf<T> {
    #[inline]
    fn as_ptr(&self) -> *const T {
        ptr::NonNull::dangling().as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        ptr::NonNull::dangling().as_ptr()
    }

    #[inline]
    fn capacity(&self) -> usize {
        if size_of::<T>() == 0 {
            usize::MAX
        } else {
            0
        }
    }

    #[inline]
    fn resize(&mut self, _: usize, _: usize) -> Result<(), AllocError> {
        Err(AllocError)
    }

    #[inline]
    fn try_merge<B>(&mut self, _: usize, other: B, _: usize) -> Result<(), B>
    where
        B: Alloc<T>,
    {
        Err(other)
    }
}

/// An allocator which cannot allocate anything.
///
/// If any operation requires allocations this will error.
#[non_exhaustive]
pub struct Disabled;

impl Disabled {
    /// Construct a new disabled allocator.
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for Disabled {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator for Disabled {
    type Alloc<T> = EmptyBuf<T>;

    #[inline]
    fn alloc<T>(self, _: T) -> Result<Self::Alloc<T>, AllocError> {
        Err(AllocError)
    }

    #[inline]
    fn alloc_empty<T>(self) -> Self::Alloc<T> {
        EmptyBuf {
            _marker: PhantomData,
        }
    }
}
