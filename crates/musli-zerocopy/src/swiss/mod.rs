//! A ZeroCopy [`Map`] and [`Set`] based on Google's [SwissTable] algorithm.
//!
//! While this results in slower map representation than [`phf`], it is more
//! suitable for large data sets.
//!
//! This implementation is derived from the [`hashbrown` crate], but has been
//! modified to:
//! * Write directly into a pre-allocated buffer rather that allocate
//!   internally.
//! * Tolerate unexpected representations. Since a loaded variant of this table
//!   can tolerate most byte representations we need to ensure that the table at
//!   most errors and don't end up exhibiting some under-specified behavior like
//!   looping forever on lookups.
//!
//! [`phf`]: crate::phf
//! [SwissTable]: <https://abseil.io/about/design/swisstables>
//! [`hashbrown` crate]: https://crates.io/crates/hashbrown

/// Map internals copied from hashbrown under the Apache 2.0 OR MIT license.
///
/// Copyright (c) 2016 Amanieu d'Antras
///
/// See:
/// <https://github.com/rust-lang/hashbrown/tree/3d2d1638d90053cb7d6a96090bc7c2bd2fd10d71>.
mod raw;

pub(crate) use self::entry::Entry;
mod entry;

#[doc(inline)]
pub use self::map::{Map, MapRef};
pub mod map;

#[doc(inline)]
pub use self::set::{Set, SetRef};
pub mod set;

#[cfg(feature = "alloc")]
mod constructor;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use self::factory::*;
#[cfg(feature = "alloc")]
mod factory;
