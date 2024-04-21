//! Trait fills.
//!
//! This will replace the following trait with an unimplementable mock in
//! `#[no_std]` environments:
//!
//! * [`ToOwned`]

#[doc(inline)]
pub use alloc::borrow::ToOwned;
