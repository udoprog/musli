use crate::ZeroCopy;

/// A pair of values which can be stored inside of any other zero copy
/// structure.
///
/// Note that this primarily exists because tuples are not support. The layout
/// of a tuple is `repr(Rust)`, so there is no way to construct legal references
/// to them.
#[derive(Debug, ZeroCopy)]
#[zero_copy(crate = crate, bounds = {A: ZeroCopy, B: ZeroCopy})]
#[repr(C)]
pub struct Pair<A, B> {
    /// The first element in the pair.
    pub a: A,
    /// The second element in the pair.
    pub b: B,
}

impl<A, B> Pair<A, B> {
    /// Construct a new pair.
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}
