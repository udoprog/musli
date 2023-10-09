#![no_std]
#![allow(clippy::module_inception)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod buf;
mod error;

mod ptr;
mod sip;

#[cfg(feature = "alloc")]
pub use self::owned_buf::OwnedBuf;
#[cfg(feature = "alloc")]
mod owned_buf;

pub use self::ref_::Ref;
mod ref_;

pub use self::slice_ref::SliceRef;
mod slice_ref;

pub use self::unsized_ref::UnsizedRef;
mod unsized_ref;

pub use self::zero_copy::{UnsizedZeroCopy, ZeroCopy};
mod zero_copy;

/// Implement the [`ZeroCopy`] trait.
pub use musli_macros::ZeroCopy;

pub use self::buf::{Buf, Validator};
pub use self::error::Error;

mod map;
pub use self::map::MapRef;

pub use self::pair::Pair;
mod pair;
