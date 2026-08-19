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
//!
//! <br>
//!
//! ## Pretty printing
//!
//! Encoding signals the structure of the document it is writing to the
//! [`Writer`] it is using through methods such as [`Writer::begin_object`] and
//! [`Writer::begin_array_element`]. These default to doing nothing, which is
//! what a compact document wants. Give an [`Encoding`] a [`Pretty`]
//! configuration and it wraps its output in a writer which uses those signals
//! to insert newlines and indentation instead:
//!
//! ```
//! use musli::Encode;
//! use musli::json::{Encoding, Pretty};
//!
//! #[derive(Encode)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! const PRETTY: Encoding = Encoding::new().with_pretty(Pretty::new());
//!
//! let person = Person {
//!     name: String::from("Aristotle"),
//!     age: 61,
//! };
//!
//! assert_eq!(PRETTY.to_string(&person)?, r#"{
//!   "name": "Aristotle",
//!   "age": 61
//! }"#);
//! # Ok::<_, musli::json::Error>(())
//! ```
//!
//! The [`to_string_pretty`], [`to_vec_pretty`], [`to_slice_pretty`], and
//! [`to_writer_pretty`] functions are shorthands for doing the same with the
//! default [`Encoding`].
//!
//! [`Writer::begin_array_element`]: crate::Writer::begin_array_element
//! [`Writer::begin_object`]: crate::Writer::begin_object
//! [`Writer`]: crate::Writer

#![cfg(feature = "json")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "json")))]

mod de;
mod en;
mod encoding;
mod error;
mod parser;
mod pretty_writer;

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(hidden)]
pub mod test;

/// Convenient result alias for use with `musli::json`.
///
/// # Examples
///
/// ```
/// use musli::json::{self, Result};
/// use musli::{Encode, Decode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// fn json_roundtrip(person: &Person) -> Result<Person> {
///     let json_string = json::to_string(person)?;
///     json::from_str(&json_string)
/// }
///
/// let original = Person {
///     name: "Alice".to_string(),
///     age: 30
/// };
/// let decoded = json_roundtrip(&original)?;
/// assert_eq!(original, decoded);
/// # Ok::<_, musli::json::Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(inline)]
pub use self::encoding::Encoding;
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use self::encoding::{
    decode, encode, from_slice, from_str, to_fixed_bytes, to_slice, to_slice_pretty,
};
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use self::encoding::{to_string, to_string_pretty, to_vec, to_vec_pretty};
#[cfg(all(feature = "std", feature = "alloc"))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", feature = "alloc"))))]
#[doc(inline)]
pub use self::encoding::{to_writer, to_writer_pretty};
#[doc(inline)]
pub use self::error::Error;
pub use self::parser::Parser;
pub use self::pretty_writer::Pretty;
