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

pub mod stack;
pub use self::stack::{Stack, StackBuffer};

mod fixed;
#[doc(hidden)]
pub use self::fixed::{FixedString, FixedVec};

#[cfg(feature = "alloc")]
mod default_alloc {
    #![allow(missing_docs)]

    use super::Allocator;

    pub struct DefaultBuffer(super::SystemBuffer);
    pub struct Default<'a>(super::System<'a>);

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer(super::SystemBuffer::new())
    }

    pub(super) fn new(DefaultBuffer(buf): &mut DefaultBuffer) -> Default<'_> {
        Default(super::System::new(buf))
    }

    impl<'a> Allocator for Default<'a> {
        type Buf<'this> = <super::System<'a> as super::Allocator>::Buf<'this>
        where
            Self: 'this;

        #[inline(always)]
        fn alloc(&self) -> Option<Self::Buf<'_>> {
            self.0.alloc()
        }
    }
}

#[cfg(not(feature = "alloc"))]
mod default_alloc {
    #![allow(missing_docs)]

    use super::Allocator;

    type InnerAllocator<'a> = super::Stack<'a>;

    pub struct DefaultBuffer(super::StackBuffer<4096>);
    pub struct Default<'a>(InnerAllocator<'a>);

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer(super::StackBuffer::new())
    }

    pub(super) fn new(DefaultBuffer(buf): &mut DefaultBuffer) -> Default<'_> {
        Default(super::Stack::new(buf))
    }

    impl<'a> Allocator for Default<'a> {
        type Buf<'this> = <super::Stack<'a> as super::Allocator>::Buf<'this>
        where
            Self: 'this;

        #[inline(always)]
        fn alloc(&self) -> Option<Self::Buf<'_>> {
            self.0.alloc()
        }
    }
}

/// Construct a new default buffer.
///
/// Uses [`HeapBuffer`] if the `alloc` feature is enabled, otherwise
/// `StackBuffer` is used with a default size of `4096`.
pub fn buffer() -> DefaultBuffer {
    self::default_alloc::buffer()
}

/// Construct a new default allocator.
///
/// Uses the [`Alloc`] allocator if the `alloc` feature is enabled, otherwise
/// [`Stack`].
///
/// Requires that [`buffer()`] is used to construct the provided buffer.
pub fn new(buf: &mut DefaultBuffer) -> Default<'_> {
    self::default_alloc::new(buf)
}

/// The default allocator.
///
/// The exact implementation depends on which features are enabled (first one
/// takes preference):
/// * If `alloc` is enabled, this is the [`Alloc`] allocator.
/// * Otherwise this is the [`Stack`] allocator.
#[doc(inline)]
pub use self::default_alloc::{Default, DefaultBuffer};

use musli::context::Buf;

/// An allocator that can be used in combination with a context.
///
/// # Examples
///
/// ```
/// use musli_allocator::Allocator;
/// use musli::context::Buf;
///
/// let mut buf = musli_allocator::buffer();
/// let alloc = musli_allocator::new(&mut buf);
///
/// let mut a = alloc.alloc().expect("allocation a failed");
/// let mut b = alloc.alloc().expect("allocation b failed");
///
/// b.write(b"He11o");
/// a.write(b.as_slice());
///
/// assert_eq!(a.as_slice(), b"He11o");
/// assert_eq!(a.len(), 5);
///
/// a.write(b" W0rld");
///
/// assert_eq!(a.as_slice(), b"He11o W0rld");
/// assert_eq!(a.len(), 11);
///
/// let mut c = alloc.alloc().expect("allocation c failed");
/// c.write(b"!");
/// a.write(c.as_slice());
///
/// assert_eq!(a.as_slice(), b"He11o W0rld!");
/// assert_eq!(a.len(), 12);
/// ```
pub trait Allocator {
    /// The type of an allocated buffer.
    type Buf<'this>: Buf
    where
        Self: 'this;

    /// Allocate an empty, uninitialized buffer.
    ///
    /// Calling this method returns `None` if the allocation failed.
    fn alloc(&self) -> Option<Self::Buf<'_>>;
}

impl<A> Allocator for &A
where
    A: ?Sized + Allocator,
{
    type Buf<'this> = A::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        (*self).alloc()
    }
}
