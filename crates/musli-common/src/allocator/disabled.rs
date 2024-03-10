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
    fn len(&self) -> usize {
        0
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
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
    type Buf<'this> = EmptyBuf;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf<'_> {
        EmptyBuf
    }
}
