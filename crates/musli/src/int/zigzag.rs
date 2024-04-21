//! Generic [zigzag encoding] for integers.
//!
//! [zigzag encoding]: https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding

use super::{Signed, Unsigned};

/// Encode an integer into zig-zag encoding.
#[inline]
pub fn encode<T>(x: T) -> T::Unsigned
where
    T: Signed,
{
    (x >> (T::BITS - 1)).unsigned() ^ (x << 1).unsigned()
}

/// Decode an integer into zig-zag encoding.
#[inline]
pub fn decode<T>(x: T) -> T::Signed
where
    T: Unsigned,
{
    (x >> 1).signed() ^ -(x & T::ONE).signed()
}
