# musli

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Excellent performance, no compromises[^1]!

Müsli is a flexible, fast, and generic binary serialization framework for
Rust, in the same vein as [`serde`].

It provides a set of [formats](#formats), each with its own well-documented
set of features and tradeoffs. Every byte-oriented serialization method
(including [`musli-json`]) has full `#[no_std]` support with or without
`alloc`. And a particularly neat component providing low-level refreshingly
simple [zero-copy serialization].

[^1]: As in Müsli should be able to do everything you need and more.

<br>

## Quick guide

* For information on how to implement [`Encode`] and [`Decode`], see
  [`derives`].
* For information on how this library is tested, see [`tests`].
* For [performance] and [size comparisons].

<br>

## Usage

Add the following to your `Cargo.toml` using the [format](#formats) you want
to use:

```toml
musli = "0.0.104"
musli-wire = "0.0.104"
```

<br>

## Design

The heavy lifting in user code is done through the [`Encode`] and [`Decode`]
derives which are thoroughly documented in the [`derives`] module. Müsli
primarily operates based on the schema types which implement these traits
imply, but self-descriptive formats are also possible (see
[`Formats`](#formats) below).

```rust
use musli::{Encode, Decode};

#[derive(Encode, Decode)]
struct Person {
    /* .. fields .. */
}
```

> **Note** by default a field is identified by its *numerical index* which
> would change if they are re-ordered. Renaming fields and setting a default
> naming policy can be done by configuring the [`derives`].

The binary serialization formats provided aim to efficiently and accurately
encode every type and data structure available in Rust. Each format comes
with [well-documented tradeoffs](#formats) and aim to be fully memory safe
to use.

Internally we use the terms "encoding", "encode", and "decode" because it's
distinct from [`serde`]'s use of "serialization", "serialize", and
"deserialize" allowing for the ease of using both libraries side by side if
desired.

Müsli is designed on similar principles as [`serde`]. Relying on Rust's
powerful trait system to generate code which can largely be optimized away.
The end result should be very similar to handwritten highly optimized code.

As an example of this, these two functions both produce the same assembly on
my machine (built with `--release`):

```rust
const OPTIONS: Options = options::new()
    .with_integer(Integer::Fixed)
    .with_byte_order(ByteOrder::NATIVE)
    .build();

const ENCODING: Encoding<DefaultMode, OPTIONS> = Encoding::new().with_options();

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

Where Müsli differs in design philosophy is twofold:

We make use of GATs to provide tighter abstractions, which should be easier
for Rust to optimize.

We make less use of the Visitor pattern in certain instances where it's
deemed unnecessary, such as [when decoding collections]. The result is
usually cleaner decode implementations, as shown here:

```rust
use musli::Context;
use musli::de::{Decode, Decoder, SequenceDecoder};

struct MyType {
    data: Vec<String>,
}

impl<'de, M> Decode<'de, M> for MyType {
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        decoder.decode_sequence(|seq| {
            let mut data = Vec::with_capacity(seq.size_hint().or_default());

            while let Some(decoder) = seq.decode_next()? {
                data.push(decoder.decode()?);
            }

            Ok(Self { data })
        })
    }
}
```

Another major aspect where Müsli differs is in the concept of
[modes](#modes) (note the `M` parameter above). Since this is a parameter of
the `Encode` and `Decode` traits it allows for the same data model to be
serialized in many different ways. This is a larger topic and is covered
further down.

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
| [`musli-json`][`musli-json`][^json]  | ✔ | ✔ | ✔ | ✔ |

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
[`musli-storage`] using `#[musli(packed)]` for example is roughly as compact
as [`bincode`] while [`musli-wire`] is comparable in size to something like
[`protobuf`]. All formats are primarily byte-oriented, but some might
perform [bit packing] if the benefits are obvious.

[^json]: This is strictly not a binary serialization, but it was implemented
as a litmus test to ensure that Müsli has the necessary framework features
to support it. Luckily, the implementation is also quite good!

<br>

## Upgrade stability

The following is an example of *full upgrade stability* using
[`musli-wire`]. Note how `Version1` can be decoded from an instance of
`Version2` because it understands how to skip fields which are part of
`Version2`. We're also explicitly `#[musli(rename = ..)]` the fields to
ensure that they don't change in case they are re-ordered.

```rust
use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version1 {
    #[musli(rename = 0)]
    name: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version2 {
    #[musli(rename = 0)]
    name: String,
    #[musli(default, rename = 1)]
    age: Option<u32>,
}

let version2 = musli_wire::to_vec(&Version2 {
    name: String::from("Aristotle"),
    age: Some(62),
})?;

let version1: Version1 = musli_wire::decode(version2.as_slice())?;
```

The following is an example of *partial upgrade stability* using
[`musli-storage`] on the same data models. Note how `Version2` can be
decoded from `Version1` but *not* the other way around. That's why it's
suitable for on-disk storage the schema can evolve from older to newer
versions.

```rust
let version2 = musli_storage::to_vec(&Version2 {
    name: String::from("Aristotle"),
    age: Some(62),
})?;

assert!(musli_storage::decode::<_, Version1>(version2.as_slice()).is_err());

let version1 = musli_storage::to_vec(&Version1 {
    name: String::from("Aristotle"),
})?;

let version2: Version2 = musli_storage::decode(version1.as_slice())?;
```

<br>

## Modes

In Müsli the same model can be serialized in different ways. Instead of
requiring the use of distinct models we support implementing different
*modes* for a single model.

A mode allows for different encoding attributes to apply depending on which
mode an encoder is configured to use. A mode can apply to *any* musli
parameter giving you a lot of flexibility.

If a mode is not specified, an implementation will apply to all modes (`M:
Mode`), if at least one mode is specified it will be implemented for all
modes which are present in a model and [`DefaultMode`]. This way, an
encoding which uses `DefaultMode` (which it does by default) should always
work.

For more information on how to configure modes, see the [`derives`] module.
Below is a simple example of how we can use two modes to provide two
different kinds of serialization to a single struct.

```rust
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_json::Encoding;

enum Alt {}

#[derive(Decode, Encode)]
#[musli(mode = Alt, packed)]
#[musli(default_field = "name")]
struct Word<'a> {
    text: &'a str,
    teineigo: bool,
}

const CONFIG: Encoding<DefaultMode> = Encoding::new();
const ALT_CONFIG: Encoding<Alt> = Encoding::new().with_mode();

let word = Word {
    text: "あります",
    teineigo: true,
};

let out = CONFIG.to_string(&word)?;
assert_eq!(out, r#"{"text":"あります","teineigo":true}"#);

let out = ALT_CONFIG.to_string(&word)?;
assert_eq!(out, r#"["あります",true]"#);
```

<br>

## Unsafety

This is a non-exhaustive list of unsafe use in this crate, and why they are
used:

* A `mem::transcode` in `Tag::kind`. Which guarantees that converting into
  the `Kind` enum which is `#[repr(u8)]` is as efficient as possible.

* A largely unsafe `SliceReader` which provides more efficient reading than
  the default `Reader` impl for `&[u8]` does. Since it can perform most of
  the necessary comparisons directly on the pointers.

* Some unsafety related to UTF-8 handling in `musli_json`, because we check
  UTF-8 validity internally ourselves (like `serde_json`).

* `FixedBytes<N>` is a stack-based container that can operate over
  uninitialized data. Its implementation is largely unsafe. With it
  stack-based serialization can be performed which is useful in no-std
  environments.

* Some unsafe is used for owned `String` decoding in all binary formats to
  support faster string processing using [`simdutf8`]. Disabling the
  `simdutf8` feature (enabled by default) removes the use of this unsafe.

To ensure this library is correctly implemented with regards to memory
safety, extensive testing is performed using `miri`. See [`tests`] for more
information on this.

<br>

[performance]: https://udoprog.github.io/musli/benchmarks/
[size comparisons]: https://udoprog.github.io/musli/benchmarks/#size-comparisons
[`bincode`]: https://docs.rs/bincode
[`Decode`]: https://docs.rs/musli/latest/musli/de/trait.Decode.html
[`DefaultMode`]: https://docs.rs/musli/latest/musli/mode/enum.DefaultMode.html
[`derives`]: https://docs.rs/musli/latest/musli/derives/
[`Encode`]: https://docs.rs/musli/latest/musli/en/trait.Encode.html
[`musli-descriptive`]: https://docs.rs/musli-descriptive
[`musli-json`]: https://docs.rs/musli-json
[`musli-storage`]: https://docs.rs/musli-storage
[`tests`]: https://github.com/udoprog/musli/tree/main/crates/tests
[`musli-value`]: https://docs.rs/musli-value
[`musli-wire`]: https://docs.rs/musli-wire
[`protobuf`]: https://developers.google.com/protocol-buffers
[`serde`]: https://serde.rs
[`simdutf8`]: https://docs.rs/simdutf8
[bit packing]: https://github.com/udoprog/musli/blob/main/crates/musli-descriptive/src/tag.rs
[when decoding collections]:
https://docs.rs/serde/latest/serde/trait.Deserializer.html#tymethod.deserialize_seq
[zero-copy serialization]: https://docs.rs/musli-zerocopy
