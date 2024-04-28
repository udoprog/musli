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

#[doc(inline)]
pub use musli_core::no_std::{Error, ToOwned};

/// A somewhat portable, but also noisy abort implementation for no_std
/// environments.
///
/// While this should ultimately cause the process to abort, it will first cause
/// the process to panic and report it through the panic hook.
#[cold]
#[cfg(not(feature = "std"))]
pub(crate) fn abort(s: &'static str) -> ! {
    struct Abort;

    // A panic during an unwinding drop leads to an abort.
    impl Drop for Abort {
        #[inline(always)]
        fn drop(&mut self) {
            panic!()
        }
    }

    let _a = Abort;
    panic!("{s}")
}

#[cfg(feature = "std")]
#[cold]
pub(crate) fn abort(_: &'static str) -> ! {
    ::std::process::abort();
}
