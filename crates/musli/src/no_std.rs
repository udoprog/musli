//! Fills for no_std to make the framework work.

use core::borrow::Borrow;

/// Never type for [ToOwned] so that `Owned` can reference some type even if
/// it's uninhabitable.
pub enum NeverOwned {}

impl<T> Borrow<[T]> for NeverOwned
where
    T: Clone,
{
    fn borrow(&self) -> &[T] {
        match *self {}
    }
}

impl<T> Borrow<T> for NeverOwned
where
    T: Clone,
{
    fn borrow(&self) -> &T {
        match *self {}
    }
}

impl Borrow<str> for NeverOwned {
    fn borrow(&self) -> &str {
        match *self {}
    }
}

/// Trait fill for ToOwned when we're in a no_std environment.
pub trait ToOwned {
    type Owned: Borrow<Self>;
}

impl<T> ToOwned for [T]
where
    T: Clone,
{
    type Owned = NeverOwned;
}

impl<T> ToOwned for T
where
    T: Clone,
{
    type Owned = NeverOwned;
}

impl ToOwned for str {
    type Owned = NeverOwned;
}
