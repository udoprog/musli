//! # The `Encode` and `Decode` derives
//!
//! The `Encode` and `Decode` derives allows for automatically implementing
//! [Decode] and [Encode].
//!
//! They come with a number of options for customizing their implementation,
//! detailed below.
//!
//! * *Container attributes* are attributes which apply to the `struct` or
//!   `enum`. Like the uses of `#[musli(packed)]` and
//!   `#[musli(default_variant_tag = "name")]` here:
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(packed)]
//!   struct Struct {
//!       /* the body of the struct */
//!   }
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_variant_tag = "name")]
//!   enum Enum {
//!       /* the body of the struct */
//!   }
//!   ```
//!
//! * *Variant attributes* are attributes which apply to each individual variant
//!   in an `enum`. Like the use of `#[musli(name)]` here:
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_variant_tag = "name")]
//!   enum Enum {
//!       #[musli(tag = "Other")]
//!       Something {
//!           /* variant body */
//!       }
//!   }
//!   ```
//!
//! * *Field attributes* are attributes which apply to each individual field
//!   either in a `struct` or an `enum` variant. Like the uses of
//!   `#[musli(tag)]` here:
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_field_tag = "name")]
//!   struct Struct {
//!       #[musli(tag = "other")]
//!       something: String,
//!   }
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_field_tag = "name")]
//!   enum Enum {
//!       Variant {
//!           #[musli(tag = "other")]
//!           something: String,
//!       }
//!   }
//!   ```
//!
//! ## Container attributes
//!
//! * `#[musli(tag_type = ..)]` indicates which type any contained `#[musli(tag
//!   = ..)]` attributes should have. Tags can usually be inferred, but
//!   specifying this field ensures that all tags have a well-defined type.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//!   #[musli(transparent)]
//!   struct CustomTag<'a>(&'a [u8]);
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(tag_type = CustomTag)]
//!   struct Struct {
//!       #[musli(tag = CustomTag(b"name in bytes"))]
//!       name: String,
//!   }
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(tag_type = CustomTag)]
//!   enum EnumWithCustomTag {
//!       #[musli(tag = CustomTag(b"variant one"))]
//!       Variant1 {
//!           /* .. */
//!       },
//!   }
//!   ```
//!
//! * `#[musli(default_field_tag = "..")]` determines how the default tag for a
//!   field is determined. It can take either `"name"` or `"index"`.
//!
//!   `#[musli(default_field_tag = "index")]` will use the index of the field.
//!   This is the default.
//!
//!   `#[musli(default_field_tag = "name")]` will use the name of the field.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_field_tag = "name")]
//!   struct Struct {
//!       field1: u32,
//!       field2: u32,
//!   }
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_field_tag = "name")]
//!   enum Enum {
//!       Variant1 {
//!           field1: u32,
//!       },
//!       Variant2 {
//!           field1: u32,
//!       },
//!   }
//!   ```
//!
//! * `#[musli(default_variant_tag = "..")]` determines how the default tag for
//!   a variant is determined. It can take either `"name"` or `"index"`.
//!
//!   `#[musli(default_variant_tag = "index")]` will use the index of the
//!   variant. This is the default.
//!
//!   `#[musli(default_variant_tag = "name")]` will use the name of the variant.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_variant_tag = "name")]
//!   enum Enum {
//!       Variant1 {
//!           field1: u32,
//!       },
//!       Variant2 {
//!           field1: u32,
//!       },
//!   }
//!   ```
//!
//! * `#[musli(transparent)]` can only be used on types which have a single
//!   field. It will cause that field to define how that variant is encoded or
//!   decoded transparently without being treated as a field.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode)]
//!   #[musli(transparent)]
//!   struct Struct(u32);
//!
//!   # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!   let data = musli_wire::to_vec(&Struct(42))?;
//!   assert_eq!(data, vec![musli_wire::types::TypeTag::Continuation as u8, 42]);
//!   # Ok(()) }
//!   ```
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
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode)]
//!   #[musli(packed)]
//!   struct Struct {
//!       field1: u32,
//!       field2: u32,
//!       field3: u32,
//!   }
//!
//!   # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!   let data = musli_storage::to_vec(&Struct {
//!       field1: 1,
//!       field2: 2,
//!       field3: 3,
//!   })?;
//!
//!   assert_eq!(data, vec![1, 2, 3]);
//!   # Ok(()) }
//!   ```
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
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(default_field_tag = "index")]
//!   enum Enum {
//!       #[musli(default_field_tag = "name")]
//!       Variant {
//!           field1: u32,
//!       },
//!       Variant2 {
//!           field1: u32,
//!       },
//!   }
//!   ```
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
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//!   #[musli(transparent)]
//!   struct CustomTag<'a>(&'a [u8]);
//!
//!   #[derive(Encode, Decode)]
//!   #[musli(tag_type = usize)]
//!   enum Enum {
//!       #[musli(tag_type = CustomTag)]
//!       Variant {
//!           #[musli(tag = CustomTag(b"name in bytes"))]
//!           name: String,
//!       }
//!   }
//!   ```
//!
//! * `#[musli(default)]` defines the variant that will be used in case no other
//!   variant matches. Only one such variant can be defined.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Debug, PartialEq, Eq, Encode, Decode)]
//!   enum Animal {
//!       #[musli(tag = "cat")]
//!       Cat,
//!       #[musli(tag = "dog")]
//!       Dog,
//!       #[musli(default)]
//!       Unknown,
//!   }
//!   ```
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
//!   ```rust,ignore
//!   fn encode<E>(field: &Field, encoder: E) -> Result<(), E::Error>
//!   where
//!      E: Encoder;
//!   ```
//!
//!   `encode` for decoding the field, which should match the following
//!   signature:
//!
//!   ```rust,ignore
//!   fn decode<'de, D>(decoder: D) -> Result<Field, D::Error>
//!   where
//!       D: Decoder<'de>;
//!   ```
//!
//!   ```
//!   # mod types {
//!   use musli::{Encode, Decode};
//!
//!   pub struct CustomUuid(u128);
//!
//!   #[derive(Encode, Decode)]
//!   struct Struct {
//!       #[musli(with = self::custom_uuid)]
//!       name: CustomUuid,
//!   }
//!
//!   mod custom_uuid {
//!       use musli::en::{Encode, Encoder};
//!       use musli::de::{Decode, Decoder};
//!
//!       use super::CustomUuid;
//!
//!       pub fn encode<E>(uuid: &CustomUuid, encoder: E) -> Result<(), E::Error>
//!       where
//!           E: Encoder
//!       {
//!           uuid.0.encode(encoder)
//!       }
//!
//!       pub fn decode<'de, D>(decoder: D) -> Result<CustomUuid, D::Error>
//!       where
//!           D: Decoder<'de>
//!       {
//!           Ok(CustomUuid(u128::decode(decoder)?))
//!       }
//!   }
//!   # }
//!   ```
//!
//! * `#[musli(default)]` constructs the field using [Default::default] in case
//!   it's not available. This is only used when a field is missing during
//!   decoding.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   struct Person {
//!       name: String,
//!       #[musli(default)]
//!       age: Option<u32>,
//!   }
//!   ```
//!
//! * `#[musli(skip_encoding_if = <path>)]` adds a condition to skip encoding a
//!   field entirely if the condition is true. This is very commonly used to
//!   skip over encoding `Option<T>` fields.
//!
//!   ```
//!   use musli::{Encode, Decode};
//!
//!   #[derive(Encode, Decode)]
//!   struct Person {
//!       name: String,
//!       #[musli(skip_encoding_if = Option::is_none)]
//!       age: Option<u32>,
//!   }
//!   ```
//!
//! [Encode]: crate::Encode
//! [Decode]: crate::Decode
