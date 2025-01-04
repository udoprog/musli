//! Traits related to memory allocation.

mod to_owned;
#[doc(inline)]
pub use self::to_owned::ToOwned;

mod allocator;
#[doc(inline)]
pub use self::allocator::Allocator;

#[cfg(feature = "alloc")]
mod system;
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub use self::system::{System, SystemAlloc};

mod disabled;
#[doc(inline)]
pub use self::disabled::Disabled;

mod string;
pub(crate) use self::string::collect_string;
#[doc(inline)]
pub use self::string::String;

mod vec;
#[doc(inline)]
pub use self::vec::Vec;

mod boxed;
#[doc(inline)]
pub use self::boxed::Box;

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
