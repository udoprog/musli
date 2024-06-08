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
//! use musli::buf::BytesBuf;
//!
//! musli::allocator::default!(|alloc| {
//!     let mut a = BytesBuf::new(alloc.alloc().expect("allocation a failed"));
//!     let mut b = BytesBuf::new(alloc.alloc().expect("allocation b failed"));
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
//!     let mut c = BytesBuf::new(alloc.alloc().expect("allocation c failed"));
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

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
mod system;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub use self::system::System;

/// The static system allocator instance.
#[cfg(all(feature = "alloc", not(loom)))]
pub static SYSTEM: System = System::new();

mod disabled;
pub use self::disabled::Disabled;

mod stack;
#[doc(inline)]
pub use self::stack::{Stack, StackBuffer};

/// The default stack buffer size for the default allocator provided through
/// [`default!`].
pub const DEFAULT_STACK_BUFFER: usize = 4096;

#[macro_export]
#[doc(hidden)]
macro_rules! __default {
    (|$alloc:ident| $body:block) => {
        $crate::allocator::__default_allocator_impl!(|$alloc| $body)
    };
}

/// Call the given block `$body` with the default allocator.
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
/// use musli::buf::BytesBuf;
///
/// musli::allocator::default!(|alloc| {
///     let mut a = BytesBuf::new(alloc.alloc().expect("allocation a failed"));
///     let mut b = BytesBuf::new(alloc.alloc().expect("allocation b failed"));
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
///     let mut c = BytesBuf::new(alloc.alloc().expect("allocation c failed"));
///     c.write(b"!");
///     a.write(c.as_slice());
///
///     assert_eq!(a.as_slice(), b"He11o W0rld!");
///     assert_eq!(a.len(), 12);
/// });
/// ```
#[doc(inline)]
pub use __default as default;

#[cfg(all(feature = "alloc", not(loom)))]
#[macro_export]
#[doc(hidden)]
macro_rules! __default_allocator_impl {
    (|$alloc:ident| $body:block) => {{
        let $alloc = &$crate::allocator::SYSTEM;
        $body
    }};
}

#[cfg(all(feature = "alloc", loom))]
#[macro_export]
#[doc(hidden)]
macro_rules! __default_allocator_impl {
    (|$alloc:ident| $body:block) => {{
        let $alloc = $crate::allocator::System::new();
        let $alloc = &$alloc;
        $body
    }};
}

#[cfg(not(feature = "alloc"))]
#[macro_export]
#[doc(hidden)]
macro_rules! __default_allocator_impl {
    (|$alloc:ident| $body:block) => {{
        let mut __buf =
            $crate::allocator::StackBuffer::<{ $crate::allocator::DEFAULT_STACK_BUFFER }>::new();
        let $alloc = $crate::allocator::Stack::new(&mut __buf);
        $body
    }};
}

#[doc(hidden)]
pub use __default_allocator_impl;
