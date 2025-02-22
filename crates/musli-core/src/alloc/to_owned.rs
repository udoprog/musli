use core::borrow::Borrow;

use crate::alloc::{Allocator, String, Vec};

/// The local `ToOwned`` implementation for Musli's allocation system.
pub trait ToOwned {
    /// The owned value.
    type Owned<A>: Borrow<Self>
    where
        A: Allocator;
}

impl<T> ToOwned for [T] {
    type Owned<A>
        = Vec<T, A>
    where
        A: Allocator;
}

impl ToOwned for str {
    type Owned<A>
        = String<A>
    where
        A: Allocator;
}
