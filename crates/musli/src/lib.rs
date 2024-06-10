//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli)
//!
//! Excellent performance, no compromises[^1]!
//!
//! Müsli is a flexible, fast, and generic binary serialization framework for
//! Rust, in the same vein as [`serde`].
//!
//! It provides a set of [formats](#formats), each with its own well-documented
//! set of features and tradeoffs. Every byte-oriented serialization method
//! including escaped formats like [`musli::json`] has full `#[no_std]` support
//! with or without `alloc`. And a particularly neat component providing
//! low-level refreshingly simple [zero-copy serialization][zerocopy].
//!
//! [^1]: As in Müsli should be able to do everything you need and more.
//!
//! <br>
//!
//! ## Overview
//!
//! * See [`derives`] to learn how to implement [`Encode`] and [`Decode`].
//! * See [`data_model`] to learn about the abstract data model of Müsli.
//! * See [benchmarks] and [size comparisons] to learn about the performance of
//!   this framework.
//! * See [`tests`] to learn how this library is tested.
//! * See [`musli::serde`] for seamless compatibility with [`serde`]. You might
//!   also be interested to learn how [Müsli is different][different].
//!
//! [different]: #müsli-is-different-from-serde
//!
//! <br>
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml` using the [format](#formats) you want
//! to use:
//!
//! ```toml
//! [dependencies]
//! musli = { version = "0.0.122", features = ["storage"] }
//! ```
//!
//! <br>
//!
//! ## Design
//!
//! The heavy lifting is done by the [`Encode`] and [`Decode`] derives which are
//! documented in the [`derives`] module.
//!
//! Müsli operates based on the schema represented by the types which implement
//! these traits.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Person {
//!     /* .. fields .. */
//! }
//! ```
//!
//! > **Note** by default a field is identified by its *numerical index* which
//! > would change if they are re-ordered. Renaming fields and setting a default
//! > naming policy can be done by configuring the [`derives`].
//!
//! The binary serialization formats provided aim to efficiently and accurately
//! encode every type and data structure available in Rust. Each format comes
//! with [well-documented tradeoffs](#formats) and aims to be fully memory safe
//! to use.
//!
//! Internally we use the terms "encoding", "encode", and "decode" because it's
//! distinct from [`serde`]'s use of "serialization", "serialize", and
//! "deserialize" allowing for the clearer interoperability between the two
//! libraries. Encoding and decoding also has more of a "binary serialization"
//! vibe, which more closely reflects the focus of this framework.
//!
//! Müsli is designed on similar principles as [`serde`]. Relying on Rust's
//! powerful trait system to generate code which can largely be optimized away.
//! The end result should be very similar to handwritten, highly optimized code.
//!
//! As an example of this, these two functions both produce the same assembly
//! (built with `--release`):
//!
//! ```
//! # use musli::{Decode, Encode};
//! # use musli::mode::Binary;
//! # use musli::options::{self, Options, Integer, ByteOrder};
//! # use musli::storage::Encoding;
//! # type Result<T, E = musli::storage::Error> = core::result::Result<T, E>;
//! const OPTIONS: Options = options::new()
//!     .with_integer(Integer::Fixed)
//!     .with_byte_order(ByteOrder::NATIVE)
//!     .build();
//!
//! const ENCODING: Encoding<OPTIONS> = Encoding::new().with_options();
//!
//! #[derive(Encode, Decode)]
//! #[musli(packed)]
//! pub struct Storage {
//!     left: u32,
//!     right: u32,
//! }
//!
//! fn with_musli(storage: &Storage) -> Result<[u8; 8]> {
//!     let mut array = [0; 8];
//!     ENCODING.encode(&mut array[..], storage)?;
//!     Ok(array)
//! }
//!
//! fn without_musli(storage: &Storage) -> Result<[u8; 8]> {
//!     let mut array = [0; 8];
//!     array[..4].copy_from_slice(&storage.left.to_ne_bytes());
//!     array[4..].copy_from_slice(&storage.right.to_ne_bytes());
//!     Ok(array)
//! }
//! ```
//!
//! <br>
//!
//! ## Müsli is different from [`serde`]
//!
//! **Müsli's data model does not speak Rust**. There are no
//! `serialize_struct_variant` methods which provides metadata about the type
//! being serialized. The [`Encoder`] and [`Decoder`] traits are agnostic on
//! this. Compatibility with Rust types is entirely handled using the [`Encode`]
//! and [`Decode`] derives in combination with [modes](#Modes).
//!
//! **We use GATs** to provide easier to use abstractions. GATs were not
//! available when serde was designed.
//!
//! **Everything is a [`Decoder`] or [`Encoder`]**. Field names are therefore
//! not limited to be strings or indexes, but can be named to [arbitrary
//! types][musli-name-type] if needed.
//!
//! **Visitor are only used when needed**. `serde` [completely uses visitors]
//! when deserializing and the corresponding method is treated as a "hint" to
//! the underlying format. The deserializer is then free to call any method on
//! the visitor depending on what the underlying format actually contains. In
//! Müsli, we swap this around. If the caller wants to decode an arbitrary type
//! it calls [`decode_any`]. The format can then either signal the appropriate
//! underlying type or call [`Visitor::visit_unknown`] telling the implementer
//! that it does not have access to type information.
//!
//! **We've invented [*moded encoding*](#Modes)** allowing the same Rust types
//! to be encoded in many different ways with much greater control over how
//! things encoded. By default we include the [`Binary`] and [`Text`] modes
//! providing sensible defaults for binary and text-based formats.
//!
//! **Müsli fully supports [no-std and no-alloc]** from the ground up without
//! compromising on features using safe and efficient [scoped allocations].
//!
//! **We support [detailed tracing]** when decoding for much improved
//! diagnostics of *where* something went wrong.
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
//! [`musli::storage`] format below, because reading from storage only requires
//! decoding to be upgrade stable. So if correctly managed with
//! `#[musli(default)]` this will never result in any readers seeing unknown
//! fields.
//!
//! The available formats and their capabilities are:
//!
//! | | `reorder` | `missing` | `unknown` | `self` |
//! |-|-|-|-|-|
//! | [`musli::storage`] `#[musli(packed)]` | ✗ | ✗ | ✗ | ✗ |
//! | [`musli::storage`]                    | ✔ | ✔ | ✗ | ✗ |
//! | [`musli::wire`]                       | ✔ | ✔ | ✔ | ✗ |
//! | [`musli::descriptive`]                | ✔ | ✔ | ✔ | ✔ |
//! | [`musli::json`] [^json]               | ✔ | ✔ | ✔ | ✔ |
//!
//! `reorder` determines whether fields must occur in exactly the order in which
//! they are specified in their type. Reordering fields in such a type would
//! cause unknown but safe behavior of some kind. This is only suitable for
//! communication where the data models of each client are strictly
//! synchronized.
//!
//! `missing` determines if reading can handle missing fields through something
//! like `Option<T>`. This is suitable for on-disk storage, because it means
//! that new optional fields can be added as the schema evolves.
//!
//! `unknown` determines if the format can skip over unknown fields. This is
//! suitable for network communication. At this point you've reached [*upgrade
//! stability*](#upgrade-stability). Some level of introspection is possible
//! here, because the serialized format must contain enough information about
//! fields to know what to skip which usually allows for reasoning about basic
//! types.
//!
//! `self` determines if the format is self-descriptive. Allowing the structure
//! of the data to be fully reconstructed from its serialized state. These
//! formats do not require models to decode and can be converted to and from
//! dynamic containers such as [`musli::value`] for introspection. Such formats
//! also allows for type-coercions to be performed, so that a signed number can
//! be correctly read as an unsigned number if it fits in the destination type.
//!
//! For every feature you drop, the format becomes more compact and efficient.
//! [`musli::storage`] using `#[musli(packed)]` for example is roughly as compact
//! as [`bincode`] while [`musli::wire`] is comparable in size to something like
//! [`protobuf`]. All formats are primarily byte-oriented, but some might
//! perform [bit packing] if the benefits are obvious.
//!
//! [^json]: This is strictly not a binary serialization, but it was implemented
//! as a litmus test to ensure that Müsli has the necessary framework features
//! to support it. Luckily, the implementation is also quite good!
//!
//! <br>
//!
//! ## Upgrade stability
//!
//! The following is an example of *full upgrade stability* using
//! [`musli::wire`]. `Version1` can be decoded from an instance of `Version2`
//! because it understands how to skip fields which are part of `Version2`.
//! We're also explicitly adding `#[musli(name = ..)]` to the fields to ensure
//! that they don't change in case they are re-ordered.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Version1 {
//!     #[musli(mode = Binary, name = 0)]
//!     name: String,
//! }
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Version2 {
//!     #[musli(mode = Binary, name = 0)]
//!     name: String,
//!     #[musli(mode = Binary, name = 1)]
//!     #[musli(default)]
//!     age: Option<u32>,
//! }
//!
//! let version2 = musli::wire::to_vec(&Version2 {
//!     name: String::from("Aristotle"),
//!     age: Some(61),
//! })?;
//!
//! let version1: Version1 = musli::wire::decode(version2.as_slice())?;
//! # Ok::<_, musli::wire::Error>(())
//! ```
//!
//! The following is an example of *partial upgrade stability* using
//! [`musli::storage`] on the same data models. Note how `Version2` can be
//! decoded from `Version1` but *not* the other way around making it suitable
//! for on-disk storage where the schema can evolve from older to newer
//! versions.
//!
//! ```
//! # use musli::{Encode, Decode};
//! # #[derive(Debug, PartialEq, Encode, Decode)]
//! # struct Version1 { name: String }
//! # #[derive(Debug, PartialEq, Encode, Decode)]
//! # struct Version2 { name: String, #[musli(default)] age: Option<u32> }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
//! # Ok(()) }
//! ```
//!
//! <br>
//!
//! ## Modes
//!
//! In Müsli in contrast to [`serde`] the same model can be serialized in
//! different ways. Instead of requiring the use of distinct models we support
//! implementing different *modes* for a single model.
//!
//! A mode is a type parameter, which allows for different attributes to apply
//! depending on which mode an encoder is configured to use. A mode can apply to
//! *any* musli attributes giving you a lot of flexibility.
//!
//! If a mode is not specified, an implementation will apply to all modes (`M`),
//! if at least one mode is specified it will be implemented for all modes which
//! are present in a model and [`Binary`]. This way, an encoding which uses
//! `Binary` which is the default mode should always work.
//!
//! For more information on how to configure modes, see [`derives`].
//!
//! Below is a simple example of how we can use two modes to provide two
//! completely different formats using a single struct:
//!
//! ```
//! use musli::{Decode, Encode};
//! use musli::json::Encoding;
//!
//! enum Alt {}
//!
//! #[derive(Decode, Encode)]
//! #[musli(mode = Alt, packed)]
//! #[musli(name_all = "name")]
//! struct Word<'a> {
//!     text: &'a str,
//!     teineigo: bool,
//! }
//!
//! const CONFIG: Encoding = Encoding::new();
//! const ALT_CONFIG: Encoding<Alt> = Encoding::new().with_mode();
//!
//! let word = Word {
//!     text: "あります",
//!     teineigo: true,
//! };
//!
//! let out = CONFIG.to_string(&word)?;
//! assert_eq!(out, r#"{"text":"あります","teineigo":true}"#);
//!
//! let out = ALT_CONFIG.to_string(&word)?;
//! assert_eq!(out, r#"["あります",true]"#);
//! # Ok::<_, musli::json::Error>(())
//! ```
//!
//! <br>
//!
//! ## Unsafety
//!
//! This is a non-exhaustive list of unsafe use in this crate, and why they are
//! used:
//!
//! * A `mem::transmute` in `Tag::kind`. Which guarantees that converting into
//!   the `Kind` enum which is `#[repr(u8)]` is as efficient as possible.
//!
//! * A largely unsafe `SliceReader` which provides more efficient reading than
//!   the default `Reader` impl for `&[u8]` does. Since it can perform most of
//!   the necessary comparisons directly on the pointers.
//!
//! * Some unsafety related to UTF-8 handling in `musli::json`, because we check
//!   UTF-8 validity internally ourselves (like `serde_json`).
//!
//! * `FixedBytes<N>`, which is a stack-based container that can operate over
//!   uninitialized data. Its implementation is largely unsafe. With it
//!   stack-based serialization can be performed which is useful in no-std
//!   environments.
//!
//! * Some `unsafe` is used for owned `String` decoding in all binary formats to
//!   support faster string processing through [`simdutf8`]. Disabling the
//!   `simdutf8` feature (enabled by default) removes the use of this unsafe.
//!
//! To ensure this library is correctly implemented with regards to memory
//! safety, extensive testing and fuzzing is performed using `miri`. See
//! [`tests`] for more information.
//!
//! <br>
//!
//! [`Binary`]: <https://docs.rs/musli/latest/musli/mode/enum.Binary.html>
//! [`bincode`]: <https://docs.rs/bincode>
//! [`data_model`]: <https://docs.rs/musli/latest/musli/help/data_model/index.html>
//! [`decode_any`]: https://docs.rs/musli/latest/musli/trait.Decoder.html#method.decode_any
//! [`Decode`]: <https://docs.rs/musli/latest/musli/de/trait.Decode.html>
//! [`Decoder`]: <https://docs.rs/musli/latest/musli/trait.Decoder.html>
//! [`derives`]: <https://docs.rs/musli/latest/musli/help/derives/index.html>
//! [`Encode`]: <https://docs.rs/musli/latest/musli/en/trait.Encode.html>
//! [`Encoder`]: <https://docs.rs/musli/latest/musli/trait.Encoder.html>
//! [`musli::descriptive`]: <https://docs.rs/musli/latest/musli/descriptive/index.html>
//! [`musli::json`]: <https://docs.rs/musli/latest/musli/json/index.html>
//! [`musli::serde`]: <https://docs.rs/musli/latest/musli/serde/index.html>
//! [`musli::storage`]: <https://docs.rs/musli/latest/musli/storage/index.html>
//! [`musli::value`]: <https://docs.rs/musli/latest/musli/value/index.html>
//! [`musli::wire`]: <https://docs.rs/musli/latest/musli/wire/index.html>
//! [`protobuf`]: <https://developers.google.com/protocol-buffers>
//! [`serde`]: <https://serde.rs>
//! [`simdutf8`]: <https://docs.rs/simdutf8>
//! [`tests`]: <https://github.com/udoprog/musli/tree/main/tests>
//! [`Text`]: <https://docs.rs/musli/latest/musli/mode/enum.Text.html>
//! [`Visitor::visit_unknown`]: https://docs.rs/musli/latest/musli/de/trait.Visitor.html#method.visit_unknown
//! [benchmarks]: <https://udoprog.github.io/musli/benchmarks/>
//! [bit packing]: <https://github.com/udoprog/musli/blob/main/crates/musli/src/descriptive/tag.rs>
//! [completely uses visitors]: https://docs.rs/serde/latest/serde/trait.Deserializer.html#tymethod.deserialize_u32
//! [detailed tracing]: <https://udoprog.github.io/rust/2023-05-22/abductive-diagnostics-for-musli.html>
//! [musli-name-type]: <https://docs.rs/musli/latest/musli/help/derives/index.html#musliname_type-->
//! [no-std and no-alloc]: <https://github.com/udoprog/musli/blob/main/no-std/examples/>
//! [scoped allocations]: <https://docs.rs/musli/latest/musli/trait.Context.html#tymethod.alloc>
//! [size comparisons]: <https://udoprog.github.io/musli/benchmarks/#size-comparisons>
//! [zerocopy]: <https://docs.rs/musli-zerocopy>

#![deny(missing_docs)]
#![allow(clippy::module_inception)]
#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc as rust_alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod macros;

#[cfg(test)]
mod tests;

#[cfg(feature = "json")]
mod dec2flt;

pub mod help;

pub mod de;
pub mod en;

#[doc(inline)]
pub use musli_core::hint;
#[doc(inline)]
pub use musli_core::mode;

/// This is an attribute macro that must be used when implementing a
/// [`Encoder`].
///
/// It is required to use because a [`Encoder`] implementation might introduce
/// new associated types in the future, and this [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::en::{Encoder, Encode};
///
/// struct MyEncoder<'a, C: ?Sized> {
///     value: &'a mut Option<u32>,
///     cx: &'a C,
/// }
///
/// #[musli::encoder]
/// impl<C: ?Sized + Context> Encoder for MyEncoder<'_, C> {
///     type Cx = C;
///     type Ok = ();
///
///     fn cx(&self) -> &C {
///         self.cx
///     }
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn encode<T>(self, value: T) -> Result<Self::Ok, C::Error>
///     where
///         T: Encode<Self::Mode>,
///     {
///         value.encode(self.cx, self)
///     }
///
///     fn encode_u32(self, value: u32) -> Result<(), Self::Error> {
///         *self.value = Some(value);
///         Ok(())
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_core::encoder;

/// This is an attribute macro that must be used when implementing a
/// [`Decoder`].
///
/// It is required to use because a [`Decoder`] implementation might introduce
/// new associated types in the future, and this is [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::{Decoder, Decode};
///
/// struct MyDecoder<'a, C: ?Sized> {
///     cx: &'a C,
/// }
///
/// #[musli::decoder]
/// impl<'de, C: ?Sized + Context> Decoder<'de> for MyDecoder<'_, C> {
///     type Cx = C;
///
///     fn cx(&self) -> &C {
///         self.cx
///     }
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self) -> Result<u32, Self::Error> {
///         Ok(42)
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_core::decoder;

/// This is an attribute macro that must be used when implementing a
/// [`Visitor`].
///
/// It is required to use because a [`Visitor`] implementation might introduce
/// new associated types in the future, and this is [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// [not yet supported]:
///     https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
/// [`Visitor`]: crate::de::Visitor
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::Visitor;
///
/// struct AnyVisitor;
///
/// #[musli::visitor]
/// impl<'de, C: ?Sized + Context> Visitor<'de, C> for AnyVisitor {
///     type Ok = ();
///
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(
///             f,
///             "value that can be decoded into dynamic container"
///         )
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_core::visitor;

#[doc(inline)]
pub use musli_core::{Context, Decode, Decoder, Encode, Encoder};

#[doc(hidden)]
pub use musli_core::__priv;

pub mod alloc;

pub mod descriptive;
pub mod json;
pub mod serde;
pub mod storage;
pub mod value;
pub mod wire;

pub mod context;

pub mod compat;

pub mod fixed;
#[doc(inline)]
pub use self::fixed::FixedBytes;

pub mod options;
#[doc(inline)]
pub use self::options::Options;

pub mod reader;
#[doc(inline)]
pub use self::reader::{IntoReader, Reader};

pub mod wrap;

pub mod writer;
#[doc(inline)]
pub use self::writer::Writer;

pub mod no_std;

mod int;
mod str;
