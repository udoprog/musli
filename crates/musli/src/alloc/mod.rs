//! Allocation support for [Müsli].
//!
//! This crate contains two types of allocators:
//! * The [`System`] allocator, which uses the system allocation facilities.
//!   Particularly [`std::alloc::System`].
//! * The [`Slice`] allocator, which can allocate buffers from a fixed-size
//!   slice.
//!
//! The following types are also provided for convenience:
//! * [`Vec`] which can be used as a vector of allocations.
//! * [`String`] which can be used as a safe string container.
//!
//! <br>
//!
//! ## Examples
//!
//! ```
//! use musli::alloc::Vec;
//!
//! musli::alloc::default!(|alloc| {
//!     let mut a = Vec::new_in(alloc);
//!     let mut b = Vec::new_in(alloc);
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
//!     let mut c = Vec::new_in(alloc);
//!     c.write(b"!");
//!     a.write(c.as_slice());
//!
//!     assert_eq!(a.as_slice(), b"He11o W0rld!");
//!     assert_eq!(a.len(), 12);
//! });
//! ```
//!
//! Explicitly using a buffer on the stack with the [`Slice`] allocator:
//!
//! ```
//! use musli::alloc::{ArrayBuffer, Slice, Vec};
//!
//! let mut buf = ArrayBuffer::new();
//! let alloc = Slice::new(&mut buf);
//!
//! let mut a = Vec::new_in(&alloc);
//! let mut b = Vec::new_in(&alloc);
//!
//! b.write(b"He11o");
//! a.write(b.as_slice());
//!
//! assert_eq!(a.as_slice(), b"He11o");
//! assert_eq!(a.len(), 5);
//!
//! a.write(b" W0rld");
//!
//! assert_eq!(a.as_slice(), b"He11o W0rld");
//! assert_eq!(a.len(), 11);
//!
//! let mut c = Vec::new_in(&alloc);
//! c.write(b"!");
//! a.write(c.as_slice());
//!
//! assert_eq!(a.as_slice(), b"He11o W0rld!");
//! assert_eq!(a.len(), 12);
//! ```
//!
//! [Müsli]: <https://docs.rs/musli>
//! [`std::alloc::System`]: https://doc.rust-lang.org/std/alloc/struct.System.html

#[cfg(test)]
mod tests;

#[doc(inline)]
pub use musli_core::alloc::{Allocator, RawVec};

#[cfg(feature = "alloc")]
mod system;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub use self::system::System;

/// The static system allocator instance.
#[cfg(feature = "alloc")]
pub static SYSTEM: System = System::new();

mod disabled;
#[doc(inline)]
pub use self::disabled::Disabled;

mod stack;
#[doc(inline)]
pub use self::stack::Slice;

mod array_buffer;
pub use self::array_buffer::ArrayBuffer;

mod string;
pub(crate) use self::string::collect_string;
#[doc(inline)]
pub use self::string::String;

mod vec;
#[doc(inline)]
pub use self::vec::Vec;

/// The default stack buffer size for the default allocator provided through
/// [`default!`].
pub const DEFAULT_ARRAY_BUFFER: usize = 4096;

#[macro_export]
#[doc(hidden)]
macro_rules! __default {
    (|$alloc:ident| $body:block) => {
        $crate::alloc::__default_allocator_impl!(|$alloc| $body)
    };
}

/// Call the given block `$body` with the default allocator.
///
/// This is useful if you want to write application which are agnostic to
/// whether the `alloc` feature is or isn't enabled.
///
/// * If the `alloc` feature is enabled, this is the [`System`] allocator.
/// * If the `alloc` feature is disabled, this is the [`Slice`] allocator with
///   [`DEFAULT_ARRAY_BUFFER`] bytes allocated on the stack.
///
/// # Examples
///
/// ```
/// use musli::alloc::Vec;
///
/// musli::alloc::default!(|alloc| {
///     let mut a = Vec::new_in(alloc);
///     let mut b = Vec::new_in(alloc);
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
///     let mut c = Vec::new_in(alloc);
///     c.write(b"!");
///     a.write(c.as_slice());
///
///     assert_eq!(a.as_slice(), b"He11o W0rld!");
///     assert_eq!(a.len(), 12);
/// });
/// ```
#[doc(inline)]
pub use __default as default;

#[cfg(feature = "alloc")]
#[macro_export]
#[doc(hidden)]
macro_rules! __default_allocator_impl {
    (|$alloc:ident| $body:block) => {{
        let $alloc = &$crate::alloc::SYSTEM;
        $body
    }};
}

#[cfg(not(feature = "alloc"))]
#[macro_export]
#[doc(hidden)]
macro_rules! __default_allocator_impl {
    (|$alloc:ident| $body:block) => {{
        let mut __buf =
            $crate::alloc::ArrayBuffer::<{ $crate::alloc::DEFAULT_ARRAY_BUFFER }>::new();
        let $alloc = $crate::alloc::Slice::new(&mut __buf);
        $body
    }};
}

#[doc(hidden)]
pub use __default_allocator_impl;
