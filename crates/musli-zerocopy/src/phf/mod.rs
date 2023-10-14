//! A ZeroCopy [`Map`] and [`Set`] based on a perfect hash functions.
//!
//! While these maps can be incredibly performant, they can be incredibly
//! expensive to build. So avoid these if you're storing many elements.
//!
//! For very large maps and sets, prefer to use [`swiss`] instead.
//!
//! [`swiss`]: crate::swiss

// Map internals copied from rust-phf under the MIT license.
//
// See:
// https://github.com/rust-phf/rust-phf/tree/b7116ff519415d302c070aa313831cd473b1a911

#[cfg(feature = "alloc")]
pub(crate) mod generator;

pub(crate) mod hashing;

pub(crate) use self::entry::Entry;
mod entry;

#[doc(inline)]
pub use self::map::{Map, MapRef};
pub mod map;

#[doc(inline)]
pub use self::set::{Set, SetRef};
pub mod set;

pub use self::factory::*;
mod factory;
