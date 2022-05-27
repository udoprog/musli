//! # The `Encode` and `Decode` derives
//!
//! The `Encode` and `Decode` derives allows for automatically implementing
//! [Encode] and [Decode].
//!
//! They come with a number of options for customizing their implementation,
//! detailed below. But first we need to talk about *modes*.
//!
//! #### Modes
//!
//! If you've paid close attention to the [Encode] and [Decode] traits you
//! might've noticed that they have an extra parameter called `M` for "mode".
//!
//! This allows a single type to have *more than one* implementation of encoding
//! traits, allowing for a high level of flexibility in how a type should be
//! encoded.
//!
//! When it comes to deriving these traits you can scope attributes to apply to
//! either any mode, the [default mode], or a completely custom mode. This is
//! done using the `#[musli(mode = ..)]` attribute like this:
//!
//! ```
//! use musli::{Encode, Decode, Mode};
//!
//! enum Json {}
//!
//! impl Mode for Json {
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_name = "index")]
//! #[musli(mode = Json, default_field_name = "name")]
//! struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```
//!
//! What this means is that if we want to serialize `Person` using named fields,
//! we can simply turn on the `Json` mode for our given serializer. If we want
//! to revert back to the default behavior and use indexed fields we can instead
//! use [DefaultMode].
//!
//! ```
//! # use musli::{Encode, Decode};
//! # enum Json {}
//! # impl musli::mode::Mode for Json {}
//! # #[derive(Encode, Decode)]
//! # #[musli(mode = Json, default_field_name = "name")]
//! # struct Person<'a> { name: &'a str, age: u32 }
//! use musli_json::Encoding;
//!
//! const JSON_ENCODING: Encoding<Json> = Encoding::new();
//! const DEFAULT_ENCODING: Encoding = Encoding::new();
//!
//! let named = JSON_ENCODING.to_buffer(&Person { name: "Aristotle", age: 62 })?;
//! assert_eq!(named.as_slice(), b"{\"name\":\"Aristotle\",\"age\":62}");
//!
//! let indexed = DEFAULT_ENCODING.to_buffer(&Person { name: "Plato", age: 84 })?;
//! assert_eq!(indexed.as_slice(), b"{\"0\":\"Plato\",\"1\":84}");
//!
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! So the `#[musli(mode)]` atttribute is supported in any position. And any of
//! its sibling attributes will be added to the given *alternative* mode, rather
//! the [default mode].
//!
//! #### Attributes
//!
//! * *Container attributes* are attributes which apply to the `struct` or
//!   `enum`. Like the uses of `#[musli(packed)]` and
//!   `#[musli(default_variant_name = "name")]` here:
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(packed)]
//! struct Struct {
//!     /* the body of the struct */
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_variant_name = "name")]
//! enum Enum {
//!     /* the body of the struct */
//! }
//! ```
//!
//! * *Variant attributes* are attributes which apply to each individual variant
//!   in an `enum`. Like the use of `#[musli(name)]` here:
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_variant_name = "name")]
//! enum Enum {
//!     #[musli(rename = "Other")]
//!     Something {
//!         /* variant body */
//!     }
//! }
//! ```
//!
//! * *Field attributes* are attributes which apply to each individual field
//!   either in a `struct` or an `enum` variant. Like the uses of
//!   `#[musli(rename)]` here:
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_name = "name")]
//! struct Struct {
//!     #[musli(rename = "other")]
//!     something: String,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_name = "name")]
//! enum Enum {
//!     Variant {
//!         #[musli(rename = "other")]
//!         something: String,
//!     }
//! }
//! ```
//!
//! ## Container attributes
//!
//! * `#[musli(default_field_name = "..")]` determines how the default tag for a
//!   field is determined. It can take either `"name"` or `"index"`.
//!
//!   `#[musli(default_field_name = "index")]` will use the index of the field.
//!   This is the default.
//!
//!   `#[musli(default_field_name = "name")]` will use the name of the field.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_name = "name")]
//! struct Struct {
//!     field1: u32,
//!     field2: u32,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_name = "name")]
//! enum Enum {
//!     Variant1 {
//!         field1: u32,
//!     },
//!     Variant2 {
//!         field1: u32,
//!     },
//! }
//! ```
//!
//! * `#[musli(default_variant_name = "..")]` determines how the default tag for
//!   a variant is determined. It can take either `"name"` or `"index"`.
//!
//!   `#[musli(default_variant_name = "index")]` will use the index of the
//!   variant. This is the default.
//!
//!   `#[musli(default_variant_name = "name")]` will use the name of the variant.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_variant_name = "name")]
//! enum Enum {
//!     Variant1 {
//!         field1: u32,
//!     },
//!     Variant2 {
//!         field1: u32,
//!     },
//! }
//! ```
//!
//! * `#[musli(transparent)]` can only be used on types which have a single
//!   field. It will cause that field to define how that variant is encoded or
//!   decoded transparently without being treated as a field.
//!
//! ```
//! use musli::{Encode, Decode};
//! use musli_wire::tag::{Tag, Kind};
//!
//! #[derive(Encode)]
//! #[musli(transparent)]
//! struct Struct(u32);
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = musli_wire::to_buffer(&Struct(42))?;
//! assert_eq!(data.as_slice(), vec![Tag::new(Kind::Continuation, 42).byte()]);
//! # Ok(()) }
//! ```
//!
//! * `#[musli(packed)]` this attribute will disable all *tagging* and the
//!   structure will simply be encoded with one field following another in the
//!   order in which they are defined.
//!
//!   A caveat of *packed* structures is that they cannot be safely versioned
//!   and the two systems communicating through them need to be using strictly
//!   synchronized representations.
//!
//!   This attribute is useful for performing simple decoding over "raw" bytes
//!   when combined with an encoder which does minimal prefixing and packs
//!   fields.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode)]
//! #[musli(packed)]
//! struct Struct {
//!     field1: u32,
//!     field2: u32,
//!     field3: u32,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = musli_storage::to_buffer(&Struct {
//!     field1: 1,
//!     field2: 2,
//!     field3: 3,
//! })?;
//!
//! assert_eq!(data.as_slice(), vec![1, 2, 3]);
//! # Ok(()) }
//! ```
//!
//! * `#[musli(name_type = ..)]` indicates which type any contained `#[musli(tag
//!   = ..)]` attributes should have. Tags can usually be inferred, but
//!   specifying this field ensures that all tags have a well-defined type.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//! #[musli(transparent)]
//! struct CustomTag<'a>(&'a [u8]);
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_type = CustomTag)]
//! struct Struct {
//!     #[musli(rename = CustomTag(b"name in bytes"))]
//!     name: String,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_type = CustomTag)]
//! enum EnumWithCustomTag {
//!     #[musli(rename = CustomTag(b"variant one"))]
//!     Variant1 {
//!         /* .. */
//!     },
//! }
//! ```
//!
//! * `#[musli(bound = ..)]` and `#[musli(decode_bound = ..)]` can be used to
//!   apply bounds to an [Encode] or [Decode] implementation.
//!
//!   These are necessary to use when a generic container is used to ensure that
//!   the given parameter implements the necessary bounds.
//!
//!   `#[musli(bound = ..)]` applies to all implementations while
//!   `#[musli(decode_bound = ..)]` only applies to the [Decode] implementation.
//!   The latter allows for using the decode lifetime parameter (which defaults
//!   to `'de`).
//!
//! ```
//! use musli::{Decode, Encode};
//!
//! #[derive(Clone, Debug, PartialEq, Encode, Decode)]
//! #[musli(bound = T: Encode<M>, decode_bound = T: Decode<'de, M>)]
//! pub struct GenericWithBound<T> {
//!     value: T,
//! }
//! ```
//!
//! ## Enum attributes
//!
//! * `#[musli(tag = ..)]` Use the internally tagged enum representation for
//!   this enum, with the given tag. See [enum
//!   representations](#enum-representations) for details on this
//!   representation.
//!
//! ## Variant attributes
//!
//! * `#[musli(rename = ..)]` allows for renaming a variant from its default value.
//!   It can take any value (including complex ones) that can be serialized with
//!   the current encoding, such as:
//!
//!   * `#[musli(rename = 1)]`
//!   * `#[musli(rename = "Hello World")]`
//!   * `#[musli(rename = b"box\0")]`
//!   * `#[musli(rename = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!     `Encode` and `Decode` as appropriate).
//!
//!   If the type of the tag is ambiguous it can be explicitly specified through
//!   the `#[musli(name_type)]` container attribute (see above).
//!
//! * `#[musli(default_field_name = "..")]` determines how the default tag for a
//!   field in the current variant is determined. This overrides the tagging
//!   convention specified on the *container* and can take either `"name"` or
//!   `"index"`.
//!
//!   `#[musli(default_field_name = "index")]` will use the index of the field.
//!   This is the default.
//!
//!   `#[musli(default_field_name = "name")]` will use the name of the field.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_name = "index")]
//! enum Enum {
//!     #[musli(default_field_name = "name")]
//!     Variant {
//!         field1: u32,
//!     },
//!     Variant2 {
//!         field1: u32,
//!     },
//! }
//! ```
//!
//! * `#[musli(transparent)]` can only be used on variants which have a single
//!   field. It will cause that field to define how that variant is encoded or
//!   decoded transparently without being treated as a field.
//!
//! * `#[musli(name_type = ..)]` indicates which type any contained `#[musli(tag
//!   = ..)]` attributes should have. Tags can usually be inferred, but
//!   specifying this field ensures that all tags have a well-defined type.
//!
//!   This attribute takes priority over the one with the same name on the
//!   container.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//! #[musli(transparent)]
//! struct CustomTag<'a>(&'a [u8]);
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_type = usize)]
//! enum Enum {
//!     #[musli(name_type = CustomTag)]
//!     Variant {
//!         #[musli(rename = CustomTag(b"name in bytes"))]
//!         name: String,
//!     }
//! }
//! ```
//!
//! * `#[musli(default)]` defines the variant that will be used in case no other
//!   variant matches. Only one such variant can be defined.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//! enum Animal {
//!     #[musli(rename = "cat")]
//!     Cat,
//!     #[musli(rename = "dog")]
//!     Dog,
//!     #[musli(default)]
//!     Unknown,
//! }
//! ```
//!
//! ## Field attributes
//!
//! * `#[musli(rename = ..)]` allows for renaming a field from its default value.
//!   It can take any value (including complex ones) that can be serialized with
//!   the current encoding, such as:
//!
//!   * `#[musli(rename = 1)]`
//!   * `#[musli(rename = "Hello World")]`
//!   * `#[musli(rename = b"box\0")]`
//!   * `#[musli(rename = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!     `Encode` and `Decode` as appropriate).
//!
//!   If the type of the tag is ambiguous it can be explicitly specified through
//!   the `#[musli(name_type)]` variant or container attributes (see above).
//!
//! * `#[musli(with = <path>)]` specifies the path to a module to use instead of
//!   the fields default [Encode] or [Decode] implementations.
//!
//!   It expects the following functions to be defined, assuming the type of the
//!   field is `Field`.
//!
//!   `encode` for encoding the field, which should match the following
//!   signature:
//!
//! ```rust,ignore
//! fn encode<M, E>(field: &Field, encoder: E) -> Result<E::Ok, E::Error>
//! where
//!     M: Mode,
//!     E: Encoder;
//! ```
//!
//!   `encode` for decoding the field, which should match the following
//!   signature:
//!
//! ```rust,ignore
//! fn decode<'de, M, D>(decoder: D) -> Result<Field, D::Error>
//! where
//!     M: Mode,
//!     D: Decoder<'de>;
//! ```
//!
//! ```
//! # mod types {
//! use musli::{Encode, Decode};
//!
//! pub struct CustomUuid(u128);
//!
//! #[derive(Encode, Decode)]
//! struct Struct {
//!     #[musli(with = self::custom_uuid)]
//!     name: CustomUuid,
//! }
//!
//! mod custom_uuid {
//!     use musli::en::{Encode, Encoder};
//!     use musli::de::{Decode, Decoder};
//!     use musli::mode::Mode;
//!
//!     use super::CustomUuid;
//!
//!     pub fn encode<M, E>(uuid: &CustomUuid, encoder: E) -> Result<E::Ok, E::Error>
//!     where
//!         M: Mode,
//!         E: Encoder,
//!     {
//!         Encode::<M>::encode(&uuid.0, encoder)
//!     }
//!
//!     pub fn decode<'de, M, D>(decoder: D) -> Result<CustomUuid, D::Error>
//!     where
//!         M: Mode,
//!         D: Decoder<'de>
//!     {
//!         Ok(CustomUuid(<u128 as Decode<M>>::decode(decoder)?))
//!     }
//! }
//! # }
//! ```
//!
//! * `#[musli(default)]` constructs the field using [Default::default] in case
//!   it's not available. This is only used when a field is missing during
//!   decoding.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Person {
//!     name: String,
//!     #[musli(default)]
//!     age: Option<u32>,
//! }
//! ```
//!
//! * `#[musli(skip_encoding_if = <path>)]` adds a condition to skip encoding a
//!   field entirely if the condition is true. This is very commonly used to
//!   skip over encoding `Option<T>` fields.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Person {
//!     name: String,
//!     #[musli(skip_encoding_if = Option::is_none)]
//!     age: Option<u32>,
//! }
//! ```
//!
//! # Enum representations
//!
//! Müsli supports the following enum representations, which mimics the ones
//! supported by *serde*:
//!
//! * Externally tagged (*default*).
//! * Internally tagged when `#[musli(tag)]` is specified on the enum.
//! * Adjacently tagged when both `#[musli(tag)]` and `#[musli(content)]` are
//!   specified.
//!
//! ```rust
//! # use musli::{Encode, Decode};
//! # #[derive(Encode, Decode)] struct Params;
//! # #[derive(Encode, Decode)] struct Value;
//! #[derive(Encode, Decode)]
//! enum Message {
//!     Request { id: String, method: String, params: Params },
//!     Response { id: String, result: Value },
//! }
//! ```
//!
//! ## Externally tagged
//!
//! When an enum is externally tagged it is represented by a single field
//! indicating the variant of the enum.
//!
//! ```json
//! {"Request": {"id": "...", "method": "...", "params": {...}}}
//! ```
//!
//! This is the most portable representation and is supported by most formats.
//! It has special support in the [Encoder] and [Decoder] traits through
//! [Encoder::encode_variant] and [Decoder::decode_variant].
//!
//! Conceptually this can be considered as a "pair", where the variant tag can
//! be extracted from the format before the variant is decoded.
//!
//! ## Internally tagged
//!
//! ```rust
//! # use musli::{Encode, Decode};
//! # #[derive(Encode, Decode)] struct Params;
//! # #[derive(Encode, Decode)] struct Value;
//! #[derive(Encode, Decode)]
//! #[musli(tag = "type")]
//! enum Message {
//!     Request { id: String, method: String, params: Params },
//!     Response { id: String, result: Value },
//! }
//! ```
//!
//! In JSON, the `Message::Request` would be represented as:
//!
//! ```json
//! {"type": "Request", "id": "...", "method": "...", "params": {...}}
//! ```
//!
//! This is only supported by formats which are *self descriptive*, which is a
//! requirement for the format to be buffered through [Decoder::decode_buffer].
//!
//! It is necessary to buffer the value, since we need to inspect the fields of
//! a map for the field corresponding to the `tag`, and then use this to
//! determine which decoder implementation to call.
//!
//! [default mode]: crate::mode::DefaultMode
//! [DefaultMode]: crate::mode::DefaultMode
//! [Encode]: crate::Encode
//! [Decode]: crate::Decode
//! [Encoder]: crate::Encoder
//! [Encoder::encode_variant]: crate::Encoder::encode_variant
//! [Decoder]: crate::Decoder
//! [Decoder::decode_variant]: crate::Decoder::decode_variant
//! [Decoder::decode_buffer]: crate::Decoder::decode_buffer

// Parts of this documentation
