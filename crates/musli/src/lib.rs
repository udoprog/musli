//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/musli/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)
//!
//! # Müsli
//!
//! Müsli is a flexible and generic binary serialization framework.
//!
//! The central components of the framework are the [Encode] and [Decode]
//! derives. They are thoroughly documented in the [derives] module.
//!
//! **Müsli currently depends on [GATs] and is nightly-only**
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
//! The end result should be very similar to a handwritten encoding.
//!
//! The heavy lifting in user code is done through the [Encode] and [Decode]
//! derives. They are both documented in the [derives] module.
//!
//! Where Müsli differs in approach is that we don't make as heavy use of the
//! visitor pattern. Instead the encoding interacts with the framework through
//! encoding interfaces that describe "what it wants" and leverages GATs to make
//! the API efficient and ergonomic.
//!
//! <br>
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! musli = "0.0.32"
//! musli-wire = "0.0.32"
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
//! <br>
//!
//! ## Unsafety
//!
//! This library currently has two instances of unsafe:
//!
//! * A `mem::transcode` in `Tag::kind`. Which guarantees that converting into
//!   the `Kind` enum which is `#[repr(u8)]` is as efficient as possible. (Soon
//!   to be replaced with an equivalent safe variant).
//!
//! * A largely unsafe `SliceReader` which provides more efficient reading than
//!   the default `Reader` impl for `&[u8]` does (which uses split_at). Since it
//!   can perform most of the necessary comparisons directly on the pointers.
//!
//! <br>
//!
//! ## Performance
//!
//! > The following are the results of preliminary benchmarking and should be
//! > taken with a big grain of 🧂.
//!
//! Preliminary benchmarking indicates that Müsli roundtrip encodings for large
//! objects are about 10x faster than using JSON through serde, 5x faster than
//! `serde_cbor`, and 12% faster than bincode. Note that the JSON comparison
//! obviously isn't apples-to-apples since the Müsli encoding isn't
//! self-descriptive, but it's included here to give a general idea of how it
//! compares. CBOR and bincode on the other hand have *comparable*
//! configurations.
//!
//! For small objects the difference in encoding performance is even more
//! significant. Müsli producing code that's 100x faster than JSON **and** CBOR,
//! 20x faster than bincode (despite doing similarly oversized pre-allocation).
//! This holds for both the wire and storage format.
//!
//! ```text
//! json/roundtrip-large    time:   [91.263 us 91.756 us 92.239 us]   
//! cbor/roundtrip-large    time:   [51.289 us 51.696 us 52.215 us]
//! bincode/roundtrip-large time:   [10.225 us 10.328 us 10.431 us]
//! musli-storage/roundtrip-large                                                                             
//!                         time:   [9.0467 us 9.0881 us 9.1329 us]
//! musli-wire/roundtrip-large
//!                         time:   [11.906 us 11.933 us 11.964 us]                              
//!
//! cbor/roundtrip-small    time:   [138.40 ns 147.94 ns 158.60 ns]
//! json/roundtrip-small    time:   [137.06 ns 137.93 ns 139.16 ns]     
//! bincode/roundtrip-small time:   [16.978 ns 17.425 ns 18.057 ns]
//! musli-wire/roundtrip-small
//!                         time:   [1.0177 ns 1.0227 ns 1.0277 ns]
//! musli-storage/roundtrip-small                                                                             
//!                         time:   [802.38 ps 803.95 ps 805.65 ps]
//! ```
//!
//! Note that these bencmarks include no "waste", like extra unrecognized
//! fields. This is an area where Müsli's current encoding indeed is expected to
//! lag behind since it needs to perform a fair bit of work to walk over
//! unrecognized data.
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
mod expecting;
mod impls;
mod internal;
pub mod mode;
pub mod never;
#[cfg(not(feature = "std"))]
mod no_std;
#[cfg(feature = "std")]
#[path = "std.rs"]
mod no_std;
pub mod utils;

pub use self::de::{Decode, Decoder};
pub use self::en::{Encode, Encoder};
pub use self::mode::Mode;
