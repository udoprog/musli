//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-allocator.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-allocator)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--allocator-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-allocator)
//!
//! Allocation support for [Müsli].
//!
//! This crate contains two types of allocators:
//! * The [`System`] allocator, which uses the system allocation facilities.
//!   Particularly [`std::alloc::System`].
//! * The [`Stack`] allocator, which can allocate buffers from a fixed-size
//!   slice.
//!
//! <br>
//!
//! ## Examples
//!
//! ```
//! use musli::{Allocator, Buf};
//!
//! musli_allocator::with(|alloc| {
//!     let mut a = alloc.alloc().expect("allocation a failed");
//!     let mut b = alloc.alloc().expect("allocation b failed");
//!
//!     b.write(b"He11o");
//!     a.write(b.as_slice());
//!
//!     assert_eq!(a.as_slice(), b"He11o");
//!     assert_eq!(a.len(), 5);
//!
//!     a.write(b" W0rld");
//!
//!     assert_eq!(a.as_slice(), b"He11o W0rld");
//!     assert_eq!(a.len(), 11);
//!
//!     let mut c = alloc.alloc().expect("allocation c failed");
//!     c.write(b"!");
//!     a.write(c.as_slice());
//!
//!     assert_eq!(a.as_slice(), b"He11o W0rld!");
//!     assert_eq!(a.len(), 12);
//! });
//! ```
//!
//! [Müsli]: <https://docs.rs/musli>
//! [`std::alloc::System`]: https://doc.rust-lang.org/std/alloc/struct.System.html

#![deny(missing_docs)]
#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg_attr(test, macro_use)]
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
mod system;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub use self::system::System;

mod disabled;
pub use self::disabled::Disabled;

mod stack;
#[doc(inline)]
pub use self::stack::{Stack, StackBuffer};

/// The default stack buffer size for the default allocator provided through
/// [`with`].
pub const DEFAULT_STACK_BUFFER: usize = 4096;

/// The default allocator.
#[cfg(feature = "alloc")]
pub type Default<'a> = System;

/// The default allocator.
#[cfg(not(feature = "alloc"))]
pub type Default<'a> = Stack<'a>;

/// Call the given closure with the default allocator.
///
/// This is useful if you want to write application which are agnostic to
/// whether the `alloc` feature is or isn't enabled.
///
/// * If the `alloc` feature is enabled, this is the [`System`] allocator.
/// * If the `alloc` feature is disabled, this is the [`Stack`] allocator with
///   [`DEFAULT_STACK_BUFFER`] bytes allocated on the stack.
///
/// # Examples
///
/// ```
/// use musli::{Allocator, Buf};
///
/// musli_allocator::with(|alloc| {
///     let mut a = alloc.alloc().expect("allocation a failed");
///     let mut b = alloc.alloc().expect("allocation b failed");
///
///     b.write(b"He11o");
///     a.write(b.as_slice());
///
///     assert_eq!(a.as_slice(), b"He11o");
///     assert_eq!(a.len(), 5);
///
///     a.write(b" W0rld");
///
///     assert_eq!(a.as_slice(), b"He11o W0rld");
///     assert_eq!(a.len(), 11);
///
///     let mut c = alloc.alloc().expect("allocation c failed");
///     c.write(b"!");
///     a.write(c.as_slice());
///
///     assert_eq!(a.as_slice(), b"He11o W0rld!");
///     assert_eq!(a.len(), 12);
/// });
/// ```
#[inline(always)]
pub fn with<F, O>(f: F) -> O
where
    F: FnOnce(&Default<'_>) -> O,
{
    with_impl(f)
}

#[cfg(feature = "alloc")]
#[inline(always)]
fn with_impl<F, O>(f: F) -> O
where
    F: FnOnce(&System) -> O,
{
    let alloc = System::new();
    f(&alloc)
}

#[cfg(not(feature = "alloc"))]
#[inline(always)]
fn with_impl<F, O>(f: F) -> O
where
    F: FnOnce(&Stack<'_>) -> O,
{
    let mut buf = StackBuffer::<DEFAULT_STACK_BUFFER>::new();
    let alloc = Stack::new(&mut buf);
    f(&alloc)
}
