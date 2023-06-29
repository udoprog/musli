//! # The [`Encode`] and [`Decode`] derives
//!
//! The [`Encode`] and [`Decode`] derives allows for automatically implementing
//! [`Encode`] and [`Decode`].
//!
//! They come with a number of options for customizing their implementation,
//! detailed below. But first we need to talk about *modes*.
//!
//! <br>
//!
//! #### Attributes
//!
//! * [*Meta attributes*](#meta-attributes) which apply to the attribute itself.
//!   It is used to filter what scope the current attribute applies to, such as
//!   only applying to an `Encode` derive using `#[musli(encode_only, ..)]` or a
//!   specific mode such as `#[musli(mode = Json, ..)]`.
//!
//! * [*Container attributes*](#container-attributes) are attributes which apply
//!   to the `struct` or `enum`.
//!
//! * [*Variant attributes*](#variant-attributes) are attributes which apply to
//!   each individual variant in an `enum`.
//!
//! * [*Field attributes*](#field-attributes) are attributes which apply to each
//!   individual field either in a `struct` or an `enum` variant.
//!
//! <br>
//!
//! #### Modes
//!
//! If you've paid close attention to the [`Encode`] and [`Decode`] traits you
//! might've noticed that they have an extra parameter called `M` which stands
//! for "mode".
//!
//! This allows a single type to have *more than one* implementation of encoding
//! traits, allowing for a high level of flexibility in how a type should be
//! encoded.
//!
//! When it comes to deriving these traits you can scope attributes to apply to
//! either any mode, the [default mode], or a completely custom mode. This is
//! done using the `#[musli(mode = ..)]` meta attribute like this:
//!
//! ```
//! use musli::{Encode, Decode, Mode};
//!
//! enum Json {}
//! impl Mode for Json {}
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
//! use [`DefaultMode`].
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
//! const JSON_ENCODING: Encoding<Json> = Encoding::new().with_mode();
//! const DEFAULT_ENCODING: Encoding = Encoding::new();
//!
//! let named = JSON_ENCODING.to_vec(&Person { name: "Aristotle", age: 62 })?;
//! assert_eq!(named.as_slice(), b"{\"name\":\"Aristotle\",\"age\":62}");
//!
//! let indexed = DEFAULT_ENCODING.to_vec(&Person { name: "Plato", age: 84 })?;
//! assert_eq!(indexed.as_slice(), b"{\"0\":\"Plato\",\"1\":84}");
//!
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! So the `#[musli(mode)]` atttribute is supported in any position. And any of
//! its sibling attributes will be added to the given *alternative* mode, rather
//! the [default mode].
//!
//! <br>
//!
//! ## Meta attributes
//!
//! Certain attributes affect which other attributes apply to a given context.
//! These are called *meta* attributes.
//!
//! Meta attributes are applicable to any context, and can be used on
//! containers, variants, and fields.
//!
//! <br>
//!
//! #### `#[musli(mode = <path>)]`
//!
//! The attributes only apply to the given `mode`.
//!
//! The `Person` struct below uses string field names when the `Json` mode is
//! enabled, otherwise it uses default numerical field names.
//!
//! ```
//! use musli::{Encode, Decode, Mode};
//!
//! enum Json {}
//! impl Mode for Json {}
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
//! <br>
//!
//! #### `#[musli(encode_only)]`
//!
//! The attributes only apply when implementing the `Encode` trait.
//!
//! An example where this is useful is if you want to apply `#[musli(packed)]`
//! in a different mode, but only for encoding, since decoding packed types is
//! not supported for enums.
//!
//! ```
//! use musli::mode::DefaultMode;
//! use musli::{Decode, Encode, Mode};
//!
//! enum Packed {}
//! impl Mode for Packed {}
//!
//! #[derive(Encode, Decode)]
//! #[musli(mode = Packed, encode_only, packed)]
//! enum Name<'a> {
//!     Full(&'a str),
//!     Given(&'a str),
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(decode_only)]`
//!
//! The attributes only apply when implementing the `Decode` trait.
//!
//! ```
//! use musli::{Decode, Encode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_name = "name")]
//! struct Name<'a> {
//!     sur_name: &'a str,
//!     #[musli(decode_only, rename = "last")]
//!     last_name: &'a str,
//! }
//! ```
//!
//! <br>
//!
//! ## Container attributes
//!
//! Container attributes apply to the container, such as directly on the
//! `struct` or `enum`. Like the uses of `#[musli(packed)]` and
//! `#[musli(default_variant_name = "name")]` here:
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
//! <br>
//!
//! #### `#[musli(default_field_name = "..")]`
//!
//! This determines how the default tag for a field is determined. It can take
//! either `"name"` or `"index"`.
//!
//! * `#[musli(default_field_name = "index")]` will use the index of the field.
//!   This is the default.
//! * `#[musli(default_field_name = "name")]` will use the name of the field.
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
//! <br>
//!
//! #### `#[musli(default_variant_name = "..")]`
//!
//! This determines how the default tag for a variant is determined. It can take
//! either `"name"` or `"index"`.
//!
//! * `#[musli(default_variant_name = "index")]` will use the index of the
//!   variant. This is the default.
//! * `#[musli(default_variant_name = "name")]` will use the name of the
//!   variant.
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
//! <br>
//!
//! #### `#[musli(transparent)]`
//!
//! This can only be used on types which have a single field. It will cause that
//! field to define how that variant is encoded or decoded transparently without
//! being treated as a field.
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
//! let data = musli_wire::to_vec(&Struct(42))?;
//! assert_eq!(data.as_slice(), vec![Tag::new(Kind::Continuation, 42).byte()]);
//! # Ok(()) }
//! ```
//!
//! <br>
//!
//! #### `#[musli(packed)]`
//!
//! This attribute will disable all *tagging* and the structure will simply be
//! encoded with one field following another in the order in which they are
//! defined.
//!
//! A caveat of *packed* structures is that they cannot be safely versioned and
//! the two systems communicating through them need to be using strictly
//! synchronized representations.
//!
//! This attribute is useful for performing simple decoding over "raw" bytes
//! when combined with an encoder which does minimal prefixing and packs fields.
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
//! let data = musli_storage::to_vec(&Struct {
//!     field1: 1,
//!     field2: 2,
//!     field3: 3,
//! })?;
//!
//! assert_eq!(data.as_slice(), vec![1, 2, 3]);
//! # Ok(()) }
//! ```
//!
//! <br>
//!
//! #### `#[musli(name_type = ..)]`
//!
//! This indicates which type any contained `#[musli(rename = ..)]` attributes
//! should have. Tags can usually be inferred, but specifying this field ensures
//! that all tags have a single well-defined type.
//!
//! ```
//! use core::fmt;
//!
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//! #[musli(transparent)]
//! struct CustomTag<'a>(&'a [u8]);
//!
//! impl fmt::Display for CustomTag<'_> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         fmt::Debug::fmt(self.0, f)
//!     }
//! }
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
//! <br>
//!
//! #### `#[musli(bound = {..})]` and `#[musli(decode_bound = {..})]`
//!
//! These attributes can be used to apply bounds to an [`Encode`] or [`Decode`]
//! implementation.
//!
//! These are necessary to use when a generic container is used to ensure that
//! the given parameter implements the necessary bounds.
//!
//! `#[musli(bound = {..})]` applies to all implementations while
//! `#[musli(decode_bound = {..})]` only applies to the [`Decode`] implementation.
//! The latter allows for using the decode lifetime parameter (which defaults to
//! `'de`).
//!
//! ```
//! use musli::{Decode, Encode};
//!
//! #[derive(Clone, Debug, PartialEq, Encode, Decode)]
//! #[musli(bound = {T: Encode<M>}, decode_bound = {T: Decode<'de, M>})]
//! pub struct GenericWithBound<T> {
//!     value: T,
//! }
//! ```
//!
//! <br>
//!
//! ## Enum attributes
//!
//! <br>
//!
//! #### `#[musli(tag = ..)]`
//!
//! This attribute causes the enum to be internally tagged, with the given tag.
//! See [enum representations](#enum-representations) for details on this
//! representation.
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
//! <br>
//!
//! ## Variant attributes
//!
//! *Variant attributes* are attributes which apply to each individual variant
//! in an `enum`. Like the use of `#[musli(rename = ..)]` here:
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
//! <br>
//!
//! #### `#[musli(rename = ..)]`
//!
//! This allows for renaming a variant from its default value. It can take any
//! value (including complex ones) that can be serialized with the current
//! encoding, such as:
//!
//! * `#[musli(rename = 1)]`
//! * `#[musli(rename = "Hello World")]`
//! * `#[musli(rename = b"box\0")]`
//! * `#[musli(rename = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!   [`Encode`] and [`Decode`] as appropriate).
//!
//! If the type of the tag is ambiguous it can be explicitly specified through
//! the `#[musli(name_type)]` attribute.
//!
//! <br>
//!
//! #### `#[musli(name_type = ..)]`
//!
//! This indicates which type any contained `#[musli(tag = ..)]` attributes
//! should have. Tags can usually be inferred, but specifying this field ensures
//! that all tags have a well-defined type.
//!
//! This attribute takes priority over the one with the same name on the
//! container.
//!
//! ```
//! use core::fmt;
//!
//! use musli::{Encode, Decode};
//!
//! #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//! #[musli(transparent)]
//! struct CustomTag<'a>(&'a [u8]);
//!
//! impl fmt::Display for CustomTag<'_> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         fmt::Debug::fmt(self.0, f)
//!     }
//! }
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
//! <br>
//!
//! #### `#[musli(default_field_name = "..")]`
//!
//! This determines how the default tag for a field in the current variant is
//! determined. This overrides the tagging convention specified on the
//! *container* and can take either `"name"` or `"index"`.
//!
//! * `#[musli(default_field_name = "index")]` will use the index of the field.
//!   This is the default.
//! * `#[musli(default_field_name = "name")]` will use the name of the field.
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
//! <br>
//!
//! #### `#[musli(transparent)]`
//!
//! This can only be used on variants which have a single field. It will cause
//! that field to define how that variant is encoded or decoded transparently
//! without being treated as a field.
//!
//! <br>
//!
//! #### `#[musli(default)]`
//!
//! This defines the variant that will be used in case no other variant matches.
//! Only one such variant can be defined.
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
//! <br>
//!
//! ## Field attributes
//!
//! *Field attributes* are attributes which apply to each individual field
//! either in a `struct` or an `enum` variant. Like the uses of
//! `#[musli(rename)]` here:
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
//! <br>
//!
//! #### `#[musli(rename = ..)]`
//!
//! This allows for renaming a field from its default value. It can take any
//! value (including complex ones) that can be serialized with the current
//! encoding, such as:
//!
//! * `#[musli(rename = 1)]`
//! * `#[musli(rename = "Hello World")]`
//! * `#[musli(rename = b"box\0")]`
//! * `#[musli(rename = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!   [`Encode`] and [`Decode`] as appropriate).
//!
//! If the type of the tag is ambiguous it can be explicitly specified through
//! the `#[musli(name_type)]` variant or container attributes (see above).
//!
//! <br>
//!
//! #### `#[musli(with = <path>)]`
//!
//! This specifies the path to a module to use instead of the fields default
//! [`Encode`] or [`Decode`] implementations.
//!
//! It expects the following functions to be defined, assuming the type of the
//! field is `Field`.
//!
//! `encode` for encoding the field, which should match the following signature:
//!
//! ```rust,ignore
//! fn encode<M, E>(field: &Field, encoder: E) -> Result<E::Ok, E::Error>
//! where
//!     M: Mode,
//!     E: Encoder;
//! ```
//!
//! `encode` for decoding the field, which should match the following signature:
//!
//! ```rust,ignore
//! fn decode<'de, M, D>(decoder: D) -> Result<Field, D::Error>
//! where
//!     M: Mode,
//!     D: Decoder<'de>;
//! ```
//!
//! Finally this can receive generic arguments like `#[musli(with =
//! crate::path::<_>)]`, in case the receiving decode and encode functions
//! receive *extra* generic arguments (beyond `M` and `D`), such as:
//!
//! ```rust,ignore
//! fn decode<'de, M, D, T>(decoder: D) -> Result<Set<T>, D::Error>
//! where
//!     M: Mode,
//!     D: Decoder<'de>,
//!     T: Decode<'de>;
//! ```
//!
//! Full example:
//!
//! ```
//! # mod types {
//! use std::collections::HashSet;
//! use musli::{Encode, Decode};
//!
//! pub struct CustomUuid(u128);
//!
//! #[derive(Encode, Decode)]
//! struct Struct {
//!     #[musli(with = self::custom_uuid)]
//!     id: CustomUuid,
//!     #[musli(with = self::custom_set::<_>)]
//!     numbers: HashSet<u32>,
//! }
//!
//! mod custom_uuid {
//!     use musli::Context;
//!     use musli::en::{Encode, Encoder};
//!     use musli::de::{Decode, Decoder};
//!     use musli::mode::Mode;
//!
//!     use super::CustomUuid;
//!
//!     pub fn encode<M, C, E>(uuid: &CustomUuid, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
//!     where
//!         M: Mode,
//!         C: Context<Input = E::Error>,
//!         E: Encoder,
//!     {
//!         Encode::<M>::encode(&uuid.0, cx, encoder)
//!     }
//!
//!     pub fn decode<'de, M, C, D>(cx: &mut C, decoder: D) -> Result<CustomUuid, C::Error>
//!     where
//!         M: Mode,
//!         C: Context<Input = D::Error>,
//!         D: Decoder<'de>
//!     {
//!         Ok(CustomUuid(<u128 as Decode<M>>::decode(cx, decoder)?))
//!     }
//! }
//!
//! mod custom_set {
//!     use std::collections::HashSet;
//!     use std::hash::Hash;
//!
//!     use musli::Context;
//!     use musli::en::{Encode, Encoder};
//!     use musli::de::{Decode, Decoder};
//!     use musli::mode::Mode;
//!
//!     pub fn encode<M, C, E, T>(set: &HashSet<T>, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
//!     where
//!         M: Mode,
//!         C: Context<Input = E::Error>,
//!         E: Encoder,
//!         T: Encode<M> + Eq + Hash,
//!     {
//!         HashSet::<T>::encode(set, cx, encoder)
//!     }
//!
//!     pub fn decode<'de, M, C, D, T>(cx: &mut C, decoder: D) -> Result<HashSet<T>, C::Error>
//!     where
//!         M: Mode,
//!         C: Context<Input = D::Error>,
//!         D: Decoder<'de>,
//!         T: Decode<'de> + Eq + Hash,
//!     {
//!         HashSet::<T>::decode(cx, decoder)
//!     }
//! }
//! # }
//! ```
//!
//! <br>
//!
//! #### `#[musli(default)]`
//!
//! This constructs the field using [Default::default] in case it's not
//! available. This is only used when a field is missing during decoding.
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
//! <br>
//!
//! #### `#[musli(skip_encoding_if = <path>)]`
//!
//! This adds a condition to skip encoding a field entirely if the condition is
//! true. This is very commonly used to skip over encoding `Option<T>` fields.
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
//! #### `#[musli(trace)]`
//!
//! This causes the field to use the [`TraceDecode`] / [`TraceEncode`] when
//! encoding the field. This is left optional for types where enabling tracing
//! for the field requires extra traits to be implemented, such as `HashMap<K,
//! V>` where we'd need `K` to implement `fmt::Display`.
//!
//! Without using the `trace` attribute below, the keys in the `values` field
//! would not be instrumented, so with a decoding error you'd see this:
//!
//! ```text
//! .values: not numeric (at bytes 15-16)
//! ```
//!
//! Instead of this (where `#[musli(trace)]` is enabled):
//!
//! ```text
//! .values[Hello]: not numeric (at bytes 15-16)
//! ```
//!
//! ```
//! use std::collections::HashMap;
//!
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Collection {
//!     #[musli(trace)]
//!     values: HashMap<String, u32>,
//! }
//! ```
//!
//! <br>
//!
//! # Enum representations
//!
//! Müsli supports the following enum representations, which mimics the ones
//! supported by *serde*:
//!
//! * Externally tagged (*default*).
//! * Internally tagged when `#[musli(tag = ..)]` is specified on the enum.
//! * Adjacently tagged when both `#[musli(tag = ..)]` and `#[musli(content)]`
//!   are specified.
//!
//! <br>
//!
//! ## Externally tagged
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
//! When an enum is externally tagged it is represented by a single field
//! indicating the variant of the enum.
//!
//! ```json
//! {"Request": {"id": "...", "method": "...", "params": {...}}}
//! ```
//!
//! This is the most portable representation and is supported by most formats.
//! It has special support in the [`Encoder`] and [`Decoder`] traits through
//! [`Encoder::encode_variant`] and [`Decoder::decode_variant`].
//!
//! Conceptually this can be considered as a "pair", where the variant tag can
//! be extracted from the format before the variant is decoded.
//!
//! <br>
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
//! requirement for the format to be buffered through [`Decoder::decode_buffer`].
//!
//! It is necessary to buffer the value, since we need to inspect the fields of
//! a map for the field corresponding to the `tag`, and then use this to
//! determine which decoder implementation to call.
//!
//! [`Decode`]: crate::Decode
//! [`Decoder::decode_buffer`]: crate::Decoder::decode_buffer
//! [`Decoder::decode_variant`]: crate::Decoder::decode_variant
//! [`Decoder`]: crate::Decoder
//! [`DefaultMode`]: crate::mode::DefaultMode
//! [`Encode`]: crate::Encode
//! [`Encoder::encode_variant`]: crate::Encoder::encode_variant
//! [`Encoder`]: crate::Encoder
//! [`TraceDecode`]: crate::de::TraceDecode
//! [`TraceEncode`]: crate::en::TraceEncode
//! [default mode]: crate::mode::DefaultMode

// Parts of this documentation
