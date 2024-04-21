//! Fully upgrade stable format for [Müsli] suitable for network communication.
//!
//! [Müsli]: https://docs.rs/musli
//!
//! Wire encoding is fully upgrade stable:
//!
//! * ✔ Can tolerate missing fields if they are annotated with
//!   `#[musli(default)]`.
//! * ✔ Can skip over unknown fields.
//!
//! This means that it's suitable as a wire format, since the data model can
//! evolve independently among clients. Once some clients are upgraded they will
//! start sending unknown fields which non-upgraded clients will be forced to
//! skip over for the duration of the upgrade.
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
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let version2 = musli::wire::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(62),
//! })?;
//!
//! let version1: Version1 = musli::wire::decode(version2.as_slice())?;
//!
//! assert_eq!(version1, Version1 {
//!     name: String::from("Aristotle"),
//! });
//! # Ok(()) }
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
//! use musli::options::{self, Options, Integer};
//! use musli::wire::Encoding;
//!
//! const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
//! const CONFIG: Encoding<OPTIONS> = Encoding::new().with_options();
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
//! # Ok::<_, musli::wire::Error>(())
//! ```
//!
//! <br>
//!
//! ## Implementation details
//!
//! Each field is prefix *typed* with a single byte tag that allows a receiver
//! to figure out exactly how much should be skipped over.

#![cfg(feature = "wire")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "wire")))]

#[cfg(test)]
mod tests;

mod de;
mod en;
mod encoding;
mod error;
mod int;
mod tag;

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(hidden)]
#[macro_use]
pub mod test;

/// Convenient result alias for use with `musli::wire`.
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use self::encoding::to_vec;
#[doc(inline)]
#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
pub use self::encoding::to_writer;
#[doc(inline)]
pub use self::encoding::{decode, encode, from_slice, to_fixed_bytes, Encoding, OPTIONS};
#[doc(inline)]
pub use self::error::Error;

/// The maximum length that can be inlined in the tag without adding additional
/// data to the wire format.
#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
pub const MAX_INLINE_LEN: usize = (self::tag::DATA_MASK - 1) as usize;
