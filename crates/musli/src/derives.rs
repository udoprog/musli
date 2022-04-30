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
//! #[musli(default_field_tag = "index")]
//! #[musli(mode = Json, default_field_tag = "name")]
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
//! # #[musli(mode = Json, default_field_tag = "name")]
//! # struct Person<'a> { name: &'a str, age: u32 }
//! use musli_json::JsonEncoding;
//!
//! const JSON_ENCODING: JsonEncoding<Json> = JsonEncoding::new();
//! const DEFAULT_ENCODING: JsonEncoding = JsonEncoding::new();
//!
//! let named = JSON_ENCODING.to_string(&Person { name: "Aristotle", age: 62 })?;
//! assert_eq!(named, "{\"name\":\"Aristotle\",\"age\":62}");
//!
//! let indexed = DEFAULT_ENCODING.to_string(&Person { name: "Plato", age: 84 })?;
//! assert_eq!(indexed, "{\"0\":\"Plato\",\"1\":84}");
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
//!   `#[musli(default_variant_tag = "name")]` here:
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
//! #[musli(default_variant_tag = "name")]
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
//! #[musli(default_variant_tag = "name")]
//! enum Enum {
//!     #[musli(tag = "Other")]
//!     Something {
//!         /* variant body */
//!     }
//! }
//! ```
//!
//! * *Field attributes* are attributes which apply to each individual field
//!   either in a `struct` or an `enum` variant. Like the uses of
//!   `#[musli(tag)]` here:
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_tag = "name")]
//! struct Struct {
//!     #[musli(tag = "other")]
//!     something: String,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_tag = "name")]
//! enum Enum {
//!     Variant {
//!         #[musli(tag = "other")]
//!         something: String,
//!     }
//! }
//! ```
//!
//! ## Container attributes
//!
//! * `#[musli(tag_type = ..)]` indicates which type any contained `#[musli(tag
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
//! #[musli(tag_type = CustomTag)]
//! struct Struct {
//!     #[musli(tag = CustomTag(b"name in bytes"))]
//!     name: String,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(tag_type = CustomTag)]
//! enum EnumWithCustomTag {
//!     #[musli(tag = CustomTag(b"variant one"))]
//!     Variant1 {
//!         /* .. */
//!     },
//! }
//! ```
//!
//! * `#[musli(default_field_tag = "..")]` determines how the default tag for a
//!   field is determined. It can take either `"name"` or `"index"`.
//!
//!   `#[musli(default_field_tag = "index")]` will use the index of the field.
//!   This is the default.
//!
//!   `#[musli(default_field_tag = "name")]` will use the name of the field.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_tag = "name")]
//! struct Struct {
//!     field1: u32,
//!     field2: u32,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_tag = "name")]
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
//! * `#[musli(default_variant_tag = "..")]` determines how the default tag for
//!   a variant is determined. It can take either `"name"` or `"index"`.
//!
//!   `#[musli(default_variant_tag = "index")]` will use the index of the
//!   variant. This is the default.
//!
//!   `#[musli(default_variant_tag = "name")]` will use the name of the variant.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_variant_tag = "name")]
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
//! let data = musli_wire::to_vec(&Struct(42))?;
//! assert_eq!(data, vec![Tag::new(Kind::Continuation, 42).byte()]);
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
//! let data = musli_storage::to_vec(&Struct {
//!     field1: 1,
//!     field2: 2,
//!     field3: 3,
//! })?;
//!
//! assert_eq!(data, vec![1, 2, 3]);
//! # Ok(()) }
//! ```
//!
//! ## Variant attributes
//!
//! * `#[musli(tag = ..)]` allows for renaming a variant from its default value.
//!   It can take any value (including complex ones) that can be serialized with
//!   the current encoding, such as:
//!
//!   * `#[musli(tag = 1)]`
//!   * `#[musli(tag = "Hello World")]`
//!   * `#[musli(tag = b"box\0")]`
//!   * `#[musli(tag = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!     `Encode` and `Decode` as appropriate).
//!
//!   If the type of the tag is ambiguous it can be explicitly specified through
//!   the `#[musli(tag_type)]` container attribute (see above).
//!
//! * `#[musli(default_field_tag = "..")]` determines how the default tag for a
//!   field in the current variant is determined. This overrides the tagging
//!   convention specified on the *container* and can take either `"name"` or
//!   `"index"`.
//!
//!   `#[musli(default_field_tag = "index")]` will use the index of the field.
//!   This is the default.
//!
//!   `#[musli(default_field_tag = "name")]` will use the name of the field.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field_tag = "index")]
//! enum Enum {
//!     #[musli(default_field_tag = "name")]
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
//! * `#[musli(tag_type = ..)]` indicates which type any contained `#[musli(tag
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
//! #[musli(tag_type = usize)]
//! enum Enum {
//!     #[musli(tag_type = CustomTag)]
//!     Variant {
//!         #[musli(tag = CustomTag(b"name in bytes"))]
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
//!     #[musli(tag = "cat")]
//!     Cat,
//!     #[musli(tag = "dog")]
//!     Dog,
//!     #[musli(default)]
//!     Unknown,
//! }
//! ```
//!
//! ## Field attributes
//!
//! * `#[musli(tag = ..)]` allows for renaming a field from its default value.
//!   It can take any value (including complex ones) that can be serialized with
//!   the current encoding, such as:
//!
//!   * `#[musli(tag = 1)]`
//!   * `#[musli(tag = "Hello World")]`
//!   * `#[musli(tag = b"box\0")]`
//!   * `#[musli(tag = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!     `Encode` and `Decode` as appropriate).
//!
//!   If the type of the tag is ambiguous it can be explicitly specified through
//!   the `#[musli(tag_type)]` variant or container attributes (see above).
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
//! [default mode]: crate::mode::DefaultMode
//! [DefaultMode]: crate::mode::DefaultMode
//! [Encode]: crate::Encode
//! [Decode]: crate::Decode
