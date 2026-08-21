//! Support for the [SQLite JSONB] format in [Müsli].
//!
//! JSONB is the binary representation which SQLite uses internally for its
//! `json` functions. It carries exactly the same data model as JSON, but every
//! element is prefixed with a header giving its type and the size of its
//! payload, so a document can be traversed without scanning the parts of it
//! which are not of interest.
//!
//! The module is named after SQLite rather than after the format because the
//! format is defined by what SQLite does with it. It exists to exchange data
//! with an SQLite database, and anything encoded with it inherits the
//! limitations of the JSON data model. Reach for [`descriptive`] or [`wire`]
//! for a general purpose binary format.
//!
//! [`descriptive`]: crate::descriptive
//! [`wire`]: crate::wire
//!
//! Encoding is upgrade stable in the same way that [`json`] is:
//!
//! * ✔ Can tolerate missing fields if they are annotated with
//!   `#[musli(default)]`.
//! * ✔ Can skip over unknown fields.
//!
//! [Müsli]: https://github.com/udoprog/musli
//! [SQLite JSONB]: https://sqlite.org/draft/jsonb.html
//! [`json`]: crate::json
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
//! let version2 = musli::sqlite_jsonb::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(61),
//! })?;
//!
//! let version1: Version1 = musli::sqlite_jsonb::from_slice(version2.as_slice())?;
//!
//! assert_eq!(version1, Version1 {
//!     name: String::from("Aristotle"),
//! });
//! # Ok::<_, musli::sqlite_jsonb::Error>(())
//! ```
//!
//! <br>
//!
//! ## Interoperability
//!
//! The output is a JSONB blob as SQLite would store it, so it can be handed
//! straight to SQLite and read back with the `json` functions:
//!
//! ```
//! let blob = musli::sqlite_jsonb::to_vec(&(1u32, 2u32))?;
//! assert_eq!(blob, [0x4b, 0x13, b'1', 0x13, b'2']);
//! // SELECT json(?) with the blob above returns the text `[1,2]`.
//! # Ok::<_, musli::sqlite_jsonb::Error>(())
//! ```
//!
//! A blob goes into a column as it is, so a document can be written by this
//! encoder, queried and edited in SQL, and decoded again without ever being
//! rendered as text. The [`sqlite_jsonb` example] does all of that against an
//! in-memory database:
//!
//! ```text
//! cargo run -p musli --example sqlite_jsonb --features sqlite-jsonb
//! ```
//!
//! [`sqlite_jsonb` example]:
//!     https://github.com/udoprog/musli/blob/main/crates/musli/examples/sqlite_jsonb.rs
//!
//! <br>
//!
//! ## Implementation details
//!
//! Every element starts with a header of between 1 and 9 bytes. The lower four
//! bits of the first byte are the element type, and the upper four bits either
//! hold the size of the payload directly, for payloads of up to 11 bytes, or
//! say how many bytes of big-endian payload size follow.
//!
//! Numbers are stored as ASCII text, exactly as they would appear in a JSON
//! document. Integers use the `INT` type. Floats use the `FLOAT` type, except
//! for infinities and NaN which have no canonical JSON representation and are
//! stored as the JSON5 `FLOAT5` values `Infinity`, `-Infinity` and `NaN`.
//!
//! Strings are stored without delimiters. A string which needs no escaping to
//! be rendered as JSON uses the `TEXT` type, one which does uses `TEXTRAW`,
//! which stores it verbatim. The escaped `TEXTJ` and `TEXT5` types are
//! translated when decoding, since they are what SQLite produces when it
//! converts JSON text to JSONB.
//!
//! Like [`json`], byte arrays are encoded as arrays of numbers, and variants
//! are externally tagged, so they are encoded as an object with a single entry.

#![cfg(feature = "sqlite-jsonb")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "sqlite-jsonb")))]

#[cfg(all(test, feature = "alloc"))]
mod tests;

mod cursor;
mod de;
mod en;
mod encoding;
mod error;
mod parse;
mod tag;

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(hidden)]
pub mod test;

/// Convenient result alias for use with `musli::sqlite_jsonb`.
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(inline)]
pub use self::cursor::{Cursor, IntoCursor, MutSliceCursor, SliceCursor};
#[doc(inline)]
pub use self::encoding::Encoding;
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
pub use self::error::Error;
