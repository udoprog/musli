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
//! use musli::alloc::{AllocError, Vec};
//!
//! musli::alloc::default(|alloc| {
//!     let mut a = Vec::new_in(alloc);
//!     let mut b = Vec::new_in(alloc);
//!
//!     b.extend_from_slice(b"He11o")?;
//!     a.extend_from_slice(b.as_slice())?;
//!
//!     assert_eq!(a.as_slice(), b"He11o");
//!     assert_eq!(a.len(), 5);
//!
//!     a.extend_from_slice(b" W0rld")?;
//!
//!     assert_eq!(a.as_slice(), b"He11o W0rld");
//!     assert_eq!(a.len(), 11);
//!
//!     let mut c = Vec::new_in(alloc);
//!     c.extend_from_slice(b"!")?;
//!     a.extend_from_slice(c.as_slice())?;
//!
//!     assert_eq!(a.as_slice(), b"He11o W0rld!");
//!     assert_eq!(a.len(), 12);
//!     Ok::<_, AllocError>(())
//! });
//! # Ok::<_, musli::alloc::AllocError>(())
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
//! b.extend_from_slice(b"He11o")?;
//! a.extend_from_slice(b.as_slice())?;
//!
//! assert_eq!(a.as_slice(), b"He11o");
//! assert_eq!(a.len(), 5);
//!
//! a.extend_from_slice(b" W0rld")?;
//!
//! assert_eq!(a.as_slice(), b"He11o W0rld");
//! assert_eq!(a.len(), 11);
//!
//! let mut c = Vec::new_in(&alloc);
//! c.extend_from_slice(b"!")?;
//! a.extend_from_slice(c.as_slice())?;
//!
//! assert_eq!(a.as_slice(), b"He11o W0rld!");
//! assert_eq!(a.len(), 12);
//! # Ok::<_, musli::alloc::AllocError>(())
//! ```
//!
//! [Müsli]: <https://docs.rs/musli>
//! [`std::alloc::System`]: https://doc.rust-lang.org/std/alloc/struct.System.html

#[cfg(test)]
mod tests;

mod default;
#[doc(inline)]
pub use self::default::{DefaultAllocator, DEFAULT_ARRAY_BUFFER};

#[doc(inline)]
pub use musli_core::alloc::{Alloc, AllocError, Allocator, Box, Disabled, String, ToOwned, Vec};

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use musli_core::alloc::{System, SystemAlloc};

mod stack;
#[doc(inline)]
pub use self::stack::{Slice, SliceAlloc};

mod array_buffer;
pub use self::array_buffer::ArrayBuffer;

/// Call the given block `body` with an instance of the [`DefaultAllocator`].
///
/// This is useful if you want to write application which are agnostic to
/// whether the `alloc` feature is or isn't enabled.
///
/// * If the `alloc` feature is enabled, this is the [`System`] allocator.
/// * If the `alloc` feature is disabled, this is the [`Slice`] allocator with
///   [`DEFAULT_ARRAY_BUFFER`] bytes allocated on the stack. The second
///   parameters allows for this to be tweaked.
///
/// Note that the [`DEFAULT_ARRAY_BUFFER`] parameter is always present since it
/// is necessary to make the type generic over all default allocators.
///
/// # Examples
///
/// ```
/// use musli::alloc::{AllocError, Vec};
///
/// musli::alloc::default(|alloc| {
///     let mut a = Vec::new_in(alloc);
///     let mut b = Vec::new_in(alloc);
///
///     b.extend_from_slice(b"He11o")?;
///     a.extend_from_slice(b.as_slice())?;
///
///     assert_eq!(a.as_slice(), b"He11o");
///     assert_eq!(a.len(), 5);
///
///     a.extend_from_slice(b" W0rld")?;
///
///     assert_eq!(a.as_slice(), b"He11o W0rld");
///     assert_eq!(a.len(), 11);
///
///     let mut c = Vec::new_in(alloc);
///     c.extend_from_slice(b"!")?;
///     a.extend_from_slice(c.as_slice())?;
///
///     assert_eq!(a.as_slice(), b"He11o W0rld!");
///     assert_eq!(a.len(), 12);
///     Ok::<_, AllocError>(())
/// });
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
#[inline]
pub fn default<O>(body: impl FnOnce(&DefaultAllocator<'_, DEFAULT_ARRAY_BUFFER>) -> O) -> O {
    default_allocator_impl::<DEFAULT_ARRAY_BUFFER, O>(body)
}

/// Same as [`default()`] but allows for specifying a default static buffer size
/// other than [`DEFAULT_ARRAY_BUFFER`].
///
/// See [`default()`] for more information.
///
/// # Examples
///
/// ```
/// use musli::alloc::{AllocError, Vec};
///
/// musli::alloc::with_buffer::<128, _>(|alloc| {
///     let mut a = Vec::new_in(alloc);
///     let mut b = Vec::new_in(alloc);
///
///     b.extend_from_slice(b"He11o")?;
///     a.extend_from_slice(b.as_slice())?;
///
///     assert_eq!(a.as_slice(), b"He11o");
///     assert_eq!(a.len(), 5);
///
///     a.extend_from_slice(b" W0rld")?;
///
///     assert_eq!(a.as_slice(), b"He11o W0rld");
///     assert_eq!(a.len(), 11);
///
///     let mut c = Vec::new_in(alloc);
///     c.extend_from_slice(b"!")?;
///     a.extend_from_slice(c.as_slice())?;
///
///     assert_eq!(a.as_slice(), b"He11o W0rld!");
///     assert_eq!(a.len(), 12);
///     Ok::<_, AllocError>(())
/// });
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
#[inline]
pub fn with_buffer<const BUF: usize, O>(body: impl FnOnce(&DefaultAllocator<'_, BUF>) -> O) -> O {
    default_allocator_impl::<BUF, O>(body)
}

#[cfg(feature = "alloc")]
#[inline(always)]
fn default_allocator_impl<const BUF: usize, O>(
    body: impl FnOnce(&DefaultAllocator<'_, BUF>) -> O,
) -> O {
    let alloc = DefaultAllocator::new(System::new());
    body(&alloc)
}

#[cfg(not(feature = "alloc"))]
#[inline(always)]
fn default_allocator_impl<const BUF: usize, O>(
    body: impl FnOnce(&DefaultAllocator<'_, BUF>) -> O,
) -> O {
    let mut buf = crate::alloc::ArrayBuffer::<BUF>::with_size();
    let slice = crate::alloc::Slice::new(&mut buf);
    let alloc = DefaultAllocator::new(slice);
    body(&alloc)
}
