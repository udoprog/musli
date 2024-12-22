# musli-zerocopy

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Refreshingly simple, blazingly fast zero copy primitives by Müsli.

This provides a basic set of tools to deal with types which do not require
copying during deserialization. You define the `T`, and we provide the safe
`&[u8]` <-> `&T` conversions.

Reading a zero-copy structure has full `#[no_std]` support. Constructing
ones currently requires the `alloc` feature to be enabled.

```rust
use musli_zerocopy::{buf, Ref, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Person {
    age: u8,
    name: Ref<str>,
}

let buf = buf::aligned_buf::<Person>(include_bytes!("author.bin"));
let person = Person::from_bytes(&buf[..])?;

assert_eq!(person.age, 35);
// References are incrementally validated.
assert_eq!(buf.load(person.name)?, "John-John");
```

For a detailed overview of how this works, see the [`ZeroCopy`] and its
corresponding [`ZeroCopy`][derive] derive. There's also a high level guide
just below.

This crate also includes a couple of neat high level data structures you
might be interested in:
* [`phf`] provides maps and sets based on [`phf` crate], or perfect hash
  functions.
* [`swiss`] is a port of the [`hashbrown` crate] which is a Google
  SwissTable implementation.
* [`trie`] is an implementation of a prefix-trie, which supports efficient
  multi-value byte-prefixed lookups.

Finally if you're interested in the performance of `musli-zerocopy` you
should go to [`benchmarks`]. I will be extending this suite with more
zero-copy types, but for now we have a clear lead in the use cases I've
tested it for.

This is because:
* Zero-copy doesn't incur a deserialization overhead if done correctly. You
  take bytes in one place, validate them, and treat them as the destination
  type. There are only so many ways this can be done;
* Padding has been implemented and optimized in such a way that it mostly
  generates the equivalent code you'd write by hand, and;
* Incremental validation means that you only need to pay for what you're
  accessing. So for random access we only need to validate the parts that
  are being accessed.

Overview:
* [Why should I consider `musli-zerocopy` over X?](#why-should-i-consider-musli-zerocopy-over-x)
* [Guide](#guide)
* [Reading data](#reading-data)
* [Writing data at offset zero](#writing-data-at-offset-zero)
* [Portability](#portability)
* [Limits](#limits)

<br>

## Why should I consider `musli-zerocopy` over X?

Since this is the first question anyone will ask, here is how we differ from
other popular libraries:
* [`zerocopy`](https://docs.rs/zerocopy) doesn't support padded
  types[^padded], bytes to reference conversions, or conversions which does
  not permit decoding types unless all bit patterns can be inhabited by
  zeroes[^zeroes]. We still want to provide more of a complete toolkit that
  you'd need to build and interact with complex data structures like we get
  through the [`phf`] and [`swiss`] modules. This crate might indeed at some
  point make use of `zerocopy`'s traits.
* [`rkyv`](https://docs.rs/rkyv) operates on `#[repr(Rust)]` types and from
  this derives an optimized `Archived` variation for you. This library lets
  you build the equivalent of the  `Archived` variant directly and the way
  you interact with the data model doesn't incur the cost of validation up
  front. With `rkyv` it took my computer 100% of a CPU core and about half a
  second to load 12 million dictionary entries[^dictionary], which is a cost
  that is simply not incurred by incrementally validating. Not validating is
  not an option since that would be wildly unsound - your application would
  be vulnerable to malicious dictionary files.

[^padded]: This is on zerocopy's roadmap, but it fundamentally doesn't play
    well with the central `FromBytes` / `ToBytes` pair of traits

[^zeroes]: [FromBytes extends FromZeros](https://docs.rs/zerocopy/latest/zerocopy/trait.FromBytes.html)

[^dictionary]: <https://github.com/udoprog/jpv/blob/main/crates/lib/src/database.rs>

<br>

## Guide

Zero-copy in this library refers to the act of interacting with data
structures that reside directly in `&[u8]` memory without the need to first
decode them.

Conceptually it works a bit like this.

Say you want to store the string `"Hello World!"`.

```rust
use musli_zerocopy::OwnedBuf;

let mut buf = OwnedBuf::new();
let string = buf.store_unsized("Hello World!");
let reference = buf.store(&string);

assert_eq!(reference.offset(), 12);
```

This would result in the following buffer:

```text
0000: "Hello World!"
// Might get padded to ensure that the size is aligned by 4 bytes.
0012: offset -> 0000
0016: size -> 12
```

What we see at offset `0016` is an 8 byte [`Ref<str>`]. The first field
stores the offset where to fetch the string, and the second field the length
of the string.

Let's have a look at a [`Ref<[u32]>`][ref-u32] next:

```rust
use musli_zerocopy::{Ref, OwnedBuf};

let mut buf = OwnedBuf::new();
let slice: Ref<[u32]> = buf.store_slice(&[1, 2, 3, 4]);
let reference = buf.store(&slice);

assert_eq!(reference.offset(), 16);
```

This would result in the following buffer:

```text
0000: u32 -> 1
0004: u32 -> 2
0008: u32 -> 3
0012: u32 -> 4
0016: offset -> 0000
0020: length -> 4
```

At address `0016` we store two fields which corresponds to a
[`Ref<[u32]>`][ref-u32].

Next lets investigate an example using a `Custom` struct:

```rust
use core::mem::size_of;
use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: Ref<str>,
}

let mut buf = OwnedBuf::new();

let string = buf.store_unsized("Hello World!");
let custom = buf.store(&Custom { field: 42, string });

// The buffer stores both the unsized string and the Custom element.
assert!(buf.len() >= 24);
// We assert that the produced alignment is smaller or equal to 8
// since we'll be relying on this below.
assert!(buf.requested() <= 8);
```

This would result in the following buffer:

```text
0000: "Hello World!"
0012: u32 -> 42
0016: offset -> 0000
0020: size -> 12
```

Our struct starts at address `0012`, first we have the `u32` field, and
immediately after that we have the string.

<br>

## Reading data

Later when we want to use the type, we take the buffer we've generated and
include it somewhere else.

There's a few pieces of data (lets call it DNA) we need to have to read a
type back from a raw buffer:
* The *alignment* of the buffer. Which you can read through the
  [`requested()`]. On the receiving end we need to ensure that the buffer
  follow this alignment. Dynamically this can be achieved by loading the
  buffer using [`aligned_buf(bytes, align)`]. Other tricks include embedding
  a static buffer inside of an aligned newtype which we'll showcase below.
  Networked applications might simply agree to use a particular alignment up
  front. This alignment has to be compatible with the types being coerced.
* The *endianness* of the machine which produced the buffer. Any numerical
  elements will in native endian ordering, so they would have to be adjusted
  on the read side if it differ.
* The type definition which is being read which implements [`ZeroCopy`].
  This is `Custom` above. The [`ZeroCopy`] derive ensures that we can safely
  coerce a buffer into a reference of the type. The data might at worst be
  garbled, but we can never do anything unsound while using safe APIs.
* The offset at where the [`ZeroCopy`] structure is read. To read a
  structure we combine a pointer and a type into a [`Ref`] instance.

If the goal is to both produce and read the buffer on the same system
certain assumptions can be made. And if those assumptions turn out to be
wrong the worst outcome will only ever be an error as long as you're using
the safe APIs or abide by the safety documentation of the unsafe APIs.

> **Info** A note on sending data over the network. This is perfectly doable
> as long as you include the alignment of the buffer and the endianness of
> the data structure. Both of these can be retrieved:
>
> ```
> # use musli_zerocopy::OwnedBuf;
> let buf = OwnedBuf::new();
>
> /* write something */
>
> let is_little_endian = cfg!(target_endian = "little");
> let alignment = buf.requested();
> ```

The following is an example of reading the type directly out of a newtype
aligned `&'static [u8]` buffer:

```rust
use core::mem::size_of;
use musli_zerocopy::Buf;

// Helper to force the static buffer to be aligned like `A`.
#[repr(C)]
struct Align<A, T: ?Sized>([A; 0], T);

static BYTES: &Align<u64, [u8]> = &Align([], *include_bytes!("custom.bin"));

let buf = Buf::new(&BYTES.1);

// Construct a pointer into the buffer.
let custom = Ref::<Custom>::new(BYTES.1.len() - size_of::<Custom>());

let custom: &Custom = buf.load(custom)?;
assert_eq!(custom.field, 42);
assert_eq!(buf.load(custom.string)?, "Hello World!");
```

<br>

## Writing data at offset zero

Most of the time you want to write data where the first element in the
buffer is the element currently being written.

This is useful because it satisfies the last requirement above, *the offset*
at where the struct can be read will then simply be zero, and all the data
it depends on are stored at subsequent offsets.

```rust
use musli_zerocopy::OwnedBuf;
use musli_zerocopy::mem::MaybeUninit;

let mut buf = OwnedBuf::new();
let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();

let string = buf.store_unsized("Hello World!");

buf.load_uninit_mut(reference).write(&Custom { field: 42, string });

let reference = reference.assume_init();
assert_eq!(reference.offset(), 0);
```

<br>

## Portability

By default archives will use the native [`ByteOrder`]. In order to construct
and load a portable archive, the byte order in use has to be explicitly
specified.

This is done by specifying the byte order in use during buffer construction
and expliclty setting the `E` parameter in types which received it such as
[`Ref<T, E, O>`].

We can start of by defining a fully `Portable` archive structure, which
received both size and [`ByteOrder`]. Note that it could also just
explicitly specify a desired byte order but doing it like this makes it
maximally flexible as an example:

```rust
use musli_zerocopy::{Size, ByteOrder, Ref, Endian, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Archive<E, O>
where
    E: ByteOrder,
    O: Size
{
    string: Ref<str, E, O>,
    number: Endian<u32, E>,
}
```

Building a buffer out of the structure is fairly straight forward,
[`OwnedBuf`] has the [`with_byte_order::<E>()`] method which allows us to
specify a "sticky" [`ByteOrder`] to use in types which interact with it
during construction:

```rust
use musli_zerocopy::{endian, Endian, OwnedBuf};
let mut buf = OwnedBuf::new()
    .with_byte_order::<endian::Little>();

let first = buf.store(&Endian::le(42u16));
let portable = Archive {
    string: buf.store_unsized("Hello World!"),
    number: Endian::new(10),
};
let portable = buf.store(&portable);

assert_eq!(&buf[..], &[
    42, 0, // 42u16
    72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33, // "Hello World!"
    0, 0, // padding
    2, 0, 0, 0, 12, 0, 0, 0, // Ref<str>
    10, 0, 0, 0 // 10u32
]);

let portable = buf.load(portable)?;

let mut buf = OwnedBuf::new()
    .with_byte_order::<endian::Big>();

let first = buf.store(&Endian::be(42u16));
let portable = Archive {
    string: buf.store_unsized("Hello World!"),
    number: Endian::new(10),
};
let portable = buf.store(&portable);

assert_eq!(&buf[..], &[
    0, 42, // 42u16
    72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33, // "Hello World!"
    0, 0, // padding
    0, 0, 0, 2, 0, 0, 0, 12, // Ref<str>
    0, 0, 0, 10 // 10u32
]);

let portable = buf.load(portable)?;
```

<br>

## Limits

Offset, the size of unsized values, and slice lengths are all limited to
32-bit. The system you're using must have a `usize` type which is at least
32-bits wide. This is done to save space by default.

The pointer width on the system is checked at compile time, while trying to
use an offset or a size larger than `2^32` will result in a panic.

Example of using an address larger than `2^32` causing a panic:

```rust
Ref::<Custom>::new(1usize << 32);
```

Example panic using a [`Ref<\[T\]>`] with a length larger than `2^32`:

```rust
Ref::<[Custom]>::with_metadata(0, 1usize << 32);
```

Example panic using an [`Ref<str>`] value with a size larger than `2^32`:

```rust
Ref::<str>::with_metadata(0, 1usize << 32);
```

If you want to address data larger than this limit, it is recommended that
you partition your dataset into 32-bit addressable chunks.

If you really want to change this limit, you can modify it by setting the
default `O` parameter on the various [`Size`]-dependent types:

The available [`Size`] implementations are:
* `u32` for 32-bit sized pointers (the default).
* `usize` for target-dependently sized pointers.

```rust
// These no longer panic:
let reference = Ref::<Custom, Native, usize>::new(1usize << 32);
let slice = Ref::<[Custom], Native, usize>::with_metadata(0, 1usize << 32);
let unsize = Ref::<str, Native, usize>::with_metadata(0, 1usize << 32);
```

To initialize an [`OwnedBuf`] with a custom [`Size`], you can use
[`OwnedBuf::with_size`]:

```rust
use musli_zerocopy::OwnedBuf;
use musli_zerocopy::buf::DefaultAlignment;

let mut buf = OwnedBuf::with_capacity_and_alignment::<DefaultAlignment>(0)
    .with_size::<usize>();
```

The [`Size`] you've specified during construction of an [`OwnedBuf`] will
then carry into any pointers it return:

```rust
use musli_zerocopy::{DefaultAlignment, OwnedBuf, Ref, ZeroCopy};
use musli_zerocopy::endian::Native;

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    reference: Ref<u32, Native, usize>,
    slice: Ref::<[u32], Native, usize>,
    unsize: Ref::<str, Native, usize>,
}

let mut buf = OwnedBuf::with_capacity(0)
    .with_size::<usize>();

let reference = buf.store(&42u32);
let slice = buf.store_slice(&[1, 2, 3, 4]);
let unsize = buf.store_unsized("Hello World");

buf.store(&Custom { reference, slice, unsize });
```

<br>

[`aligned_buf(bytes, align)`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/trait.Size.html
[`benchmarks`]: https://udoprog.github.io/musli/benchmarks/
[`ByteOrder`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/trait.ByteOrder.html
[`hashbrown` crate]: https://docs.rs/phf
[`OwnedBuf::with_size`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/buf/struct.OwnedBuf.html#method.with_size
[`OwnedBuf`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/buf/struct.OwnedBuf.html
[`phf` crate]: https://docs.rs/phf
[`phf`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/phf/index.html
[`Ref`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Ref.html
[`Ref<str>`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Ref.html
[`Ref<T, E, O>`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Ref.html
[`requested()`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.OwnedBuf.html#method.requested
[`Size`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/trait.Size.html
[`swiss`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/swiss/index.html
[`trie`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/trie/index.html
[`with_byte_order::<E>()`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/buf/struct.OwnedBuf.html#method.with_byte_order
[`ZeroCopy`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/trait.ZeroCopy.html
[derive]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/derive.ZeroCopy.html
[ref-u32]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Ref.html
