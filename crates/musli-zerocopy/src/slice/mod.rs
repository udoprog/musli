//! Utilities for working with stored slices.

pub use self::binary_search::{binary_search, binary_search_by, BinarySearch};
mod binary_search;

pub use self::slice::Slice;
mod slice;

pub use self::packed::Packed;
mod packed;
