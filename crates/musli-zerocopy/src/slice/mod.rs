//! Utilities for working with stored slices.

#[doc(inline)]
pub use self::binary_search::{BinarySearch, binary_search, binary_search_by};
mod binary_search;

#[doc(inline)]
pub use self::slice::Slice;
mod slice;

#[doc(inline)]
pub use self::packed::Packed;
mod packed;
