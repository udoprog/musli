//! JSON support for [Müsli] suitable for network and usually browser
//! communication.
//!
//! JSON encoding is fully upgrade stable:
//!
//! * ✔ Can tolerate missing fields if they are annotated with
//!   `#[musli(default)]`.
//! * ✔ Can skip over unknown fields.
//!
//! [Müsli]: https://github.com/udoprog/musli
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
//! let version2 = musli::json::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(61),
//! })?;
//!
//! let version1: Version1 = musli::json::from_slice(version2.as_slice())?;
//!
//! assert_eq!(version1, Version1 {
//!     name: String::from("Aristotle"),
//! });
//! # Ok::<_, musli::json::Error>(())
//! ```

#![cfg(feature = "json")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "json")))]

mod de;
mod en;
mod encoding;
mod error;
mod parser;

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(hidden)]
pub mod test;

/// Convenient result alias for use with `musli::json`.
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(all(feature = "std", feature = "alloc"))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", feature = "alloc"))))]
#[doc(inline)]
pub use self::encoding::to_writer;
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use self::encoding::{decode, encode, from_slice, from_str, to_fixed_bytes, to_slice};
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use self::encoding::{to_string, to_vec};
#[doc(inline)]
pub use self::encoding::{Encoding, DEFAULT};
#[doc(inline)]
pub use self::error::Error;
pub use self::parser::Parser;
