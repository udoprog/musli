use crate as musli_zerocopy;
use crate::ZeroCopy;

/// A pair of values which can be stored inside of any other zero copy
/// structure.
///
/// Note that this primarily exists because tuples are not support. The layout
/// of a tuple is `repr(Rust)`, so there is no way to construct legal references
/// to them.
#[derive(Debug, Clone, Copy, ZeroCopy)]
#[zero_copy(bounds = {A: ZeroCopy, B: ZeroCopy})]
#[repr(C)]
pub struct Pair<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> Pair<A, B> {
    /// Construct a new pair.
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}
