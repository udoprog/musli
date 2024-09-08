//! Trait fills for `#[no_std]` environments.
//!
//! * [`ToOwned`] - if the `alloc` feature is enabled, this is an alias for
//!   `alloc::borrow::ToOwned`.

#[cfg(feature = "alloc")]
pub use rust_alloc::borrow::ToOwned;

#[cfg(not(feature = "alloc"))]
pub use self::to_owned::ToOwned;

#[cfg(not(feature = "alloc"))]
mod to_owned {
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

    /// Trait fill for ToOwned when we're in a `#[no_std]` environment.
    pub trait ToOwned {
        /// The value borrowed.
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
}
