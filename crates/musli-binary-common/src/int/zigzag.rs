//! Generic [zigzag encoding] for integers.
//!
//! [zigzag encoding]: https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding
//!
//! ```rust
//! assert_eq!(musli_binary_common::int::zigzag::encode(-1i32), 1u32);
//! assert_eq!(musli_binary_common::int::zigzag::encode(-2i32), 3u32);
//! ```

use crate::int::{Signed, Unsigned};

/// Encode an integer into zig-zag encoding.
pub fn encode<T>(x: T) -> T::Unsigned
where
    T: Signed,
{
    (x >> (T::BITS - 1)).unsigned() ^ (x << 1).unsigned()
}

/// Decode an integer into zig-zag encoding.
pub fn decode<T>(x: T) -> T::Signed
where
    T: Unsigned,
{
    (x >> 1).signed() ^ -(x & T::ONE).signed()
}
