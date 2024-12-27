//! Efficient binary storage encoding for Müsli.
//!
//! The storage encoding is partially upgrade safe:
//!
//! * ✔ Can tolerate missing fields if they are annotated with
//!   `#[musli(default)]`.
//! * ✗ Cannot skip over extra unrecognized fields.
//!
//! This means that it's suitable as a storage format, since the data model only
//! evolves in one place. But unsuitable as a wire format since it cannot allow
//! clients to upgrade independent of each other.
//!
//! See [`wire`] or [`descriptive`] for formats which are upgrade stable.
//!
//! [`descriptive`]: crate::descriptive
//! [`wire`]: crate::wire
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Version1 {
//!     name: String,
//! }
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Version2 {
//!     name: String,
//!     #[musli(default)]
//!     age: Option<u32>,
//! }
//!
//! let version2 = musli::storage::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(61),
//! })?;
//!
//! assert!(musli::storage::decode::<_, Version1>(version2.as_slice()).is_err());
//!
//! let version1 = musli::storage::to_vec(&Version1 {
//!     name: String::from("Aristotle"),
//! })?;
//!
//! let version2: Version2 = musli::storage::decode(version1.as_slice())?;
//!
//! assert_eq!(version2, Version2 {
//!     name: String::from("Aristotle"),
//!     age: None,
//! });
//! # Ok::<_, musli::storage::Error>(())
//! ```
//!
//! <br>
//!
//! ## Configuring
//!
//! To tweak the behavior of the storage format you can use the [`Encoding`]
//! type:
//!
//! ```
//! use musli::{Encode, Decode};
//! use musli::mode::Binary;
//! use musli::options::{self, Options, Integer};
//! use musli::storage::Encoding;
//!
//! const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
//! const CONFIG: Encoding<OPTIONS> = Encoding::new().with_options();
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//!
//! let mut out = Vec::new();
//!
//! let expected = Person {
//!     name: "Aristotle",
//!     age: 61,
//! };
//!
//! CONFIG.encode(&mut out, &expected)?;
//! let actual = CONFIG.decode(&out[..])?;
//!
//! assert_eq!(expected, actual);
//! # Ok::<_, musli::storage::Error>(())
//! ```

#![cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
#![cfg_attr(doc_cfg, doc(cfg(feature = "storage")))]

pub(crate) mod de;
pub(crate) mod en;
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
