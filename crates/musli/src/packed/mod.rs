//! The most efficient binary storage encoding for Müsli.
//!
//! The packed encoding is not upgrade safe:
//!
//! * ✗ Can not tolerate missing fields.
//! * ✗ Cannot skip over extra unrecognized fields.
//!
//! This means that it's probably not suitable as a storage format, nor as a
//! wire since it cannot allow clients to upgrade independent of each other.
//!
//! In order to make full use of the packed format, the data model should use
//! the `#[musli(packed)]` attribute on the container. This among other things
//! prevents field identifiers from being emitted. See [`derives`] for more
//! information.
//!
//! See [`storage`] or [`wire`] or [`descriptive`] for formats which are upgrade
//! stable.
//!
//! Note that this is simply a specialization of the `storage` format with
//! different options. But it allows for much more efficient encoding.
//!
//! [`storage`]: crate::storage
//! [`descriptive`]: crate::descriptive
//! [`wire`]: crate::wire
//! [`derives`]: crate::_help::derives

#![cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
#![cfg_attr(doc_cfg, doc(cfg(feature = "storage")))]

mod encoding;
mod error;

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(hidden)]
pub mod test;

/// Convenient result alias for use with `musli::storage`.
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use self::encoding::to_vec;
#[cfg(all(feature = "std", feature = "alloc"))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", feature = "alloc"))))]
#[doc(inline)]
pub use self::encoding::to_writer;
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use self::encoding::{decode, encode, from_slice, to_fixed_bytes, to_slice};
#[doc(inline)]
pub use self::encoding::{Encoding, DEFAULT, OPTIONS};
#[doc(inline)]
pub use self::error::Error;
