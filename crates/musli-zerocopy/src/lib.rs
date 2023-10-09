#![no_std]
#![allow(clippy::module_inception)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod buf;
mod error;
#[cfg(feature = "alloc")]
mod owned_buf;
mod ptr;
mod ref_;
mod sip;
mod slice_ref;
mod unsized_ref;
mod zero_copy;

pub use self::buf::{Buf, CowBuf, Validator};
pub use self::error::Error;
#[cfg(feature = "alloc")]
pub use self::owned_buf::OwnedBuf;
pub use self::ref_::Ref;
pub use self::unsized_ref::UnsizedRef;
pub use self::zero_copy::{SliceZeroCopy, UnsizedZeroCopy, ZeroCopy};

pub use self::slice_ref::SliceRef;

/// Implement the necessary traits for a type to be zero copy.
pub use musli_macros::ZeroCopy;
