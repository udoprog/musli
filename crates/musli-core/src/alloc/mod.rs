//! Traits related to memory allocation.

mod allocator;
#[doc(inline)]
pub use self::allocator::Allocator;

#[allow(clippy::module_inception)]
mod alloc;
#[doc(inline)]
pub use self::alloc::Alloc;

use core::fmt;

/// An allocation error.
#[derive(Debug)]
pub struct AllocError;

impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to allocate memory")
    }
}

impl core::error::Error for AllocError {}
