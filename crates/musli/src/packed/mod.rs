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
//! information. Since the packed format doesn't use field identifiers, it only
//! supports optional fields *at the end* of the stream.
//!
//! See [`storage`] or [`wire`] or [`descriptive`] for formats which are upgrade
//! stable.
//!
//! Note that this is simply a specialization of the `storage` format with
//! different options. But it allows for much more efficient encoding.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! #[musli(packed)]
//! struct Version1 {
//!     name: String,
//! }
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! #[musli(packed)]
//! struct Version2 {
//!     name: String,
//!     #[musli(default)]
//!     age: Option<u32>,
//! }
//!
//! let version2 = musli::packed::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(61),
//! })?;
//!
//! let version1 = musli::packed::decode::<_, Version1>(version2.as_slice())?;
//! assert_eq!(version1.name, "Aristotle");
//!
//! let version1 = musli::packed::to_vec(&Version1 {
//!     name: String::from("Aristotle"),
//! })?;
//!
//! let version2: Version2 = musli::packed::decode(version1.as_slice())?;
//!
//! assert_eq!(version2, Version2 {
//!     name: String::from("Aristotle"),
//!     age: None,
//! });
//! # Ok::<_, musli::packed::Error>(())
//! ```
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
///
/// # Examples
///
/// ```
/// use musli::packed::{self, Result};
/// use musli::{Encode, Decode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Item {
///     id: u32,
///     name: String,
/// }
///
/// fn packed_roundtrip(item: &Item) -> Result<Item> {
///     let bytes = packed::to_vec(item)?;
///     packed::from_slice(&bytes)
/// }
///
/// let original = Item {
///     id: 1,
///     name: "Test".to_string(),
/// };
/// let decoded = packed_roundtrip(&original)?;
/// assert_eq!(original, decoded);
/// # Ok::<_, musli::packed::Error>(())
/// ```
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
pub use self::encoding::{Encoding, OPTIONS};
#[doc(inline)]
pub use self::error::Error;
