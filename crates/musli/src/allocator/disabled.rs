use core::marker::PhantomData;
use core::ptr;

use crate::{Allocator, Buf};

/// An empty buffer.
pub struct EmptyBuf<T> {
    _marker: PhantomData<T>,
}

impl<T> Buf<T> for EmptyBuf<T> {
    #[inline]
    fn resize(&mut self, _: usize, _: usize) -> bool {
        false
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        ptr::NonNull::dangling().as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        ptr::NonNull::dangling().as_ptr()
    }

    #[inline]
    fn try_merge<B>(&mut self, _: usize, other: B, _: usize) -> Result<(), B>
    where
        B: Buf<T>,
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
    type Buf<'this, T> = EmptyBuf<T> where T: 'this;

    #[inline(always)]
    fn alloc<'a, T>(&'a self) -> Self::Buf<'a, T>
    where
        T: 'a,
    {
        EmptyBuf {
            _marker: PhantomData,
        }
    }
}
