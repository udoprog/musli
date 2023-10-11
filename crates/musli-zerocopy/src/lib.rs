//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy)
//!
//! Zero copy primitives for use in Müsli.
//!
//! This provides a base set of tools to deal with types which do not require
//! copying during deserialization.
//!
//! To implement zero-copy support for a Rust type, see the [`ZeroCopy`] derive.
//!
//! <br>
//!
//! ## Guide
//!
//! ```
//! # use anyhow::Context;
//! use core::mem::size_of;
//! use musli_zerocopy::{AlignedBuf, Pair, Unsized, ZeroCopy};
//!
//! #[derive(ZeroCopy)]
//! #[repr(C)]
//! struct Custom {
//!     field: u32,
//!     string: Unsized<str>,
//! }
//!
//! let mut buf = AlignedBuf::new();
//!
//! let string = buf.store_unsized("Hello World!")?;
//! let custom = buf.store(&Custom { field: 42, string })?;
//!
//! // The buffer stores both the unsized string and the Custom element.
//! assert!(buf.len() >= 24);
//! // We assert that the produced alignment is smaller or equal to 8
//! // since we'll be relying on this below.
//! assert!(buf.requested() <= 8);
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! Later when we want to use the type, we take the buffer we've generated and
//! include it somewhere else.
//!
//! There's a few pieces of data (called DNA) we need to have to read a type
//! back from a raw buffer:
//! * The type being read which implements [`ZeroCopy`]. This is `Custom` above.
//!   The [`ZeroCopy`] derive ensures that we can safely coerce a buffer into a
//!   reference of the type.
//! * The alignment of the buffer, which you can access through the
//!   [`requested()`]. On the receiving end we need to ensure that the buffer
//!   follow this alignment. Dynamically this can be achieved by loading the
//!   buffer back into an appropriately constructed [`AlignedBuf`] instance.
//!   Other tricks include embedding a static buffer inside of an aligned
//!   newtype which we'll showcase below.
//! * The [`Offset`] at where the [`ZeroCopy`] structure is read. To read a
//!   structure we combine a pointer and a type into the [`Ref`] type.
//! * The endianness of the machine which produced the buffer. Any numerical
//!   elements will have been encoded in native endian ordering, so they would
//!   have to be adjusted on the receiving side if it differs.
//!
//! If the goal is to both produce and read the buffer on the same system
//! certain assumptions can be made. But even if those assumptions are wrong,
//! the worst outcome will only ever be an error as long as you're using the
//! safe APIs or abide by the safety documentation of the unsafe APIs.
//!
//! The following is an example of reading the type directly out of a newtype
//! aligned `&'static [u8]` buffer:
//!
//! ```
//! # use musli_zerocopy::{ZeroCopy, Unsized};
//! # macro_rules! include_bytes {
//! # ($path:literal) => { &[
//! #    b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd', b'!',
//! #    42, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0,
//! # ] };
//! # }
//! # #[derive(ZeroCopy)]
//! # #[repr(C)]
//! # struct Custom { field: u32, string: Unsized<str> }
//! use core::mem::size_of;
//! use musli_zerocopy::{Ref, Offset, Buf};
//!
//! // Helper to force the static buffer to be aligned.
//! #[repr(align(8))]
//! struct Align(&'static [u8]);
//!
//! static BYTES: Align = Align(include_bytes!("custom.bin"));
//!
//! let buf = Buf::new(BYTES.0);
//!
//! // Construct a pointer into the buffer.
//! let custom = Ref::new(Offset::new(BYTES.0.len() - size_of::<Custom>()));
//!
//! let custom: &Custom = buf.load(custom)?;
//! assert_eq!(custom.field, 42);
//! assert_eq!(buf.load(custom.string)?, "Hello World!");
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! <br>
//!
//! ## Limits
//!
//! Offset, the size of unsized values, and slice lengths are all limited to
//! 32-bit. The system you're using must have a `usize` type which is at least
//! 32-bits wide. This is done to save space by default.
//!
//! The pointer width on the system is checked at compile time, while trying to
//! use an offset or a size larger than `2^32` will result in a panic.
//!
//! Example of using an [`Offset`] larger than `2^32` causing a panic:
//!
//! ```should_panic
//! # use musli_zerocopy::Offset;
//! Offset::new(1usize << 32);
//! ```
//!
//! Example panic using a [`Slice`] with a length larger than `2^32`:
//!
//! ```should_panic
//! # use musli_zerocopy::{Offset, Slice};
//! Slice::<u32>::new(Offset::ZERO, 1usize << 32);
//! ```
//!
//! Example panic using an [`Unsized`] value with a size larger than `2^32`:
//!
//! ```should_panic
//! # use musli_zerocopy::{Offset, Unsized};
//! Unsized::<str>::new(Offset::ZERO, 1usize << 32);
//! ```
//!
//! If you want to address data larger than this limit, it is recommended that
//! you partition your dataset into 32-bit addressable chunks.
//!
//! [`requested()`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.AlignedBuf.html#method.requested
//! [`Ref`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Ref.html
//! [`Offset`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Offset.html
//! [`Slice`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Slice.html
//! [`Unsized`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Unsized.html
//! [`AlignedBuf`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.AlignedBuf.html
//! [`ZeroCopy`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/derive.ZeroCopy.html

#![no_std]
#![allow(clippy::module_inception)]
#![deny(missing_docs)]
#![cfg_attr(all(feature = "nightly", test), feature(repr128))]
#![cfg_attr(all(feature = "nightly", test), allow(incomplete_features))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub use self::load::{Load, LoadMut};
mod load;

pub use self::visit::Visit;
mod visit;

pub use self::buf::{Buf, Validator};
mod buf;

pub use self::buf_mut::BufMut;
mod buf_mut;

pub use self::error::Error;
mod error;

pub use self::offset::Offset;
mod offset;

pub use self::store_struct::StoreStruct;
mod store_struct;

#[cfg(feature = "alloc")]
pub use self::aligned_buf::AlignedBuf;

#[cfg(feature = "alloc")]
mod aligned_buf;

pub use self::r#ref::Ref;
mod r#ref;

pub use self::slice::Slice;
mod slice;

pub use self::r#unsized::Unsized;
mod r#unsized;

pub use self::zero_copy::{UnsizedZeroCopy, ZeroCopy, ZeroSized};
mod zero_copy;

pub use self::phf::{Map, MapRef};
mod phf;

pub use self::pair::Pair;
mod pair;

pub use self::bind::Bindable;
mod bind;

/// Derive macro to implement [`ZeroCopy`].
///
/// Implementing this trait ensures that the type can safely be coerced to and
/// from initialized bytes.
///
/// # Using with structs
///
/// The following are the requirements for deriving structs:
/// * The struct must either be `#[repr(C)]` or `[repr(transparent)]`.
/// * All fields in the struct must either implement [`ZeroCopy`] or be
///   [`ZeroSized`] and marked as `#[zero_copy(ignore)]`.
///
/// If the struct is zero-sized, it will implement [`ZeroSized`] along with the
/// [`ZeroCopy`] trait.
///
/// ```
/// use musli_zerocopy::{AlignedBuf, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(C, align(128))]
/// struct Custom {
///     field: u32,
/// }
///
/// let mut buf = AlignedBuf::new();
/// let ptr = buf.store(&Custom { field: 10 })?;
/// let buf = buf.as_aligned();
/// assert_eq!(buf.load(ptr)?, &Custom { field: 10 });
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// [`ZeroCopy`]: trait@crate::zero_copy::ZeroCopy
///
/// # Using with enums
///
/// The following are the requirements for deriving for enums:
/// * The enum must be marked with a valid, fixed representation. Such as
///   `#[repr(u8)]`, or `#[repr(usize)]`.
/// * If custom discriminators are used, only constant values can be used.
///
/// ```
/// use musli_zerocopy::{AlignedBuf, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(u32)]
/// enum Flags {
///     First = 1,
///     Second, // will be automatically assigned 2
///     Third = 5,
/// }
///
/// let mut buf = AlignedBuf::new();
/// let ptr = buf.store(&Flags::First)?;
/// let buf = buf.as_aligned();
/// assert_eq!(buf.load(ptr)?, &Flags::First);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// # Attributes
///
/// ## Type attributes
///
/// The following `repr` attributes are supported:
/// * repr(C) - Ensures that the type has the mandatory represention.
/// * repr(transparent) - If there is a single field inside of the marked struct
///   which implements `ZeroCopy`.
/// * repr(align(..)) - Allows for control over the struct alignment.
///
/// The following `zero_copy(..)` attribute are supported:
///
/// ### `zero_copy(bounds = {<bound>,*})`
///
/// Allows for adding additional bounds to implement `ZeroCopy` for generic
/// types:
///
/// ```
/// use musli_zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// #[zero_copy(bounds = {A: ZeroCopy, B: ZeroCopy})]
/// struct Pair<A, B> {
///     left: A,
///     right: B,
/// }
/// ```
///
/// ### `zero_copy(crate = <path>)`
///
/// Allows for specifying a custom path to the `musli_zerocopy`` crate
/// (default).
///
/// ```
/// use musli_zerocopy as zerocopy;
///
/// use zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// #[zero_copy(crate = zerocopy)]
/// struct Custom {
///     field: u32,
/// }
/// ```
#[doc(inline)]
pub use musli_macros::ZeroCopy;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub mod __private {
    pub use ::core::result;
}
