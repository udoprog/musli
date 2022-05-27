//! Marker traits used for determining the format of an integer.

use core::marker;

use crate::int::NetworkEndian;

/// Type that indicates that the given numerical type should use variable-length
/// encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Variable {}

/// A fixed-length integer encoding which encodes something to a little-endian
/// encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct Fixed<B = NetworkEndian> {
    _marker: marker::PhantomData<B>,
}

/// A fixed-length encoding which encodes numbers to the width of `L` and the
/// endianness of `B`.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct FixedUsize<L = u32, B = NetworkEndian> {
    _marker: marker::PhantomData<(L, B)>,
}
