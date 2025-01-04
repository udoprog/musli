use core::borrow::Borrow;

use crate::alloc::{Allocator, String, Vec};

/// The local `ToOwned`` implementation for Musli's allocation system.
pub trait ToOwned<A>
where
    A: Allocator,
{
    /// The owned value.
    type Owned: Borrow<Self>;
}

impl<T, A> ToOwned<A> for [T]
where
    A: Allocator,
{
    type Owned = Vec<T, A>;
}

impl<A> ToOwned<A> for str
where
    A: Allocator,
{
    type Owned = String<A>;
}
