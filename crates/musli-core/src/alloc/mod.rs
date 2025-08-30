//! # Müsli Rust core allocation and collections library
//!
//! This library provides smart pointers and collections for managing allocated
//! values. This is similar to the Rust [`alloc`][std-alloc] crate, it provides
//! similar but more limited functionality. However it can do so safely using
//! the Müsli-specific [`Allocator`] trait allowing these types to be used in
//! `no_std` environments without requiring a nightly compiler or `unsafe`.
//!
//! [std-alloc]: rust_alloc

mod to_owned;
#[doc(inline)]
pub use self::to_owned::ToOwned;

mod allocator;
#[doc(inline)]
pub use self::allocator::{Allocator, GlobalAllocator};

mod string;
pub(crate) use self::string::collect_string;
#[doc(inline)]
pub use self::string::{FromUtf8Error, String};

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
#[derive(Debug, PartialEq, Eq)]
pub struct AllocError;

impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to allocate memory")
    }
}

impl core::error::Error for AllocError {}
