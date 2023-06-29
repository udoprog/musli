use core::ptr;

use musli::context::Buffer;

use crate::allocator::Allocator;

/// An empty buffer.
pub struct EmptyBuf;

impl Buffer for EmptyBuf {
    #[inline(always)]
    fn write(&mut self, _: &[u8]) -> bool {
        false
    }

    #[inline(always)]
    fn write_at(&mut self, _: usize, _: &[u8]) -> bool {
        false
    }

    #[inline(always)]
    fn copy_back<B>(&mut self, _: B) -> bool
    where
        B: Buffer,
    {
        false
    }

    #[inline(always)]
    fn len(&self) -> usize {
        0
    }

    #[inline(always)]
    fn raw_parts(&self) -> (*const u8, usize, usize) {
        (ptr::null(), 0, 0)
    }

    #[inline(always)]
    unsafe fn as_slice(&self) -> &[u8] {
        &[]
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
    type Buf = EmptyBuf;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf {
        EmptyBuf
    }
}
