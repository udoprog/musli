#![no_std]
#![allow(clippy::module_inception)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod buf;
mod error;
mod map;
#[cfg(feature = "alloc")]
mod owned_buf;
mod ptr;
mod sip;
mod slice;
mod traits;
mod unsized_ref;
mod value_ref;

pub use self::buf::Buf;
pub use self::error::Error;
pub use self::map::{Map, MapBuilder, MapRef};
#[cfg(feature = "alloc")]
pub use self::owned_buf::OwnedBuf;
#[cfg(feature = "build")]
pub use self::slice::SliceBuilder;
pub use self::slice::{Slice, SliceRef};
pub use self::unsized_ref::UnsizedRef;
pub use self::value_ref::ValueRef;
