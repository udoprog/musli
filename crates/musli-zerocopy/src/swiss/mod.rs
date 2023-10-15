//! A ZeroCopy [`Map`] and [`Set`] based on Google's SwizzTable algorithm.
//!
//! While this results in slower map representation than [`phf`], it is more
//! suitable for large data sets.
//!
//! [`phf`]: crate::phf

// Map internals copied from hashbrown under the MIT license.
//
// See:
// https://github.com/rust-lang/hashbrown/tree/3d2d1638d90053cb7d6a96090bc7c2bd2fd10d71

mod raw;

pub(crate) use self::entry::Entry;
mod entry;

pub use self::map::{Map, MapRef};
pub mod map;

pub use self::set::{Set, SetRef};
pub mod set;

#[cfg(feature = "alloc")]
mod constructor;

#[cfg(feature = "alloc")]
pub use self::factory::*;
#[cfg(feature = "alloc")]
mod factory;
