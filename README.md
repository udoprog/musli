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
simple [zero-copy serialization][zerocopy].

[^1]: As in Müsli should be able to do everything you need and more.

<br>

## Overview

* See [`derives`] to learn how to implement [`Encode`] and [`Decode`].
* See [benchmarks] and [size comparisons] to learn about the performance of
  this framework.
* See [`tests`] to learn how this library is tested.
* See [`musli-serde`] for seamless compatibility with [`serde`]. You might
  also be interested to learn how [Müsli is different][different].

[different]: #müsli-is-different-from-serde

<br>

## Usage

Add the following to your `Cargo.toml` using the [format](#formats) you want
to use:

```toml
[dependencies]
musli = "0.0.112"
musli-wire = "0.0.112"
```

<br>

## Design

The heavy lifting is done by the [`Encode`] and [`Decode`] derives which are
documented in the [`derives`] module.

Müsli operates based on the schema represented by the types which implement
these traits.

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
with [well-documented tradeoffs](#formats) and aims to be fully memory safe
to use.

Internally we use the terms "encoding", "encode", and "decode" because it's
distinct from [`serde`]'s use of "serialization", "serialize", and
"deserialize" allowing for the clearer interoperability between the two
libraries. Encoding and decoding also has more of a "binary serialization"
vibe, which more closely reflects the focus of this framework.

Müsli is designed on similar principles as [`serde`]. Relying on Rust's
powerful trait system to generate code which can largely be optimized away.
The end result should be very similar to handwritten, highly optimized code.

As an example of this, these two functions both produce the same assembly
(built with `--release`):

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

<br>

## Müsli is different from [`serde`]

* We make use of GATs to provide tighter abstractions. GATs were not
  available when serde was designed.
* When decoding or encoding we operate by the principle that most things
  return either return a [`Decoder`] or [`Encoder`]. This means for example
  that field names are not restricted to be strings or indexes, but can be
  renamed to [completely arbitrary types][musli-name-type].
* We make less use of the Visitor pattern in certain instances where it's
  deemed unnecessary, such as [when decoding collections]. The result is
  usually cleaner decode implementations like below.
* We make use of [*moded encoding*](#Modes) allowing the same struct to be
  encoded in many different ways.
* We support [detailed tracing] when decoding for rich diagnostics.
* Müsli was designed to support [no-std and no-alloc] environments from the
  ground up without compromising on features using a safe and efficient
  [scoped allocations].

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
communication where the data models of each client are strictly
synchronized.

`missing` determines if reading can handle missing fields through something
like `Option<T>`. This is suitable for on-disk storage, because it means
that new optional fields can be added as the schema evolves.

`unknown` determines if the format can skip over unknown fields. This is
suitable for network communication. At this point you've reached [*upgrade
stability*](#upgrade-stability). Some level of introspection is possible
here, because the serialized format must contain enough information about
fields to know what to skip which usually allows for reasoning about basic
types.

`self` determines if the format is self-descriptive. Allowing the structure
of the data to be fully reconstructed from its serialized state. These
formats do not require models to decode and can be converted to and from
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
[`musli-wire`]. `Version1` can be decoded from an instance of `Version2`
because it understands how to skip fields which are part of `Version2`.
We're also explicitly adding `#[musli(name = ..)]` to the fields to ensure
that they don't change in case they are re-ordered.

```rust
use musli::{Encode, Decode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version1 {
    #[musli(name = 0)]
    name: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version2 {
    #[musli(name = 0)]
    name: String,
    #[musli(default, name = 1)]
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
decoded from `Version1` but *not* the other way around making it suitable
for on-disk storage where the schema can evolve from older to newer
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

In Müsli in contrast to [`serde`] the same model can be serialized in
different ways. Instead of requiring the use of distinct models we support
implementing different *modes* for a single model.

A mode is a type parameter, which allows for different attributes to apply
depending on which mode an encoder is configured to use. A mode can apply to
*any* musli attributes giving you a lot of flexibility.

If a mode is not specified, an implementation will apply to all modes (`M`),
if at least one mode is specified it will be implemented for all modes which
are present in a model and [`DefaultMode`]. This way, an encoding which uses
`DefaultMode` which is the default mode should always work.

For more information on how to configure modes, see [`derives`].

Below is a simple example of how we can use two modes to provide two
completely different formats using a single struct:

```rust
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_json::Encoding;

enum Alt {}

#[derive(Decode, Encode)]
#[musli(mode = Alt, packed)]
#[musli(name_all = "name")]
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

* A `mem::transmute` in `Tag::kind`. Which guarantees that converting into
  the `Kind` enum which is `#[repr(u8)]` is as efficient as possible.

* A largely unsafe `SliceReader` which provides more efficient reading than
  the default `Reader` impl for `&[u8]` does. Since it can perform most of
  the necessary comparisons directly on the pointers.

* Some unsafety related to UTF-8 handling in `musli_json`, because we check
  UTF-8 validity internally ourselves (like `serde_json`).

* `FixedBytes<N>`, which is a stack-based container that can operate over
  uninitialized data. Its implementation is largely unsafe. With it
  stack-based serialization can be performed which is useful in no-std
  environments.

* Some `unsafe` is used for owned `String` decoding in all binary formats to
  support faster string processing through [`simdutf8`]. Disabling the
  `simdutf8` feature (enabled by default) removes the use of this unsafe.

To ensure this library is correctly implemented with regards to memory
safety, extensive testing and fuzzing is performed using `miri`. See
[`tests`] for more information.

<br>

[`bincode`]: https://docs.rs/bincode
[`Decode`]: https://docs.rs/musli/latest/musli/de/trait.Decode.html
[`Decoder`]: https://docs.rs/musli/latest/musli/trait.Decoder.html
[`DefaultMode`]: https://docs.rs/musli/latest/musli/mode/enum.DefaultMode.html
[`derives`]: https://docs.rs/musli/latest/musli/derives/
[`Encode`]: https://docs.rs/musli/latest/musli/en/trait.Encode.html
[`Encoder`]: https://docs.rs/musli/latest/musli/trait.Encoder.html
[`musli-descriptive`]: https://docs.rs/musli-descriptive
[`musli-json`]: https://docs.rs/musli-json
[`musli-serde`]: https://docs.rs/musli-serde
[`musli-storage`]: https://docs.rs/musli-storage
[`musli-value`]: https://docs.rs/musli-value
[`musli-wire`]: https://docs.rs/musli-wire
[`protobuf`]: https://developers.google.com/protocol-buffers
[`serde`]: https://serde.rs
[`simdutf8`]: https://docs.rs/simdutf8
[`tests`]: https://github.com/udoprog/musli/tree/main/tests
[benchmarks]: https://udoprog.github.io/musli/benchmarks/
[bit packing]: https://github.com/udoprog/musli/blob/main/crates/musli-descriptive/src/tag.rs
[detailed tracing]: https://udoprog.github.io/rust/2023-05-22/abductive-diagnostics-for-musli.html
[musli-name-type]: https://docs.rs/musli/latest/musli/derives/index.html#musliname_type--
[no-std and no-alloc]: https://github.com/udoprog/musli/blob/main/no-std/examples/no-std-json.rs
[scoped allocations]: https://docs.rs/musli-allocator
[size comparisons]: https://udoprog.github.io/musli/benchmarks/#size-comparisons
[when decoding collections]: https://docs.rs/serde/latest/serde/trait.Deserializer.html#tymethod.deserialize_seq
[zerocopy]: https://docs.rs/musli-zerocopy
