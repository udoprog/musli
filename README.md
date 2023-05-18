# musli

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Müsli is a flexible and generic binary serialization framework.

The central components of the framework are the [`Encode`] and [`Decode`]
derives. They are thoroughly documented in the [`derives`] module.

I've chosen to internally use the term "encoding", "encode", and "decode"
because it's common terminology when talking about binary formats. It's also
distinct from [`serde`]'s use of "serialization" allowing for the ease of
using both libraries side by side if desired.

<br>

## Quick guide

* For information on how to implement [`Encode`] and [`Decode`], see
  [`derives`].
* For information on how this library is tested, see [`musli-tests`].

<br>

## Usage

Add the following to your `Cargo.toml` using the [format](#formats) you want
to use:

```toml
musli = "0.0.46"
musli-wire = "0.0.46"
```

<br>

## Design

Müsli is designed with similar principles as [`serde`]. Relying on Rust's
powerful trait system to generate code which can largely be optimized away.
The end result should be very similar to a handwritten encoding. The binary
serialization formats provided aim to efficiently and natively support and
accurately encode every type and data structure available in Rust.

As an example of this, these two functions both produce the same assembly on
my machine (built with `--release`):

```rust
const ENCODING: Encoding<DefaultMode, Fixed<NativeEndian>, Variable> =
    Encoding::new().with_fixed_integers_endian();

#[derive(Encode, Decode)]
#[musli(packed)]
pub struct Storage {
    left: u32,
    right: u32,
}

fn with_musli(storage: &Storage) -> Result<[u8; 8]> {
    let mut array = [0; 8];
    ENCODING.encode(&mut array[..], storage)?;
    Ok(array)
}

fn without_musli(storage: &Storage) -> Result<[u8; 8]> {
    let mut array = [0; 8];
    array[..4].copy_from_slice(&storage.left.to_ne_bytes());
    array[4..].copy_from_slice(&storage.right.to_ne_bytes());
    Ok(array)
}
```

The heavy lifting in user code is done through the [`Encode`] and [`Decode`]
derives. They are both documented in the [`derives`] module. Müsli operates
solely based on the schema derived from the types it uses.

```rust
use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Person {
    /* .. fields .. */
}
```

> **Note** by default a field is identified by its *numerical index* which
> would change if they are re-ordered. Renaming fields and setting a default
> naming policy can be done by configuring the [`derives`].

Where Müsli differs in design is that we make sparser use of the visitor
pattern. Instead the encoding interacts with the framework through encoding
interfaces that describe "what it wants" and leverages GATs to make the API
ergonomic and efficient.

Note how decoding a sequence [does not require the use of a visitor]:

```rust
use musli::de::{Decode, Decoder, SequenceDecoder};
use musli::mode::Mode;

struct MyType {
    data: Vec<String>,
}

impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut seq = decoder.decode_sequence()?;
        let mut data = Vec::with_capacity(seq.size_hint().or_default());

        while let Some(decoder) = seq.next()? {
            data.push(Decode::<M>::decode(decoder)?);
        }

        seq.end()?;

        Ok(Self {
            data
        })
    }
}
```

Another major aspect where Müsli differs is in the concept of
[modes](#modes) (note the `M` parameter above). Since this is a parameter of
the `Encode` and `Decode` traits it allows for the same data model to be
serialized in many different ways. This is a larger topic and is covered
further down.

[does not require the use of a visitor]: https://docs.rs/serde/latest/serde/trait.Deserializer.html#tymethod.deserialize_seq

<br>

## Formats

Formats are currently distinguished by supporting various degrees of
*upgrade stability*. A fully upgrade stable encoding format must tolerate
that one model can add fields that an older version of the model should be
capable of ignoring.

Partial upgrade stability can still be useful as is the case of the
*musli-storage* format below, because reading from storage only requires
decoding to be upgrade stable. So if correctly managed with
`#[musli(default)]` this will never result in any readers seeing unknown
fields.

The available formats and their capabilities are:

| | `reorder` | `missing` | `unknown` | `self` |
|-|-|-|-|-|
| [`musli-storage`] `#[musli(packed)]` | ✗ | ✗ | ✗ | ✗ |
| [`musli-storage`]                    | ✔ | ✔ | ✗ | ✗ |
| [`musli-wire`]                       | ✔ | ✔ | ✔ | ✗ |
| [`musli-descriptive`]                | ✔ | ✔ | ✔ | ✔ |

`reorder` determines whether fields must occur in exactly the order in which
they are specified in their type. Reordering fields in such a type would
cause unknown but safe behavior of some kind. This is only suitable for
byte-oriented IPC where the data models of each client are are strictly
synchronized.

`missing` determines if reading can handle missing fields through something
like `Option<T>`. This is suitable for on-disk storage, because it means
that new optional fields can be added as the schema evolves.

`unknown` determines if the format can skip over unknown fields. This is
suitable for network communication. At this point you've reached *upgrade
stability*. Some level of introspection is possible here, because the
serialized format must contain enough information about fields to know what
to skip which usually allows for reasoning about basic types.

`self` determines if the format is self-descriptive. Allowing the structure
of the data to be fully reconstructed from its serialized state. These
formats do not require models to decode, and can be converted to and from
dynamic containers such as [`musli-value`] for introspection.

For every feature you drop, the format becomes more compact and efficient.
`musli-storage` `#[musli(packed)]` for example is roughly as compact as
[`bincode`] while [`musli-wire`] is comparable in size to something like
[`protobuf`].

<br>

## Examples

The following is an example of *full upgrade stability* using [`musli-wire`]:

```rust
use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version1 {
    name: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version2 {
    name: String,
    #[musli(default)]
    age: Option<u32>,
}

let version2 = musli_wire::to_buffer(&Version2 {
    name: String::from("Aristotle"),
    age: Some(62),
})?;

let version1: Version1 = musli_wire::decode(version2.as_slice())?;

assert_eq!(version1, Version1 {
    name: String::from("Aristotle"),
});
```

The following is an example of *partial upgrade stability* using
[`musli-storage`]:

```rust
use musli::{Encode, Decode};

let version2 = musli_storage::to_buffer(&Version2 {
    name: String::from("Aristotle"),
    age: Some(62),
})?;

assert!(musli_storage::decode::<_, Version1>(version2.as_slice()).is_err());

let version1 = musli_storage::to_buffer(&Version1 {
    name: String::from("Aristotle"),
})?;

let version2: Version2 = musli_storage::decode(version1.as_slice())?;

assert_eq!(version2, Version2 {
    name: String::from("Aristotle"),
    age: None,
});
```

<br>

## Modes

In Müsli the same model can be serialized in different ways. Instead of
requiring the use of multiple models, we instead support each model
implementing different *modes*.

A mode allows for different encoding attributes to apply depending on which
mode something is performed in. A mode can apply to *any* musli parameter
giving you a lot of flexibility.

If a mode is not specified, an implementation will apply to all modes (`M:
Mode`), if at least one mode is specified it will be implemented for all
modes which are present in a model and [`DefaultMode`]. This way, an
encoding which uses `DefaultMode` (which it does by default) should always
work.

```rust
use musli::mode::{DefaultMode, Mode};
use musli::{Decode, Encode};
use musli_json::Encoding;

enum Alt {}
impl Mode for Alt {}

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(mode = Alt, packed)]
#[musli(default_field_name = "name")]
struct Word<'a> {
    text: &'a str,
    teineigo: bool,
}

let CONFIG: Encoding<DefaultMode> = Encoding::new();
let ALT_CONFIG: Encoding<Alt> = Encoding::new();

let word = Word {
    text: "あります",
    teineigo: true,
};

let out = CONFIG.to_string(&word)?;
assert_eq!(out, r#"{"text":"あります","teineigo":true}"#);
let word2 = CONFIG.from_str(&out[..])?;
assert_eq!(word, word2);

let out = ALT_CONFIG.to_string(&word)?;
assert_eq!(out, r#"["あります",true]"#);
let word2 = ALT_CONFIG.from_str(&out[..])?;
assert_eq!(word, word2);

```

<br>

## Unsafety

This library currently has two instances of unsafe:

* A `mem::transcode` in `Tag::kind`. Which guarantees that converting into
  the `Kind` enum which is `#[repr(u8)]` is as efficient as possible. (Soon
  to be replaced with an equivalent safe variant).

* A largely unsafe `SliceReader` which provides more efficient reading than
  the default `Reader` impl for `&[u8]` does (which uses split_at). Since it
  can perform most of the necessary comparisons directly on the pointers.

<br>

## Performance

> The following are the results of preliminary benchmarking and should be
> taken with a big grain of 🧂.

The two benchmark suites portrayed are:
* `rt-prim` - which is a small object containing one of each primitive type
  and a string and a byte array.
* `rt-lg` - which is roundtrip encoding of a large object, containing
  vectors and maps of other objects.

<img src="https://raw.githubusercontent.com/udoprog/musli/main/images/rt-lg.png" alt="Roundtrip of a large object">

<img src="https://raw.githubusercontent.com/udoprog/musli/main/images/rt-prim.png" alt="Roundtrip of a small object">

[`bincode`]: https://docs.rs/bincode
[`Decode`]: https://docs.rs/musli/latest/musli/trait.Decode.html
[`DefaultMode`]: https://docs.rs/musli/latest/musli/mode/enum.DefaultMode.html
[`derives`]: https://docs.rs/musli/latest/musli/derives/
[`Encode`]: https://docs.rs/musli/latest/musli/trait.Encode.html
[`musli-descriptive`]: https://docs.rs/musli-descriptive
[`musli-storage`]: https://docs.rs/musli-storage
[`musli-tests`]: https://github.com/udoprog/musli/tree/main/crates/musli-tests
[`musli-value`]: https://docs.rs/musli-value
[`musli-wire`]: https://docs.rs/musli-wire
[`protobuf`]: https://developers.google.com/protocol-buffers
[`serde`]: https://serde.rs
