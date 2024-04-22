//! A fully self-descriptive format for [Müsli].
//!
//! Descriptive encoding is fully upgrade stable:
//!
//! * ✔ Can tolerate missing fields if they are annotated with
//!   `#[musli(default)]`.
//! * ✔ Can skip over unknown fields.
//! * ✔ Can be fully converted back and forth between dynamic containers such as
//!   the [`Value`] type.
//! * ✔ Can handle coercion from different types of primitive types, such as
//!   signed to unsigned integers. So primitive field types can be assuming they
//!   only inhabit compatible values.
//!
//! [Müsli]: https://docs.rs/musli
//! [`Value`]: crate::value
//!
//! This means that it's suitable as a wire and general interchange format. It's
//! also suitable for dynamically translating to and from different wire formats
//! such as JSON without having access to the data model.
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
//! let version2 = musli::descriptive::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(62),
//! })?;
//!
//! let version1: Version1 = musli::descriptive::decode(version2.as_slice())?;
//!
//! assert_eq!(version1, Version1 {
//!     name: String::from("Aristotle"),
//! });
//! # Ok::<_, musli::descriptive::Error>(())
//! ```
//!
//! <br>
//!
//! ## Configuring
//!
//! To configure the behavior of the wire format you can use the [`Encoding`]
//! type:
//!
//! ```
//! use musli::{Encode, Decode};
//! use musli::descriptive::Encoding;
//!
//! const CONFIG: Encoding = Encoding::new();
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Struct<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//!
//! let mut out = Vec::new();
//!
//! let expected = Struct {
//!     name: "Aristotle",
//!     age: 61,
//! };
//!
//! CONFIG.encode(&mut out, &expected)?;
//! let actual = CONFIG.decode(&out[..])?;
//!
//! assert_eq!(expected, actual);
//! # Ok::<_, musli::descriptive::Error>(())
//! ```
//!
//! <br>
//!
//! ## Implementation details
//!
//! Each field is prefix *typed* with a single byte tag that describes exactly
//! the type which is contained in the field.

#![cfg(feature = "descriptive")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "descriptive")))]

#[cfg(test)]
mod tests;

mod de;
mod en;
mod encoding;
mod error;
mod integer_encoding;
mod tag;

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(hidden)]
#[macro_use]
pub mod test;

/// Convenient result alias for use with `musli::descriptive`.
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use self::encoding::to_vec;
#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[doc(inline)]
pub use self::encoding::to_writer;
#[doc(inline)]
pub use self::encoding::{decode, encode, from_slice, to_fixed_bytes, Encoding, OPTIONS};
#[doc(inline)]
pub use self::error::Error;

/// The maximum length that can be inlined in the tag without adding additional
/// data to the wire format.
#[cfg(test)]
pub(crate) const MAX_INLINE_LEN: usize = (self::tag::DATA_MASK - 1) as usize;
