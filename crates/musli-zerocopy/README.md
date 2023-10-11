# musli-zerocopy

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Refreshingly simple zero copy primitives provided by Müsli.

This provides a base set of tools to deal with types which do not require
copying during deserialization.

To implement zero-copy support for a Rust type, see the [`ZeroCopy`] derive.

<br>

## Guide

Zero-copy in this library refers to the act of interacting with data
structures that reside directly in `&[u8]` memory without the need to first
decode them.

Conceptually it works a bit like this.

Say you want to store the string `"Hello World!"`.

```rust
use musli_zerocopy::AlignedBuf;

let mut buf = AlignedBuf::new();
let string = buf.store_unsized("Hello World!")?;
let reference = buf.store(&string)?;

assert_eq!(reference.offset(), 12);
```

This would result in the following buffer:

```text
0000: "Hello World!"
// Might get padded to ensure that the size is aligned by 4 bytes.
0012: offset -> 0000
0016: size -> 12
```

What we see at offset `0016` is an 8 byte [`Unsized<str>`]. The first field
stores the offset where to fetch the string, and the second field the length
of the string.

Let's have a look at a [`Slice<u32>`] next:

```rust
use musli_zerocopy::AlignedBuf;

let mut buf = AlignedBuf::new();
let slice = buf.store_slice(&[1u32, 2, 3, 4])?;
let reference = buf.store(&slice)?;

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

At address `0016` we store two fields which corresponds to a [`Slice<u32>`].

Next lets investigate an example using a `Custom` struct:

```rust
use core::mem::size_of;
use musli_zerocopy::{AlignedBuf, ZeroCopy};
use musli_zerocopy::pointer::Unsized;

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: Unsized<str>,
}

let mut buf = AlignedBuf::new();

let string = buf.store_unsized("Hello World!")?;
let custom = buf.store(&Custom { field: 42, string })?;

// The buffer stores both the unsized string and the Custom element.
assert!(buf.len() >= 24);
// We assert that the produced alignment is smaller or equal to 8
// since we'll be relying on this below.
assert!(buf.requested() <= 8);
```

This would result in the following buffer:

```rust
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
  buffer back into an appropriately constructed [`AlignedBuf`] instance.
  Other tricks include embedding a static buffer inside of an aligned
  newtype which we'll showcase below. Networked applications might simply
  agree to use a particular alignment up front. This alignment has to be
  compatible with the types being coerced.
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
> ```no_run
> use musli_zerocopy::AlignedBuf;
> let buf: AlignedBuf = todo!();
>
> let is_little_endian = cfg!(target_endian = "little");
> let alignment = buf.requested();
> ```

The following is an example of reading the type directly out of a newtype
aligned `&'static [u8]` buffer:

```rust
use core::mem::size_of;
use musli_zerocopy::Buf;
use musli_zerocopy::pointer::Ref;

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

Example panic using a [`Slice`] with a length larger than `2^32`:

```rust
Slice::<Custom>::new(0, 1usize << 32);
```

Example panic using an [`Unsized`] value with a size larger than `2^32`:

```rust
Unsized::<str>::new(0, 1usize << 32);
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
let reference = Ref::<Custom, usize>::new(1usize << 32);
let slice = Slice::<Custom, usize>::new(0, 1usize << 32);
let unsize = Unsized::<str, usize>::new(0, 1usize << 32);
```

[`AlignedBuf`] can also be initialized with a custom [`Size`]:

To initialize an [`AlignedBuf`] with a custom [`Size`] you simply use this
constructor while specifying one of the above parameters:

```rust
use musli_zerocopy::AlignedBuf;
use musli_zerocopy::buf::DEFAULT_ALIGNMENT;

let mut buf = AlignedBuf::<usize>::with_capacity_and_alignment(0, DEFAULT_ALIGNMENT);
```

And to use a custom target size in a struct using the [`ZeroCopy`], you
simply specify the default parameter:

```rust
use musli_zerocopy::{ZeroCopy, AlignedBuf};
use musli_zerocopy::buf::DEFAULT_ALIGNMENT;
use musli_zerocopy::pointer::{Ref, Slice, Unsized};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    reference: Ref<u32, usize>,
    slice: Slice::<u32, usize>,
    unsize: Unsized::<str, usize>,
}

let mut buf = AlignedBuf::with_capacity_and_alignment(0, DEFAULT_ALIGNMENT);

let reference = buf.store(&42u32)?;
let slice = buf.store_slice(&[1, 2, 3, 4])?;
let unsize = buf.store_unsized("Hello World")?;

buf.store(&Custom { reference, slice, unsize })?;
```

[`requested()`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.AlignedBuf.html#method.requested
[`ZeroCopy`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/derive.ZeroCopy.html
[`Ref`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Ref.html
[`Slice`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Slice.html
[`Slice<u32>`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Slice.html
[`Unsized`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Unsized.html
[`Unsized<str>`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Unsized.html
[`AlignedBuf`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/buf/struct.AlignedBuf.html
[`Size`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/trait.Size.html
