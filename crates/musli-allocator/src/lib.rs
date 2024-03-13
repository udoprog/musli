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
//! [Müsli]: <https://docs.rs/musli>
//! [`std::alloc::System`]: https://doc.rust-lang.org/std/alloc/struct.System.html

#![allow(clippy::type_complexity)]
#![deny(missing_docs)]
#![no_std]

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
pub use self::system::{System, SystemBuffer};

mod disabled;
pub use self::disabled::Disabled;

mod stack;
#[doc(inline)]
pub use self::stack::{Stack, StackBuffer};

/// The default stack buffer size.
pub const DEFAULT_STACK_BUFFER: usize = 4096;

#[cfg(feature = "alloc")]
mod default_alloc {
    #[doc(hidden)]
    pub type DefaultBuffer = super::SystemBuffer;
    #[doc(hidden)]
    pub type Default<'a> = super::System<'a>;

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer::new()
    }

    pub(super) fn new(buf: &mut DefaultBuffer) -> Default<'_> {
        Default::new(buf)
    }
}

#[cfg(not(feature = "alloc"))]
mod default_alloc {
    use super::DEFAULT_STACK_BUFFER;

    #[doc(hidden)]
    pub type DefaultBuffer = super::StackBuffer<{ DEFAULT_STACK_BUFFER }>;
    #[doc(hidden)]
    pub type Default<'a> = super::Stack<'a>;

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer::new()
    }

    pub(super) fn new(buf: &mut DefaultBuffer) -> Default<'_> {
        Default::new(buf)
    }
}

/// The default backing allocator buffer.
///
/// * If the `alloc` feature is enabled, this is [`SystemBuffer`].
/// * Otherwise this is [`StackBuffer`] with a default size of
///   [`DEFAULT_STACK_BUFFER`].
#[doc(inline)]
pub use self::default_alloc::DefaultBuffer;

/// The default allocator.
///
/// * If the `alloc` feature is enabled, this is the [`System`] allocator.
/// * Otherwise this is the [`Stack`] allocator.
#[doc(inline)]
pub use self::default_alloc::Default;

/// Construct a new default buffer.
///
/// * If the `alloc` feature is enabled, this is [`SystemBuffer`].
/// * Otherwise this is [`StackBuffer`] with a default size of
///   [`DEFAULT_STACK_BUFFER`].
pub fn buffer() -> DefaultBuffer {
    self::default_alloc::buffer()
}

/// Construct a new default allocator.
///
/// * If the `alloc` feature is enabled, this is the [`System`] allocator.
/// * Otherwise this is the [`Stack`] allocator.
pub fn new(buf: &mut DefaultBuffer) -> Default<'_> {
    self::default_alloc::new(buf)
}
