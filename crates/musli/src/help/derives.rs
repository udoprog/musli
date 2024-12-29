//! Deriving [`Encode`] and [`Decode`].
//!
//! The [`Encode`] and [`Decode`] derives allows for automatically implementing
//! [`Encode`] and [`Decode`].
//!
//! They come with a number of options for customizing their implementation,
//! detailed below. But first we need to talk about *modes*.
//!
//! <br>
//!
//! #### Modes
//!
//! If you've paid close attention to the [`Encode`] and [`Decode`] traits you
//! might notive that they have an extra parameter called `M`. This stands for
//! "mode".
//!
//! This parameter allows us to have different implementations of these traits
//! for the same type.
//!
//! By default we implements two modes, which each have subtly different default
//! behaviors:
//! * [`Binary`] - which uses indexed fields, the equivalent of
//!   `#[musli(name_type = usize)]`.
//! * [`Text`] - which uses literally text fields by their name, the equivalent
//!   of `#[musli(name_type = str)]`.
//!
//! When it comes to deriving these traits you can scope attributes to apply to
//! any mode including custom local ones. This is done using the `#[musli(mode =
//! ..)]` meta attribute like this:
//!
//! ```
//! use musli::{Encode, Decode};
//! use musli::mode::Binary;
//! use musli::json::Encoding;
//!
//! #[derive(Encode, Decode)]
//! struct Person<'a> {
//!     #[musli(mode = Text, name = "name")]
//!     not_name: &'a str,
//!     age: u32,
//! }
//!
//! const TEXT: Encoding = Encoding::new();
//! const BINARY: Encoding<Binary> = Encoding::new().with_mode();
//!
//! let named = TEXT.to_vec(&Person { not_name: "Aristotle", age: 61 })?;
//! assert_eq!(named.as_slice(), br#"{"name":"Aristotle","age":61}"#);
//!
//! let indexed = BINARY.to_vec(&Person { not_name: "Plato", age: 84 })?;
//! assert_eq!(indexed.as_slice(), br#"{"0":"Plato","1":84}"#);
//! # Ok::<_, musli::json::Error>(())
//! ```
//!
//! So the `#[musli(mode)]` atttribute is supported in any position. And any of
//! its sibling attributes will be added to the given *alternative* mode, rather
//! the [default mode].
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
//! The `Person` struct below uses string field names by default when the `Text`
//! mode is enabled, but we can change this behavior only for that particular
//! mode like this:
//!
//! ```
//! use musli::{Encode, Decode};
//! use musli::mode::Text;
//!
//! #[derive(Encode, Decode)]
//! #[musli(mode = Text, name_type = usize)]
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
//! use musli::mode::Binary;
//! use musli::{Decode, Encode};
//!
//! enum Packed {}
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
//! #[musli(name_all = "name")]
//! struct Name<'a> {
//!     sur_name: &'a str,
//!     #[musli(decode_only, name = "last")]
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
//! `#[musli(name_all = "name")]` here:
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
//! #[musli(name_all = "name")]
//! enum Enum {
//!     /* the body of the enum */
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(name_all = "..")]`
//!
//! Allos for renaming every field in the container. It can take any of the
//! following values:
//!
//! * `index` (default) - the index of the field will be used.
//! * `name` - the literal name of the field will be used.
//! * `PascalCase` - the field will be converted to pascal case.
//! * `camelCase` - the field will be converted to camel case.
//! * `snake_case` - the field will be converted to snake case.
//! * `SCREAMING_SNAKE_CASE` - the field will be converted to screaming snake case.
//! * `kebab-case` - the field will be converted to kebab case.
//! * `SCREAMING-KEBAB-CASE` - the field will be converted to screaming kebab case.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "PascalCase")]
//! struct PascalCaseStruct {
//!     field_name: u32,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name")]
//! struct NamedStruct {
//!     field1: u32,
//!     field2: u32,
//! }
//! ```
//!
//! If applied to an enum, it will instead rename all variants:
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "PascalCase")]
//! enum PascalCaseEnum {
//!     VariantName {
//!         field_name: u32,
//!     }
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name")]
//! enum NamedEnum {
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
//!
//! #[derive(Encode)]
//! #[musli(transparent)]
//! struct Struct(u32);
//!
//! let data = musli::wire::to_vec(&Struct(42))?;
//! let actual: u32 = musli::wire::from_slice(&data)?;
//! assert_eq!(actual, 42u32);
//! # Ok::<_, musli::wire::Error>(())
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
//! let data = musli::storage::to_vec(&Struct {
//!     field1: 1,
//!     field2: 2,
//!     field3: 3,
//! })?;
//!
//! assert_eq!(data.as_slice(), [1, 2, 3]);
//! # Ok::<_, musli::storage::Error>(())
//! ```
//!
//! <br>
//!
//! #### `#[musli(bitwise)]`
//!
//! This attribute has the same requirements as `#[musli(packed)]` and also
//! requires every field to implement `Encode` or `Decode`. It is also only
//! supported on structs.
//!
//! If a struct is tagged with `#[musli(bitwise)]`, and the bitwise pattern of a
//! given serialization is *identical* to the bitwise memory pattern of the
//! struct, then serialization and deserialization can be made more efficient.
//!
//! Note that since [`#[repr(Rust)]`][repr-rust] is not strictly defined, it
//! might be necessary to mark the struct with `#[repr(C)]` to benefit from this
//! optimization.
//!
//! If the `#[musli(bitwise)]` optimization doesn't work, it will either have no
//! effect or cause a compilation error.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(bitwise)]
//! struct Struct {
//!     a: u32,
//!     b: u32,
//! }
//!
//! const _: () = assert!(musli::is_bitwise_encode::<Struct>());
//! const _: () = assert!(musli::is_bitwise_decode::<Struct>());
//! ```
//!
//! Note that some combinations of fields currently only support encoding in one
//! direction. This is the case for `NonZero` types, since they cannot inhabit
//! all possible bit patterns.
//!
//! ```
//! use core::num::NonZero;
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(bitwise)]
//! struct Struct {
//!     a: NonZero<u32>,
//!     b: u32,
//! }
//!
//! const _: () = assert!(musli::is_bitwise_encode::<Struct>());
//! const _: () = assert!(!musli::is_bitwise_decode::<Struct>());
//! ```
//!
//! [repr-rust]: <https://doc.rust-lang.org/nomicon/repr-rust.html>
//!
//! <br>
//!
//! #### `#[musli(name_type = ..)]`
//!
//! This indicates which type any contained `#[musli(name = ..)]` attributes
//! should have. Tags can usually be inferred, but specifying this field ensures
//! that all tags have a single well-defined type.
//!
//! The following values are treated specially:
//! * `str` applies `#[musli(name_all = "name")]` by default.
//! * `[u8]` applies `#[musli(name_all = "name")]` by default.
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
//!     #[musli(name = CustomTag(b"name in bytes"))]
//!     name: String,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_type = CustomTag)]
//! enum EnumWithCustomTag {
//!     #[musli(name = CustomTag(b"variant one"))]
//!     Variant1 {
//!         /* .. */
//!     },
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(name_method = ..)]`
//!
//! This allows for explicitly setting which method should be used to decode
//! names. Available options are:
//!
//! * `"value"` (default) - the name is decoded as a value.
//! * `"unsized"` - the name is decoded as an unsized value, this is the default
//!   if for example `#[musli(name_type = str)]` is used.
//! * `"unsized_bytes"` - the name is decoded as a unsized bytes, this is the
//!   default if for example `#[musli(name_type = [u8])]` is used.
//!
//! This can be overrided for values which are unsized, but cannot be determined
//! through heuristics. Such a type must also implement [`Decode`] (for
//! `"value"`), `DecodeUnsized`, or `DecodeUnsizedBytes` as appropriate.
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
//! `#[musli(decode_bound = {..})]` only applies to the [`Decode`]
//! implementation. The latter allows for using the decode lifetime parameter
//! (which defaults to `'de`).
//!
//! ```
//! use musli::{Decode, Encode};
//! use musli::mode::{Binary, Text};
//!
//! #[derive(Clone, Debug, PartialEq, Encode, Decode)]
//! #[musli(mode = Binary, bound = {T: Encode<Binary>}, decode_bound = {T: Decode<'de, Binary>})]
//! #[musli(mode = Text, bound = {T: Encode<Text>}, decode_bound = {T: Decode<'de, Text>})]
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
//! ```
//! # use musli::{Encode, Decode};
//! # #[derive(Encode, Decode)] struct Params;
//! # #[derive(Encode, Decode)] struct Value;
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name", tag = "type")]
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
//! in an `enum`. Like the use of `#[musli(name = ..)]` here:
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//!#[musli(name_all = "name")]
//! enum Enum {
//!     Variant {
//!         /* variant body */
//!     },
//!     #[musli(name = "Other")]
//!     Something {
//!         /* variant body */
//!     },
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(name = ..)]`
//!
//! This allows for renaming a variant from its default value. It can take any
//! value (including complex ones) that can be serialized with the current
//! encoding, such as:
//!
//! * `#[musli(name = 1)]`
//! * `#[musli(name = "Hello World")]`
//! * `#[musli(name = b"box\0")]`
//! * `#[musli(name = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!   [`Encode`] and [`Decode`] as appropriate).
//!
//! If the type of the tag is ambiguous it can be explicitly specified through
//! the `#[musli(name_type)]` attribute.
//!
//! <br>
//!
//! #### `#[musli(pattern = ..)]`
//!
//! A pattern to match for decoding a variant.
//!
//! This allows for more flexibility when decoding variants.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! enum Enum {
//!     Variant1,
//!     Variant2,
//!     #[musli(mode = Binary, pattern = 2..=4)]
//!     Deprecated,
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(name_all = "..")]`
//!
//! Allos for renaming every field in the variant. It can take any of the
//! following values:
//!
//! * `index` (default) - the index of the field will be used.
//! * `name` - the literal name of the field will be used.
//! * `PascalCase` - the field will be converted to pascal case.
//! * `camelCase` - the field will be converted to camel case.
//! * `snake_case` - the field will be converted to snake case.
//! * `SCREAMING_SNAKE_CASE` - the field will be converted to screaming snake case.
//! * `kebab-case` - the field will be converted to kebab case.
//! * `SCREAMING-KEBAB-CASE` - the field will be converted to screaming kebab case.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! enum PascalCaseEnum {
//!     #[musli(name_all = "PascalCase")]
//!     Variant {
//!         field_name: u32,
//!     }
//! }
//!
//! #[derive(Encode, Decode)]
//! enum NamedEnum {
//!     #[musli(name_all = "name")]
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
//! #### `#[musli(name_type = ..)]`
//!
//! This indicates which type any contained `#[musli(tag = ..)]` attributes
//! should have. Tags can usually be inferred, but specifying this field ensures
//! that all tags have a well-defined type.
//!
//! The following values are treated specially:
//! * `str` applies `#[musli(name_all = "name")]` by default.
//! * `[u8]` applies `#[musli(name_all = "name")]` by default.
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
//!     #[musli(name = 0usize, name_type = CustomTag)]
//!     Variant {
//!         #[musli(name = CustomTag(b"field1"))]
//!         field1: u32,
//!         #[musli(name = CustomTag(b"field2"))]
//!         field2: u32,
//!     },
//!     #[musli(name = 1usize, name_all = "name")]
//!     Variant2 {
//!         #[musli(name = "field1")]
//!         field1: u32,
//!         #[musli(name = "field2")]
//!         field2: u32,
//!     },
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(name_method = ..)]`
//!
//! This allows for explicitly setting which method should be used to decode
//! field names. Available options are:
//!
//! * `"value"` (default) - the name is decoded as a value.
//! * `"unsized"` - the name is decoded as an unsized value, this is the default
//!   if for example `#[musli(name_type = str)]` is used.
//! * `"unsized_bytes"` - the name is decoded as a unsized bytes, this is the
//!   default if for example `#[musli(name_type = [u8])]` is used.
//!
//! This can be overrided for values which are unsized, but cannot be determined
//! through heuristics. Such a type must also implement [`Decode`] (for
//! `"value"`), `DecodeUnsized`, or `DecodeUnsizedBytes` as appropriate.
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
//! #[musli(name_all = "kebab-case")]
//! enum Animal {
//!     Cat,
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
//! either in a `struct` or an `enum` variant. Like the uses of `#[musli(all)]`
//! here:
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name")]
//! struct Struct {
//!     #[musli(name = "other")]
//!     something: String,
//!     #[musli(skip, default = default_field)]
//!     skipped_field: u32,
//! }
//!
//! fn default_field() -> u32 {
//!     42
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name")]
//! enum Enum {
//!     #[musli(name_all = "name")]
//!     Variant {
//!         #[musli(name = "other")]
//!         something: String,
//!     }
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(skip)]`
//!
//! This attribute means that the entire field is skipped. If a field is decoded
//! it uses [`Default::default`] to construct the value. Other defaults can be
//! specified with [`#[musli(default = <path>)]`][#muslidefault--path].
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Person {
//!     name: String,
//!     #[musli(skip)]
//!     age: Option<u32>,
//!     #[musli(skip, default = default_country)]
//!     country: Option<String>,
//! }
//!
//! fn default_country() -> Option<String> {
//!     Some(String::from("Earth"))
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(default [= <path>])]`
//!
//! When a field is absent or disabled with `#[musli(skip)]`, this attribute
//! specifies that a default value should be used instead.
//!
//! If `#[musli(default)]` is specified, the default value is constructed using
//! [`Default::default`].
//!
//! If `#[musli(default = <path>)]` is specified, the default value is
//! constructed by calling the function at `<path>`.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Person {
//!     name: String,
//!     #[musli(default)]
//!     age: Option<u32>,
//!     #[musli(default = default_height)]
//!     height: Option<u32>,
//!     #[musli(skip, default = default_meaning)]
//!     meaning: u32,
//! }
//!
//! fn default_height() -> Option<u32> {
//!     Some(180)
//! }
//!
//! fn default_meaning() -> u32 {
//!     42
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(name = ..)]`
//!
//! This allows for renaming a field from its default value. It can take any
//! value (including complex ones) that can be serialized with the current
//! encoding, such as:
//!
//! * `#[musli(name = 1)]`
//! * `#[musli(name = "Hello World")]`
//! * `#[musli(name = b"box\0")]`
//! * `#[musli(name = SomeStruct { field: 42 })]` (if `SomeStruct` implements
//!   [`Encode`] and [`Decode`] as appropriate).
//!
//! If the type of the tag is ambiguous it can be explicitly specified through
//! the `#[musli(name_type)]` variant or container attributes.
//!
//! <br>
//!
//! #### `#[musli(pattern = ..)]`
//!
//! A pattern to match for decoding the given field.
//!
//! This allows for more flexibility when decoding fields.
//!
//! ```
//! use musli::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Struct {
//!     field1: u32,
//!     field2: u32,
//!     #[musli(mode = Binary, pattern = 2..=4)]
//!     other: u32,
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(packed)]`
//!
//! This specifies that encoding and decoding should happen through the
//! [`EncodePacked`] and [`DecodePacked`] traits, instead of the default
//! [`Encode`] and [`Decode`].
//!
//! These traits contained implementations which are biased towards encoding the
//! field as a compact, non-future compatible pack. In essense, the fields are
//! encoded "one after another" without any metadata separating them. So for
//! packed fields, the order, types and number of the fields are important.
//!
//! ```
//! use std::collections::VecDeque;
//!
//! use musli::{Decode, Encode};
//!
//! #[derive(Decode, Encode)]
//! struct Container {
//!     #[musli(packed)]
//!     tuple: (u32, u64),
//!     #[musli(packed)]
//!     array: [u32; 4],
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(bytes)]`
//!
//! This specifies that encoding and decoding should happen through the
//! [`EncodeBytes`] and [`DecodeBytes`] traits, instead of the default
//! [`Encode`] and [`Decode`].
//!
//! These traits contained implementations which are biased towards encoding the
//! field as an array of bytes.
//!
//! ```
//! use std::collections::VecDeque;
//!
//! use musli::{Decode, Encode};
//!
//! #[derive(Decode, Encode)]
//! struct Container<'de> {
//!     #[musli(bytes)]
//!     vec: Vec<u8>,
//!     #[musli(bytes)]
//!     vec_deque: VecDeque<u8>,
//!     #[musli(bytes)]
//!     bytes: &'de [u8],
//! }
//! ```
//!
//! <br>
//!
//! #### `#[musli(with = <path>)]`
//!
//! This specifies the path to a module to use instead of the fields default
//! [`Encode`] or [`Decode`] implementations.
//!
//! It expects `encode` and `decode` and decodee function to be defined in the path being specified, like this:
//!
//! ```
//! # mod example {
//! use musli::{Decode, Encode};
//!
//! #[derive(Decode, Encode)]
//! struct Container {
//!     #[musli(with = self::module)]
//!     field: Field,
//! }
//!
//! struct Field {
//!     /* internal */
//! }
//!
//! mod module {
//!     use musli::{Decoder, Encoder};
//!
//!     use super::Field;
//!
//!     pub fn encode<E>(field: &Field, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
//!     where
//!         E: Encoder,
//! # { todo!() }
//!
//!     pub fn decode<'de, D>(cx: &D::Cx, decoder: D) -> Result<Field, D::Error>
//!     where
//!         D: Decoder<'de>,
//! # { todo!() }
//! }
//! # }
//! ```
//!
//! This can also be generic such as:
//!
//! ```
//! # mod example {
//! use musli::{Decode, Encode};
//!
//! #[derive(Decode, Encode)]
//! struct Container {
//!     #[musli(with = self::module)]
//!     field: Field<u32>,
//! }
//!
//! struct Field<T> {
//!     /* internal */
//! #   value: T,
//! }
//!
//! mod module {
//!     use musli::{Decoder, Encoder};
//!
//!     use super::Field;
//!
//!     pub fn encode<E, T>(field: &Field<T>, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
//!     where
//!         E: Encoder,
//! # { todo!() }
//!
//!     pub fn decode<'de, D, T>(cx: &D::Cx, decoder: D) -> Result<Field<T>, D::Error>
//!     where
//!         D: Decoder<'de>,
//! # { todo!() }
//! }
//! # }
//! ```
//!
//! More complete example:
//!
//! ```
//! # mod example {
//! use std::collections::HashSet;
//! use musli::{Encode, Decode};
//!
//! pub struct CustomUuid(u128);
//!
//! #[derive(Encode, Decode)]
//! struct Struct {
//!     #[musli(with = self::custom_uuid)]
//!     id: CustomUuid,
//!     #[musli(with = self::custom_set)]
//!     numbers: HashSet<u32>,
//! }
//!
//! mod custom_uuid {
//!     use musli::{Context, Decode, Decoder, Encode, Encoder};
//!
//!     use super::CustomUuid;
//!
//!     pub fn encode<E>(uuid: &CustomUuid, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
//!     where
//!         E: Encoder,
//!     {
//!         uuid.0.encode(cx, encoder)
//!     }
//!
//!     pub fn decode<'de, D>(cx: &D::Cx, decoder: D) -> Result<CustomUuid, D::Error>
//!     where
//!         D: Decoder<'de>,
//!     {
//!         Ok(CustomUuid(decoder.decode()?))
//!     }
//! }
//!
//! mod custom_set {
//!     use std::collections::HashSet;
//!     use std::hash::Hash;
//!
//!     use musli::{Context, Decode, Decoder, Encode, Encoder};
//!
//!     pub fn encode<E, T>(set: &HashSet<T>, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
//!     where
//!         E: Encoder,
//!         T: Encode<E::Mode> + Eq + Hash,
//!     {
//!         encoder.encode(set)
//!     }
//!
//!     pub fn decode<'de, D, T>(cx: &D::Cx, decoder: D) -> Result<HashSet<T>, D::Error>
//!     where
//!         D: Decoder<'de>,
//!         T: Decode<'de, D::Mode> + Eq + Hash,
//!     {
//!         decoder.decode()
//!     }
//! }
//! # }
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
//! This causes the field to use the [`DecodeTrace`] / [`EncodeTrace`] when
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
//! ```
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
//! ```
//! # use musli::{Encode, Decode};
//! # #[derive(Encode, Decode)] struct Params;
//! # #[derive(Encode, Decode)] struct Value;
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name", tag = "type")]
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
//! [`Binary`]: crate::mode::Binary
//! [`Text`]: crate::mode::Text
//! [`Decode`]: crate::Decode
//! [`DecodeBytes`]: crate::de::DecodeBytes
//! [`DecodePacked`]: crate::de::DecodePacked
//! [`Decoder::decode_buffer`]: crate::Decoder::decode_buffer
//! [`Decoder::decode_variant`]: crate::Decoder::decode_variant
//! [`Decoder`]: crate::Decoder
//! [`DecodeTrace`]: crate::de::DecodeTrace
//! [`Encode`]: crate::Encode
//! [`EncodeBytes`]: crate::en::EncodeBytes
//! [`EncodePacked`]: crate::en::EncodePacked
//! [`Encoder::encode_variant`]: crate::Encoder::encode_variant
//! [`Encoder`]: crate::Encoder
//! [`EncodeTrace`]: crate::en::EncodeTrace
//! [default mode]: crate::mode::Binary

// Parts of this documentation
