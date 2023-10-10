//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy)
//!
//! Zero copy primitives for use in Müsli.
//!
//! This provides a base set of tools to deal with types which do not require
//! copying during deserialization.
//!
//! <br>
//!
//! ## Examples
//!
//! ```
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
//! let string = buf.insert_unsized("string")?;
//!
//! let c1 = buf.insert_sized(Custom { field: 1, string })?;
//! let c2 = buf.insert_sized(Custom { field: 2, string })?;
//!
//! let mut map = Vec::new();
//!
//! map.push(Pair::new(1, c1));
//! map.push(Pair::new(2, c2));
//!
//! let map = buf.insert_map(&mut map)?;
//! let buf = buf.as_aligned_buf();
//! let map = buf.bind(map)?;
//!
//! let c1 = map.get(&1)?.expect("Missing key 1");
//! let c1 = buf.load(c1)?;
//! assert_eq!(c1.field, 1);
//! assert_eq!(buf.load(c1.string)?, "string");
//!
//! let c2 = map.get(&2)?.expect("Missing key 2");
//! let c2 = buf.load(c2)?;
//! assert_eq!(c2.field, 2);
//! assert_eq!(buf.load(c2.string)?, "string");
//!
//! assert!(map.get(&3)?.is_none());
//! Ok::<_, musli_zerocopy::Error>(())
//! ```

#![no_std]
#![allow(clippy::module_inception)]
#![deny(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub use self::buf::{Buf, BufMut, Validator};
mod buf;

pub use self::error::Error;
mod error;

pub use self::ptr::Ptr;
mod ptr;

mod sip;

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

pub use self::zero_copy::{UnsizedZeroCopy, ZeroCopy};
mod zero_copy;

mod map;
pub use self::map::{Map, MapRef};

pub use self::pair::Pair;
mod pair;

pub use self::bind::Bindable;
mod bind;

/// Implement the [`ZeroCopy`] trait.
pub use musli_macros::ZeroCopy;
