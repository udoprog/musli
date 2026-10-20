//! CSV support for [Müsli].
//!
//! The API and semantics are modelled after the [`csv` crate], but CSV is
//! parsed and written by Müsli itself. Quoting follows [RFC 4180]: fields which
//! contain the delimiter, a quote or a line terminator are quoted, and quotes
//! inside of quoted fields are doubled up.
//!
//! The top-level value being encoded or decoded is a whole CSV document. A
//! sequence such as a [`Vec`] is a sequence of records, while anything else
//! such as a single struct is treated as a document with a single record.
//!
//! The supported records are:
//! * Structs with named fields, where each field is a column. If headers are
//!   enabled, which is the default, the names of fields are written as a header
//!   row and fields are matched by name against the header row when reading.
//! * Tuples, tuple structs and sequences, where each element is a column.
//! * Scalars, which are records with a single column.
//!
//! The values of fields can be integers, floats, booleans, strings, characters,
//! unit enum variants and [`Option`] where `None` is an empty field. Nested
//! containers such as maps, sequences or structs inside of a field raise an
//! error.
//!
//! CSV with headers is upgrade stable for structs:
//!
//! * ✔ Can tolerate missing fields if they are annotated with
//!   `#[musli(default)]`.
//! * ✔ Can skip over unknown fields.
//!
//! [Müsli]: https://github.com/udoprog/musli
//! [RFC 4180]: https://www.rfc-editor.org/rfc/rfc4180
//! [`csv` crate]: https://docs.rs/csv
//! [`Vec`]: rust_alloc::vec::Vec
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
//! let version2 = musli::csv::to_string(&vec![
//!     Version2 { name: String::from("Aristotle"), age: Some(61) },
//!     Version2 { name: String::from("Plato, the philosopher"), age: None },
//! ])?;
//!
//! assert_eq!(version2, "name,age\nAristotle,61\n\"Plato, the philosopher\",\n");
//!
//! let version1: Vec<Version1> = musli::csv::from_str(&version2)?;
//!
//! assert_eq!(version1, vec![
//!     Version1 { name: String::from("Aristotle") },
//!     Version1 { name: String::from("Plato, the philosopher") },
//! ]);
//! # Ok::<_, musli::csv::Error>(())
//! ```
//!
//! <br>
//!
//! ## Headers
//!
//! When headers are enabled, the header row is written before the first
//! record. For structs it contains the names of their fields, and for other
//! kinds of records it contains the position of each field starting at `0`.
//! This differs from the `csv` crate, which doesn't write a header row for
//! records without field names, but means that documents written with headers
//! enabled can always be read back.
//!
//! When reading with headers enabled, the first row is always treated as a
//! header row.
//!
//! ```
//! use musli::csv::Encoding;
//! # use musli::csv::Error;
//!
//! const NO_HEADERS: Encoding = Encoding::new().with_headers(false);
//!
//! let points = vec![(1, 2), (3, 4)];
//!
//! assert_eq!(musli::csv::to_string(&points)?, "0,1\n1,2\n3,4\n");
//! assert_eq!(NO_HEADERS.to_string(&points)?, "1,2\n3,4\n");
//!
//! let decoded: Vec<(u32, u32)> = musli::csv::from_str("x,y\n1,2\n3,4\n")?;
//! assert_eq!(decoded, points);
//! # Ok::<_, Error>(())
//! ```
//!
//! Since structs are encoded the same way twice when writing a header, fields
//! which are conditionally skipped using `#[musli(skip_encoding_if = ..)]` are
//! not supported, since that would cause records to have differing columns.
//!
//! <br>
//!
//! ## Reading and writing records
//!
//! The record-oriented [`Reader`] and [`Writer`] decode or encode one record
//! at a time:
//!
//! ```
//! use musli::{Decode, Encode};
//! use musli::csv::Encoding;
//! # use musli::csv::Error;
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//!
//! const ENCODING: Encoding = Encoding::new().with_delimiter(b';');
//!
//! let mut writer = ENCODING.writer(Vec::new());
//! writer.serialize(&Person { name: "Aristotle", age: 61 })?;
//! writer.serialize(&Person { name: "Plato", age: 80 })?;
//! let data = writer.into_inner();
//!
//! assert_eq!(data, b"name;age\nAristotle;61\nPlato;80\n");
//!
//! let mut reader = ENCODING.reader(&data);
//!
//! assert_eq!(reader.next_record()?, Some(Person { name: "Aristotle", age: 61 }));
//! assert_eq!(reader.next_record()?, Some(Person { name: "Plato", age: 80 }));
//! assert_eq!(reader.next_record::<Person<'_>>()?, None);
//! # Ok::<_, Error>(())
//! ```

#![cfg(feature = "csv")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "csv")))]

mod de;
mod en;
mod encoding;
mod error;
mod parser;
mod reader;
mod writer;

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(hidden)]
pub mod test;

#[cfg(test)]
mod tests;

/// Convenient result alias for use with `musli::csv`.
///
/// # Examples
///
/// ```
/// use musli::csv::{self, Result};
/// use musli::{Encode, Decode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// fn csv_roundtrip(people: &[Person]) -> Result<Vec<Person>> {
///     let csv_string = csv::to_string(people)?;
///     csv::from_str(&csv_string)
/// }
///
/// let original = vec![Person {
///     name: "Alice".to_string(),
///     age: 30
/// }];
///
/// let decoded = csv_roundtrip(&original)?;
/// assert_eq!(original, decoded);
/// # Ok::<_, musli::csv::Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(inline)]
pub use self::de::Headers;
#[doc(inline)]
pub use self::encoding::{Encoding, Terminator};
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use self::encoding::{decode, encode, from_slice, from_str, to_fixed_bytes, to_slice};
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[doc(inline)]
pub use self::encoding::{to_string, to_vec};
#[cfg(all(feature = "std", feature = "alloc"))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", feature = "alloc"))))]
#[doc(inline)]
pub use self::encoding::to_writer;
#[doc(inline)]
pub use self::error::Error;
pub use self::parser::{IntoParser, Parser};
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use self::reader::DeserializeRecords;
#[doc(inline)]
pub use self::reader::Reader;
#[doc(inline)]
pub use self::writer::Writer;
