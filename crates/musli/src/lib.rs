//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/musli/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)
//!
//! # Müsli
//!
//! Müsli is a flexible and generic binary serialization framework.
//!
//! **Müsli currently depends on [GATs] and is nightly-only**
//!
//! We make the following assumptions:
//!
//! * Anything being deserialized must be fully held in memory and able to hand
//!   out contiguous slices of it. This allows users of musli to perform
//!   zero-copy deserialization for bytes-oriented types.
//!
//! * Decoding is biased to assume strings are encoded verbatim in the format
//!   used so that references to strings can always be used. That means strings
//!   have to be UTF-8. A format that deviates from this will have to rely on
//!   runtime errors.
//!
//! I've chosen to internally use the term "encoding", "encode", and "decode"
//! because it's common terminology when talking about binary formats. It's also
//! distinct from [serde]'s use of "serialization" allowing for the ease of
//! using both libraries side by side if desired.
//!
//! <br>
//!
//! ## Design
//!
//! Müsli is designed with similar principles as [serde]. Relying on Rust's
//! powerful trait system to generate code which can largely be optimized away.
//! The end result should be very similar to a handwritten encoder / decoder.
//!
//! The central components of the framework are the [Encode] and [Decode]
//! derives. They are thoroughly documented in the [derives] module.
//!
//! <br>
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! musli = "0.0.8"
//! musli-wire = "0.0.8"
//! ```
//!
//! <br>
//!
//! ## Formats
//!
//! Formats are currently distinguished by supporting various degrees of
//! *upgrade stability*. A fully upgrade stable encoding format must tolerate
//! that one model can add fields that an older version of the model should be
//! capable of ignoring.
//!
//! Partial upgrade stability can still be useful as is the case of the
//! *musli-storage* format below, because reading from storage only requires
//! decoding to be upgrade stable. So if correctly managed with
//! `#[musli(default)]` this will never result in any readers seeing unknown
//! fields.
//!
//! The available formats and their capabilities are:
//!
//! | | reorder? | missing? | unknown? |
//! |-|-----------------|-----------------|--------------------|
//! | [musli-storage] `#[musli(packed)]` | ✗ | ✗ | ✗ |
//! | [musli-storage]                    | ✔ | ✔ | ✗ |
//! | [musli-wire]                       | ✔ | ✔ | ✔ |
//!
//! `recorder?` determines whether fields must occur in exactly the order in
//! which they are specified. So reordering fields in such a struct would cause
//! an error. This is only suitable for byte-oriented IPC where data models are
//! strictly synchronized.
//!
//! `missing?` determines if the reader can handle missing fields, as
//! exemplified above. This is suitable for on-disk storage.
//!
//! `unknown?` determines if the format can skip over unknown fields. This is
//! suitable for network communication.
//!
//! For every feature you drop, the format becomes more compact and efficient.
//! `musli-storage` `#[musli(packed)]` for example is roughly as compact and
//! efficient as [bincode] while [musli-wire] is comparable to something like
//! [protobuf]*.
//!
//! <br>
//!
//! # Examples
//!
//! The following is an example of *full upgrade stability* using [musli-wire]:
//!
//! ```rust
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
//! let version2 = musli_wire::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(62),
//! })?;
//!
//! let version1: Version1 = musli_wire::decode(&version2[..])?;
//!
//! assert_eq!(version1, Version1 {
//!     name: String::from("Aristotle"),
//! });
//! # Ok(()) }
//! ```
//!
//! The following is an example of *partial upgrade stability* using
//! [musli-storage]:
//!
//! ```rust
//! use musli::{Encode, Decode};
//!
//! # #[derive(Debug, PartialEq, Encode, Decode)]
//! # struct Version1 { name: String }
//! # #[derive(Debug, PartialEq, Encode, Decode)]
//! # struct Version2 { name: String, #[musli(default)] age: Option<u32> }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let version2 = musli_storage::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(62),
//! })?;
//!
//! assert!(musli_storage::decode::<_, Version1>(&version2[..]).is_err());
//!
//! let version1 = musli_storage::to_vec(&Version1 {
//!     name: String::from("Aristotle"),
//! })?;
//!
//! let version2: Version2 = musli_storage::decode(&version1[..])?;
//!
//! assert_eq!(version2, Version2 {
//!     name: String::from("Aristotle"),
//!     age: None,
//! });
//! # Ok(()) }
//! ```
//!
//! [bincode]: https://docs.rs/bincode
//! [Decode]: Decode
//! [derives]: derives
//! [Encode]: Encode
//! [GATs]: https://github.com/rust-lang/rust/issues/44265
//! [json-serde-value]: https://docs.rs/serde_json/latest/serde_json/enum.Value.html
//! [musli-storage]: https://docs.rs/musli-storage
//! [musli-wire]: https://docs.rs/musli-wire
//! [protobuf]: https://developers.google.com/protocol-buffers
//! [serde]: https://serde.rs

#![feature(generic_associated_types)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod compat;
pub mod de;
pub mod derives;
pub mod en;
pub mod error;
mod impls;
mod internal;

pub use self::de::{Decode, Decoder};
pub use self::en::{Encode, Encoder};
