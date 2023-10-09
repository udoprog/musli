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
mod to_buf;
mod unsized_ref;

pub use self::buf::{Buf, Validator};
pub use self::error::Error;
#[cfg(feature = "alloc")]
pub use self::owned_buf::OwnedBuf;
pub use self::ref_::Ref;
pub use self::to_buf::{UnsizedZeroCopy, ZeroCopy};
pub use self::unsized_ref::UnsizedRef;

/// Implement the necessary traits for a type to be zero copy.
pub use musli_macros::ZeroCopy;
