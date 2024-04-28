//! Trait fills for `#[no_std]` environments.
//!
//! * [`ToOwned`] - if the `alloc` feature is enabled, this is an alias for
//!   `alloc::borrow::ToOwned`.
//! * [`Error`] - if the `std` feature is enabled, this is an alias for
//!   `std::error::Error`. If the `std` feature is disabled, this is a trait
//!   which is implemented for everything that implements [`Debug`] and
//!   [`Display`]. Note that this means that enabling the `std` feature might
//!   cause code that is designed carelessly to break due to no longer
//!   implementing the trait.
//!
//! [`Debug`]: core::fmt::Debug
//! [`Display`]: core::fmt::Display

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "alloc")]
pub use alloc::borrow::ToOwned;

#[cfg(not(feature = "alloc"))]
pub use self::to_owned::ToOwned;

#[cfg(feature = "std")]
pub use std::error::Error;

/// Standard error trait used when the `std` feature is not enabled.
#[cfg(not(feature = "std"))]
pub trait Error: fmt::Debug + fmt::Display {}

#[cfg(not(feature = "std"))]
impl<T> Error for T where T: fmt::Debug + fmt::Display {}

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
