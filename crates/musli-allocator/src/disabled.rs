use core::fmt::Arguments;

use musli::buf::Error;
use musli::{Allocator, Buf};

/// An empty buffer.
pub struct EmptyBuf;

impl Buf for EmptyBuf {
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

    #[inline(always)]
    fn write_fmt(&mut self, _: Arguments<'_>) -> Result<(), Error> {
        Err(Error)
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
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        Some(EmptyBuf)
    }
}
