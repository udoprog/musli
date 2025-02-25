## Deriving [`Encode`] and [`Decode`] in Müsli

The [`Encode`] and [`Decode`] traits can be automatically implemented through
derives.

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Person {
    name: String,
    age: u32,
}
```

These derives come with a number of options for customizing the implementation
being generated documented below.

<br>

## Attributes

* [*Meta attributes*](#meta-attributes) which apply to the attribute itself. It
  is used to filter what scope the current attribute applies to, such as only
  applying to an `Encode` derive using `#[musli(encode_only, ..)]` or a specific
  mode such as `#[musli(mode = Json, ..)]`.

* [*Container attributes*](#container-attributes) are attributes which apply to
  the `struct` or `enum`.

* [*Variant attributes*](#variant-attributes) are attributes which apply to each
  individual variant in an `enum`.

* [*Field attributes*](#field-attributes) are attributes which apply to each
  individual field either in a `struct` or an `enum` variant.

<br>

## Meta attributes

Certain attributes affect which other attributes apply to a given context. These
are called *meta* attributes.

Meta attributes are applicable to any context, and can be used on containers,
variants, and fields.

<br>

#### `#[musli(Binary)]`, `#[musli(Text)]`, or `#[musli(mode = <path>)]`

Any sibling attributes only apply to the given `mode`. For `binary` the mode
will be [`Binary`]. For `text` the mode will be [`Text`]. Custom modes can be
specified with `#[musli(mode = <path>)]`.

This allows for building multiple distinct implementations of `Encode` and
`Decode` in parallel which has different behaviors. See [modes](#modes) for more
information.

<br>

##### Examples

The `Person` struct below uses string field names by default when the `Text`
mode is enabled, but we can change this behavior only for that particular mode
like this:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(Text, name(type = usize))]
struct Person<'a> {
    name: &'a str,
    age: u32,
}
```

<br>

#### `#[musli(encode_only)]`

The attributes only apply when implementing the `Encode` trait.

An example where this is useful is if you want to apply `#[musli(packed)]` in a
different mode, but only for encoding, since decoding packed types is not
supported for enums.

<br>

##### Examples

```rust
use musli::mode::Binary;
use musli::{Decode, Encode};

enum Packed {}

#[derive(Encode, Decode)]
#[musli(mode = Packed, encode_only, packed)]
enum Name<'a> {
    Full(&'a str),
    Given(&'a str),
}
```

<br>

#### `#[musli(decode_only)]`

The attributes only apply when implementing the `Decode` trait.

<br>

##### Examples

```rust
use musli::{Decode, Encode};

#[derive(Encode, Decode)]
struct Name<'a> {
    sur_name: &'a str,
    #[musli(Text, decode_only, name = "last")]
    last_name: &'a str,
}
```

<br>

## Container attributes

Container attributes apply to the container, such as directly on the `struct` or
`enum`. Like the uses of `#[musli(packed)]` and `#[musli(name_all =
"PascalCase")]` here:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Struct {
    /* the body of the struct */
}

#[derive(Encode, Decode)]
#[musli(name_all = "PascalCase")]
enum Enum {
    /* the body of the enum */
}
```

<br>

#### `#[musli(name_all = "..")]`

Allows for renaming every field or variant in the container. It can take any of
the following values:

* `"index"` - the index starting at `0` of the field or variant will be used.
  This is the default for the [`Binary`] mode.
* `"name"` - the literal name of the field or variant will be used. This is the
  default for the [`Text`] mode.
* `"PascalCase"` - the literal name of the field or variant will be converted to
  pascal case.
* `"camelCase"` - the literal name of the field or variant will be converted to
  camel case.
* `"snake_case"` - the literal name of the field or variant will be converted to
  snake case.
* `"SCREAMING_SNAKE_CASE"` - the literal name of the field or variant will be
  converted to screaming snake case.
* `"kebab-case"` - the literal name of the field or variant will be converted to
  kebab case.
* `"SCREAMING-KEBAB-CASE"` - the literal name of the field or variant will be
  converted to screaming kebab case.

<br>

##### Renaming struct fields

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(name_all = "PascalCase")]
struct PascalCaseStruct {
    // This will be named `FieldName`.
    field_name: u32,
}
```

<br>

##### Renaming enum variants

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(name_all = "kebab-case")]
enum KebabCase {
    // This will be named `first-variant`.
    FirstVariant {
        field_name: u32,
    },
    // This will be named `second-variant`.
    SecondVariant {
        field_name: u32,
    },
}
```

<br>

#### `#[musli(transparent)]`

This can only be used on types which have a single field. It will cause that
field to define how that variant is encoded or decoded transparently without
being treated as a field.

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Encode)]
#[musli(transparent)]
struct Struct(u32);

let data = musli::wire::to_vec(&Struct(42))?;
let actual: u32 = musli::wire::from_slice(&data)?;
assert_eq!(actual, 42u32);
Ok::<_, musli::wire::Error>(())
```

<br>

#### `#[musli(packed)]`

This attribute will disable all *tagging* and the structure will simply be
encoded with one field following another in the order in which they are defined.

Structures which are *packed* cannot be easily versioned and the two systems
communicating using them need to handle field versioning out of bounds.

This attribute is useful for performing simple decoding over "raw" bytes when
combined with an encoder which does minimal prefixing and packs fields. Using a
packed format is typically also the most efficient mode of operation in Müsli.

##### Bitwise optimizations

If a struct is tagged with `#[musli(packed)]`, and the bitwise pattern of a
given serialization is *identical* to the bitwise memory pattern of the struct,
then serialization and deserialization can be made more efficient.

Note that since [`#[repr(Rust)]`][repr-rust] is not strictly defined, it might
be necessary to mark the struct with `#[repr(C)]` to benefit from this
optimization. But this has no safety implications.

If the bitwise optimizations does not work. You can test if bitwise
optimizations are possible through [`musli::is_bitwise_encode`] and
[`musli::is_bitwise_decode`].

Bitwise optimizations are disabled if:
* Any of the field uses a custom encoding method through for example
  [`#[musli(with = <path>)]`](#musliwith--path).
* If the type implements `Drop`.

<br>

##### Packed struct

```rust
use musli::{Encode, Decode};

#[derive(Encode)]
#[musli(packed)]
struct Struct {
    field1: u32,
    field2: u64,
}

let data = musli::storage::to_vec(&Struct {
    field1: 1,
    field2: 2,
})?;

assert_eq!(data.as_slice(), [1, 2]);
Ok::<_, musli::storage::Error>(())
```

<br>

##### Bitwise encoded fields

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Struct {
    a: u32,
    b: u32,
}

const _: () = assert!(musli::is_bitwise_encode::<Struct>());
const _: () = assert!(musli::is_bitwise_decode::<Struct>());
```

Note that some combinations of fields currently only support encoding in one
direction. This is the case for `NonZero` types, since they cannot inhabit all
possible bit patterns.

```rust
use core::num::NonZero;
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Struct {
    a: NonZero<u32>,
    b: u32,
}

const _: () = assert!(musli::is_bitwise_encode::<Struct>());
const _: () = assert!(!musli::is_bitwise_decode::<Struct>());
```

Bitwise optimizations are disabled if custom encoding is specified:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Struct {
    a: u32,
    #[musli(with = musli::serde)]
    b: u32,
}

const _: () = assert!(!musli::is_bitwise_encode::<Struct>());
const _: () = assert!(!musli::is_bitwise_decode::<Struct>());
```

Bitwise optimizations are disabled if the type implements [`Drop`]:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Struct {
    a: u32,
    b: u32,
}

const _: () = assert!(!musli::is_bitwise_encode::<Struct>());
const _: () = assert!(!musli::is_bitwise_decode::<Struct>());

impl Drop for Struct {
    fn drop(&mut self) {
    }
}
```

<br>

#### `#[musli(name(type = <type>))]`

This indicates which type any contained `#[musli(name = ..)]` attributes should
have. Tags can usually be inferred, but specifying this field ensures that all
tags have a single well-defined type.

The following values are treated specially:
* `str` applies `#[musli(name_all = "name")]` by default.
* `[u8]` applies `#[musli(name_all = "name")]` by default.

Apart from those two types, the `name(type)` must be a sized type which
implements [`Encode`] and [`Decode`].

The default type depends on the mode in use:
* [`Binary`] and any other custom mode uses indexed fields, the equivalent of
  `#[musli(name(type = usize))]`.
* [`Text`] uses literal text fields by their name, the equivalent of
  `#[musli(name(type = str))]`.

<br>

##### Examples

```rust
use core::fmt;

use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(transparent)]
struct CustomTag<'a>(&'a [u8]);

impl fmt::Display for CustomTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}

#[derive(Encode, Decode)]
#[musli(name(type = CustomTag))]
struct Struct {
    #[musli(name = CustomTag(b"name in bytes"))]
    name: String,
}

#[derive(Encode, Decode)]
#[musli(name(type = CustomTag))]
enum EnumWithCustomTag {
    #[musli(name = CustomTag(b"variant one"))]
    Variant1 {
        /* .. */
    },
}
```

<br>

#### `#[musli(name(format_with = <path>))]`

This indicates the method which should be used to format `#[musli(name = ..)]`
attributes for diagnostics.

This can be useful if the default [`Debug`] implementation might not be
particularly helpful, one example is `#[musli(name(type = [u8]))]` which formats
the value as an array of numbers. Here `#[musli(name(format_with = BStr::new))]`
might be more helpful.

##### Examples

```rust
use bstr::BStr;
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(name(type = [u8], format_with = BStr::new))]
struct StructBytesArray {
    #[musli(name = b"field1")]
    field1: u32,
    #[musli(name = b"field2")]
    field2: u32,
}

```

<br>

#### `#[musli(name(method = "sized" | "unsized" | "unsized_bytes"))]`

This allows for explicitly setting which method should be used to decode names.
Available options are:

* `"sized"` (default) - decode as a sized value.
* `"unsized"` - the name is decoded as an unsized value, this is the default if
  for example `#[musli(name(type = str))]` is used.
* `"unsized_bytes"` - the name is decoded as a unsized bytes, this is the
  default if for example `#[musli(name(type = [u8]))]` is used.

This can be overrided for values which are unsized, but cannot be determined
through heuristics. Such a type must also implement [`Decode`] (for `"sized"`),
`DecodeUnsized`, or `DecodeUnsizedBytes` as appropriate.

<br>

#### `#[musli(bound = {..})]` and `#[musli(decode_bound<'de, A> = {..})]`

These attributes can be used to apply bounds to an [`Encode`] or [`Decode`]
implementation.

These are necessary to use when a generic container is used to ensure that the
given parameter implements the necessary bounds.

`#[musli(bound = {..})]` applies to all implementations while
`#[musli(decode_bound<'de, A> = {..})]` only applies to the [`Decode`]
implementation. The latter allows for using the decode lifetime and allocator
parameter. If these parameters are not part of the type signature, they can be
specified in the `decode_bound` parameter directly like
`#[musli(decode_bound<'de, A> = {..})]`.

An HRTB can also be used like `#[musli(decode_bound<A> = {T: for<'de>
Decode<'de, Binary, A>})]`.

<br>

##### Examples

```rust
use musli::{Decode, Encode};
use musli::mode::{Binary, Text};

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
#[musli(Binary, bound = {T: Encode<Binary>}, decode_bound<'de, A> = {T: Decode<'de, Binary, A>})]
#[musli(Text, bound = {T: Encode<Text>}, decode_bound<'de, A> = {T: Decode<'de, Text, A>})]
pub struct GenericWithBound<T> {
    value: T,
}
```

<br>

## Enum attributes

<br>

#### `#[musli(tag = ..)]`

This attribute causes the enum to be internally tagged, with the given tag. See
[enum representations](#enum-representations) for details on this
representation.

The value of the attributes specifies the name of the tag to use for this.

The `tag` attribute supports the same options such as `name`, which are:
* `#[tag(value = ..)]` - Specify the value of the tag when list options are
  used.
* `#[tag(type = ..)]` - Specify the type of the tag.
* `#[tag(format_with = ..)]` - Specify how to format the tag for diagnostics.

<br>

##### Using a string tag

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(tag = "type")]
enum Message {
    Request { id: String, method: String },
    Reply { id: String, body: Vec<u8> },
}
```

<br>

#### `#[musli(tag = .., content = ..)]`

This attribute causes the enum to be adjacently tagged, with the given tag. See
[enum representations](#enum-representations) for details on this
representation.

The value of the attributes specifies the name of the tag to use and the content
where the data will be stored.

The `tag` and `content` attributes support the same options such as `name`,
which are:
* `#[tag(value = ..)]` or `#[content(value = ..)]` - Specify the value of the
  tag or content when list options are used.
* `#[tag(type = ..)]` or `#[content(type = ..)]` - Specify the type of the tag
  or content.
* `#[tag(format_with = ..)]` or `#[content(format?with = ..)]` - Specify how to
  format the tag or content for diagnostics.

<br>

##### Using a string tag

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(tag = "type", content = "data")]
enum Message {
    Request { id: String, method: String },
    Reply { id: String, body: Vec<u8> },
}
```

##### Using a tag with custom formatting

```rust
use bstr::BStr;
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(tag(value = b"type", format_with = BStr::new), content(value = b"data", format_with = BStr::new))]
enum Message {
    Request { id: String, method: String },
    Reply { id: String, body: Vec<u8> },
}
```

<br>

##### Using numerical tagging

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(name(type = usize), tag = 10)]
enum Message {
    #[musli(name = 1)]
    Request { id: String, method: String },
    #[musli(name = 2)]
    Reply { id: String, body: Vec<u8> },
}
```

<br>

## Variant attributes

*Variant attributes* are attributes which apply to each individual variant in an
`enum`. Like the use of `#[musli(name = ..)]` here:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
enum Enum {
    #[musli(transparent)]
    Variant(Variant),
    #[musli(Text, name = "Other")]
    Something {
        /* variant body */
    },
}

#[derive(Encode, Decode)]
struct Variant {
    field: String,
}
```

<br>

#### `#[musli(name = ..)]`

This allows for renaming a variant from its default value. It can take any value
(including complex ones) that can be serialized with the current encoding, such
as:

* `#[musli(name = 1)]`
* `#[musli(name = "Hello World")]`
* `#[musli(name = b"box\0")]`
* `#[musli(name = SomeStruct { field: 42 })]` (if `SomeStruct` implements
  [`Encode`] and [`Decode`] as appropriate).

If the type of the tag is ambiguous it can be explicitly specified through the
`#[musli(name_type)]` attribute.

<br>

#### `#[musli(pattern = ..)]` or `#[musli(pattern = (<pat> | <pat2> | ..))]`

A pattern to match for decoding a variant.

This allows for more flexibility when decoding variants.

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
enum Enum {
    Variant1,
    Variant2,
    #[musli(Binary, pattern = 2..=4)]
    Deprecated,
}
```

Multiple patterns are supported with parenthesis:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
enum Enum {
    Variant1,
    Variant2,
    #[musli(Binary, pattern = (2..=4 | 10..=20))]
    Deprecated,
}
```

<br>

#### `#[musli(name_all = "..")]`

Allows for renaming every field in a variant. It can take any of the following
values:

* `"index"` - the index starting at `0` of the field will be used. This is the
  default for the [`Binary`] mode.
* `"name"` - the literal name of the field will be used. This is the default for
  the [`Text`] mode.
* `"PascalCase"` - the literal name of the field will be converted to pascal
  case.
* `"camelCase"` - the literal name of the field will be converted to camel case.
* `"snake_case"` - the literal name of the field will be converted to snake
  case.
* `"SCREAMING_SNAKE_CASE"` - the literal name of the field will be converted to
  screaming snake case.
* `"kebab-case"` - the literal name of the field will be converted to kebab
  case.
* `"SCREAMING-KEBAB-CASE"` - the literal name of the field will be converted to
  screaming kebab case.

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
enum PascalCaseEnum {
    #[musli(name_all = "PascalCase")]
    Variant {
        // This field will be named `FieldName`.
        field_name: u32,
    }
}
```

<br>

#### `#[musli(name(type = <type>))]`

This indicates which type any contained `#[musli(tag = ..)]` attributes should
have. Tags can usually be inferred, but specifying this field ensures that all
tags have a well-defined type.

The following values are treated specially:
* `str` applies `#[musli(name_all = "name")]` by default.
* `[u8]` applies `#[musli(name_all = "name")]` by default.

<br>

##### Examples

```rust
use core::fmt;

use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(transparent)]
struct CustomTag<'a>(&'a [u8]);

impl fmt::Display for CustomTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}

#[derive(Encode, Decode)]
#[musli(name(type = usize))]
enum Enum {
    #[musli(name = 0, name(type = CustomTag))]
    Variant {
        #[musli(name = CustomTag(b"field1"))]
        field1: u32,
        #[musli(name = CustomTag(b"field2"))]
        field2: u32,
    },
    #[musli(name = 1, name_all = "name")]
    Variant2 {
        field1: u32,
        field2: u32,
    },
}
```

<br>

#### `#[musli(name(method = "sized" | "unsized" | "unsized_bytes"))]`

This allows for explicitly setting which method should be used to decode field
names. Available options are:

* `"sized"` (default) - the name is decoded as a sized value.
* `"unsized"` - the name is decoded as an unsized value, this is the default if
  for example `#[musli(name(type = str))]` is used.
* `"unsized_bytes"` - the name is decoded as a unsized bytes, this is the
  default if for example `#[musli(name(type = [u8]))]` is used.

This can be overrided for values which are unsized, but cannot be determined
through heuristics. Such a type must also implement [`Decode`] (for `"sized"`),
`DecodeUnsized`, or `DecodeUnsizedBytes` as appropriate.

##### Examples

```rust
use bstr::BStr;
use musli::{Encode, Decode};
use musli::de::{Decoder, DecodeUnsized};

use core::fmt;
use core::ops::Deref;

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name(type = UnsizedBytes, method = "unsized"))]
pub struct StructWithUnsizedBytes {
    #[musli(name = UnsizedBytes::new(&[1, 2, 3, 4]), pattern = UnsizedBytes([1, 2, 3, 4]))]
    field1: u32,
    #[musli(name = UnsizedBytes::new(&[2, 3, 4, 5]), pattern = UnsizedBytes([2, 3, 4, 5]))]
    field2: u32,
}

#[derive(Encode)]
#[repr(transparent)]
#[musli(transparent)]
struct UnsizedBytes(#[musli(bytes)] [u8]);

impl<'de, M> DecodeUnsized<'de, M> for UnsizedBytes {
    fn decode_unsized<D, F, O>(decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de, Mode = M>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        decoder.decode_unsized_bytes(|bytes: &[u8]| f(UnsizedBytes::new(bytes)))
    }
}

impl UnsizedBytes {
    const fn new(data: &[u8]) -> &Self {
        // SAFETY: `UnsizedBytes` is a transparent wrapper around `[u8]`.
        unsafe { &*(data as *const [u8] as *const UnsizedBytes) }
    }
}

impl Deref for UnsizedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for UnsizedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(&self.0).fmt(f)
    }
}

impl fmt::Debug for UnsizedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(&self.0).fmt(f)
    }
}
```

<br>

#### `#[musli(transparent)]`

This can only be used on variants which have a single field. It will cause that
field to define how that variant is encoded or decoded transparently without
being treated as a field.

<br>

#### `#[musli(default)]`

This defines the variant that will be used in case no other variant matches.
Only one such variant can be defined.

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
enum Animal {
    Cat,
    Dog,
    #[musli(default)]
    Unknown,
}
```

<br>

## Field attributes

*Field attributes* are attributes which apply to each individual field either in
a `struct` or an `enum` variant. Like the uses of `#[musli(all)]` here:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Struct {
    #[musli(Text, name = "other")]
    something: String,
    #[musli(skip, default = default_field)]
    skipped_field: u32,
}

fn default_field() -> u32 {
    42
}

#[derive(Encode, Decode)]
enum Enum {
    Variant {
        #[musli(Text, name = "other")]
        something: String,
    }
}
```

<br>

#### `#[musli(skip)]`

This attribute means that the entire field is skipped. If a field is decoded it
uses [`Default::default`] to construct the value. Other defaults can be
specified with [`#[musli(default = <path>)]`][#muslidefault--path].

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Person {
    name: String,
    #[musli(skip)]
    age: Option<u32>,
    #[musli(skip, default = default_country)]
    country: Option<String>,
}

fn default_country() -> Option<String> {
    Some(String::from("Earth"))
}
```

<br>

#### `#[musli(default [= <path>])]`

When a field is absent or disabled with `#[musli(skip)]`, this attribute
specifies that a default value should be used instead.

If `#[musli(default)]` is specified, the default value is constructed using
[`Default::default`].

If `#[musli(default = <path>)]` is specified, the default value is constructed
by calling the function at `<path>`.

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Person {
    name: String,
    #[musli(default)]
    age: Option<u32>,
    #[musli(default = default_height)]
    height: Option<u32>,
    #[musli(skip, default = default_meaning)]
    meaning: u32,
}

fn default_height() -> Option<u32> {
    Some(180)
}

fn default_meaning() -> u32 {
    42
}
```

<br>

#### `#[musli(name = ..)]`

This allows for renaming a field from its default value. It can take any value
(including complex ones) that can be serialized with the current encoding, such
as:

* `#[musli(name = 1)]`
* `#[musli(name = "Hello World")]`
* `#[musli(name = b"box\0")]`
* `#[musli(name = SomeStruct { field: 42 })]` (if `SomeStruct` implements
  [`Encode`] and [`Decode`] as appropriate).

If the type of the tag is ambiguous it can be explicitly specified through the
`#[musli(name(type))]` variant or container attributes.

<br>

#### `#[musli(pattern = ..)]` or `#[musli(pattern = (<pat> | <pat2> | ..))]`

A pattern to match for decoding the given field.

This allows for more flexibility when decoding fields.

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Struct {
    field1: u32,
    field2: u32,
    #[musli(Binary, pattern = 2..=4)]
    other: u32,
}
```

Multiple patterns are supported with parenthesis:

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Struct {
    field1: u32,
    field2: u32,
    #[musli(Binary, pattern = (2..=4 | 10..=20))]
    other: u32,
}
```

<br>

#### `#[musli(packed)]`

This specifies that encoding and decoding should happen through the
[`EncodePacked`] and [`DecodePacked`] traits, instead of the default [`Encode`]
and [`Decode`].

These traits contained implementations which are biased towards encoding the
field as a compact, non-future compatible pack. In essense, the fields are
encoded "one after another" without any metadata separating them. So for packed
fields, the order, types and number of the fields are important.

<br>

##### Examples

```rust
use std::collections::VecDeque;

use musli::{Decode, Encode};

#[derive(Decode, Encode)]
struct Container {
    #[musli(packed)]
    tuple: (u32, u64),
    #[musli(packed)]
    array: [u32; 4],
}
```

<br>

#### `#[musli(bytes)]`

This specifies that encoding and decoding should happen through the
[`EncodeBytes`] and [`DecodeBytes`] traits, instead of the default [`Encode`]
and [`Decode`].

These traits contained implementations which are biased towards encoding the
field as an array of bytes.

<br>

##### Examples

```rust
use std::collections::VecDeque;

use musli::{Decode, Encode};

#[derive(Decode, Encode)]
struct Container<'de> {
    #[musli(bytes)]
    vec: Vec<u8>,
    #[musli(bytes)]
    vec_deque: VecDeque<u8>,
    #[musli(bytes)]
    bytes: &'de [u8],
}
```

<br>

#### `#[musli(with = <path>)]`

This specifies the path to a module to use instead of the fields default
[`Encode`] or [`Decode`] implementations.

It expects `encode` and `decode` function to be defined in the path being
specified.

<br>

##### Basic example

```rust
mod example {
    use musli::{Decode, Encode};

    #[derive(Decode, Encode)]
    struct Container {
        #[musli(with = self::module)]
        field: Field,
    }

    struct Field {
        /* internal */
    }

    mod module {
        use musli::{Decoder, Encoder};

        use super::Field;

        pub fn encode<E>(field: &Field, encoder: E) -> Result<(), E::Error>
        where
            E: Encoder,
        {
            todo!()
        }

        pub fn decode<'de, D>(decoder: D) -> Result<Field, D::Error>
        where
            D: Decoder<'de>,
        {
            todo!()
        }
    }
}
```

<br>

##### Generic implementation

```rust
mod example {
    use musli::{Decode, Encode};

    #[derive(Decode, Encode)]
    struct Container {
        #[musli(with = self::module)]
        field: Field<u32>,
    }

    struct Field<T> {
        value: T,
    }

    mod module {
        use musli::{Decoder, Encoder};

        use super::Field;

        pub fn encode<E, T>(field: &Field<T>, encoder: E) -> Result<(), E::Error>
        where
            E: Encoder,
        {
            todo!()
        }

        pub fn decode<'de, D, T>(decoder: D) -> Result<Field<T>, D::Error>
        where
            D: Decoder<'de>,
        {
            todo!()
        }
    }
}
```

<br>

##### More complete example

```rust
mod example {
    use std::collections::HashSet;
    use musli::{Encode, Decode};

    pub struct CustomUuid(u128);

    #[derive(Encode, Decode)]
    struct Struct {
        #[musli(with = self::custom_uuid)]
        id: CustomUuid,
        #[musli(with = self::custom_set)]
        numbers: HashSet<u32>,
    }

    mod custom_uuid {
        use musli::{Context, Decode, Decoder, Encode, Encoder};

        use super::CustomUuid;

        pub fn encode<E>(uuid: &CustomUuid, encoder: E) -> Result<(), E::Error>
        where
            E: Encoder,
        {
            uuid.0.encode(encoder)
        }

        pub fn decode<'de, D>(decoder: D) -> Result<CustomUuid, D::Error>
        where
            D: Decoder<'de>,
        {
            Ok(CustomUuid(decoder.decode()?))
        }
    }

    mod custom_set {
        use std::collections::HashSet;
        use std::hash::Hash;

        use musli::{Context, Decode, Decoder, Encode, Encoder};

        pub fn encode<E, T>(set: &HashSet<T>, encoder: E) -> Result<(), E::Error>
        where
            E: Encoder,
            T: Encode<E::Mode> + Eq + Hash,
        {
            encoder.encode(set)
        }

        pub fn decode<'de, D, T>(decoder: D) -> Result<HashSet<T>, D::Error>
        where
            D: Decoder<'de>,
            T: Decode<'de, D::Mode, D::Allocator> + Eq + Hash,
        {
            decoder.decode()
        }
    }
}
```

<br>

#### `#[musli(skip_encoding_if = <path>)]`

This adds a condition to skip encoding a field entirely if the condition is
true. This is very commonly used to skip over encoding `Option<T>` fields.

<br>

##### Examples

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Person {
    name: String,
    #[musli(skip_encoding_if = Option::is_none)]
    age: Option<u32>,
}
```

#### `#[musli(trace)]`

This causes the field to use the [`DecodeTrace`] / [`EncodeTrace`] when encoding
the field. This is left optional for types where enabling tracing for the field
requires extra traits to be implemented, such as `HashMap<K, V>` where we'd need
`K` to implement `fmt::Display`.

Without using the `trace` attribute below, the keys in the `values` field would
not be instrumented, so with a decoding error you'd see this:

```text
.values: not numeric (at bytes 15-16)
```

Instead of this (where `#[musli(trace)]` is enabled):

```text
.values[Hello]: not numeric (at bytes 15-16)
```

<br>

##### Examples

```rust
use std::collections::HashMap;

use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Collection {
    #[musli(trace)]
    values: HashMap<String, u32>,
}
```

<br>

## Modes

If you've paid close attention to the [`Encode`] and [`Decode`] traits you might
notive that they have an extra parameter called `M`. This stands for "mode".

This parameter allows us to have different implementations of these traits for
the same type.

By default we implement two special modes, which each have subtly different
default behaviors:
* [`Binary`] and any other custom mode uses indexed fields, the equivalent of
  `#[musli(name(type = usize))]`.
* [`Text`] uses literal text fields by their name, the equivalent of
  `#[musli(name(type = str))]`.

When it comes to deriving these traits you can scope attributes to apply to any
mode including custom local ones. This is done using the `#[musli(mode = ..)]`
meta attribute like this:

```rust
use musli::{Encode, Decode};
use musli::mode::Binary;
use musli::json::Encoding;

#[derive(Encode, Decode)]
struct Person<'a> {
    #[musli(Text, name = "name")]
    not_name: &'a str,
    age: u32,
}

const TEXT: Encoding = Encoding::new();
const BINARY: Encoding<Binary> = Encoding::new().with_mode();

let named = TEXT.to_vec(&Person { not_name: "Aristotle", age: 61 })?;
assert_eq!(named.as_slice(), br#"{"name":"Aristotle","age":61}"#);

let indexed = BINARY.to_vec(&Person { not_name: "Plato", age: 84 })?;
assert_eq!(indexed.as_slice(), br#"{"0":"Plato","1":84}"#);
Ok::<_, musli::json::Error>(())
```

So the `#[musli(mode)]` atttribute is supported in any position. And any of its
sibling attributes will be added to the given *alternative* mode, rather the
[default mode].

<br>

## Enum representations

Müsli supports the following enum representations, which mimics the ones
supported by *serde*:

* Externally tagged (*default*).
* Internally tagged when `#[musli(tag = ..)]` is specified on the enum.
* Adjacently tagged when both `#[musli(tag = ..)]` and `#[musli(content)]` are
  specified.

<br>

### Externally tagged

```rust
# use musli::{Encode, Decode};
# #[derive(Encode, Decode)] struct Params;
# #[derive(Encode, Decode)] struct Value;
#[derive(Encode, Decode)]
enum Message {
    Request { id: String, method: String, params: Params },
    Response { id: String, result: Value },
}
```

When an enum is externally tagged it is represented by a single field indicating
the variant of the enum.

```json
{"Request": {"id": "...", "method": "...", "params": {...}}}
```

This is the most portable representation and is supported by most formats. It
has special support in the [`Encoder`] and [`Decoder`] traits through
[`Encoder::encode_variant`] and [`Decoder::decode_variant`].

Conceptually this can be considered as a "pair", where the variant tag can be
extracted from the format before the variant is decoded.

<br>

### Internally tagged

```rust
# use musli::{Encode, Decode};
# #[derive(Encode, Decode)] struct Params;
# #[derive(Encode, Decode)] struct Value;
#[derive(Encode, Decode)]
#[musli(tag = "type")]
enum Message {
    Request { id: String, method: String, params: Params },
    Response { id: String, result: Value },
}
```

In JSON, the `Message::Request` would be represented as:

```json
{"type": "Request", "id": "...", "method": "...", "params": {...}}
```

This is only supported by formats which are *self descriptive*, which is a
requirement for the format to be buffered through [`Decoder::decode_buffer`].

It is necessary to buffer the value, since we need to inspect the fields of a
map for the field corresponding to the `tag`, and then use this to determine
which decoder implementation to call.

<br>

### Adjacently tagged

```rust
# use musli::{Encode, Decode};
# #[derive(Encode, Decode)] struct Params;
# #[derive(Encode, Decode)] struct Value;
#[derive(Encode, Decode)]
#[musli(tag = "type", content = "data")]
enum Message {
    Request { id: String, method: String, params: Params },
    Response { id: String, result: Value },
}
```

In JSON, the `Message::Request` would be represented as:

```json
{"type": "Request", "data": {"id": "...", "method": "...", "params": {...}}}
```

This is only supported by formats which are *self descriptive*, which is a
requirement for the format to be buffered through [`Decoder::decode_buffer`].

It is necessary to buffer the value, since we need to inspect the fields of a
map for the field corresponding to the `tag`, and then use this to determine
which decoder implementation to call.

[`Binary`]: <https://docs.rs/musli/latest/musli/mode/enum.Binary.html>
[`Decode`]: <https://docs.rs/musli/latest/musli/trait.Decode.html>
[`DecodeBytes`]: <https://docs.rs/musli/latest/musli/de/trait.DecodeBytes.html>
[`DecodePacked`]: <https://docs.rs/musli/latest/musli/de/trait.DecodePacked.html>
[`Decoder::decode_buffer`]: <https://docs.rs/musli/latest/musli/trait.Decoder.html#method.decode_buffer>
[`Decoder::decode_variant`]: <https://docs.rs/musli/latest/musli/trait.Decoder.html#method.decode_variant>
[`Decoder`]: <https://docs.rs/musli/latest/musli/trait.Decoder.html>
[`DecodeTrace`]: <https://docs.rs/musli/latest/musli/trait.DecodeTrace.html>
[`Drop`]: <https://doc.rust-lang.org/std/ops/trait.Drop.html>
[`Encode`]: <https://docs.rs/musli/latest/musli/trait.Encode.html>
[`EncodeBytes`]: <https://docs.rs/musli/latest/musli/en/trait.EncodeBytes.html>
[`EncodePacked`]: <https://docs.rs/musli/latest/musli/en/trait.EncodePacked.html>
[`Encoder::encode_variant`]: <https://docs.rs/musli/latest/musli/trait.Encoder.html#method.encode_variant>
[`Encoder`]: <https://docs.rs/musli/latest/musli/trait.Encoder.html>
[`EncodeTrace`]: <https://docs.rs/musli/latest/musli/en/trait.EncodeTrace.html>
[`musli::is_bitwise_decode`]: https://docs.rs/musli/latest/musli/fn.is_bitwise_decode.html
[`musli::is_bitwise_encode`]: https://docs.rs/musli/latest/musli/fn.is_bitwise_encode.html
[`Text`]: <https://docs.rs/musli/latest/musli/mode/enum.Text.html>
[default mode]: <https://docs.rs/musli/latest/musli/mode/enum.Binary.html>
[repr-rust]: <https://doc.rust-lang.org/nomicon/repr-rust.html>
