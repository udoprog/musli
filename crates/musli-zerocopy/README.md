# musli-zerocopy

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Zero copy primitives for use in Müsli.

This provides a base set of tools to deal with types which do not require
copying during deserialization.

To implement zero-copy support for a Rust type, see the [`ZeroCopy`] derive.

<br>

## Guide

```rust
use core::mem::size_of;
use musli_zerocopy::{AlignedBuf, Pair, Unsized, ZeroCopy};

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

Later when we want to use the type, we take the buffer we've generated and
include it somewhere else.

There's a few pieces of data (called DNA) we need to have to read a type
back from a raw buffer:
* The type being read which implements [`ZeroCopy`]. This is `Custom` above.
  The [`ZeroCopy`] derive ensures that we can safely coerce a buffer into a
  reference of the type.
* The alignment of the buffer, which you can access through the
  [`requested()`]. On the receiving end we need to ensure that the buffer
  follow this alignment. Dynamically this can be achieved by loading the
  buffer back into an appropriately constructed [`AlignedBuf`] instance.
  Other tricks include embedding a static buffer inside of an aligned
  newtype which we'll showcase below.
* The [`Offset`] at where the [`ZeroCopy`] structure is read. To read a
  structure we combine a pointer and a type into the [`Ref`] type.
* The endianness of the machine which produced the buffer. Any numerical
  elements will have been encoded in native endian ordering, so they would
  have to be adjusted on the receiving side if it differs.

If the goal is to both produce and read the buffer on the same system
certain assumptions can be made. But even if those assumptions are wrong,
the worst outcome will only ever be an error as long as you're using the
safe APIs or abide by the safety documentation of the unsafe APIs.

The following is an example of reading the type directly out of a newtype
aligned `&'static [u8]` buffer:

```rust
use core::mem::size_of;
use musli_zerocopy::{Ref, Offset, Buf};

// Helper to force the static buffer to be aligned like `A`.
#[repr(C)]
struct Align<A, T: ?Sized>([A; 0], T);

static BYTES: &Align<u64, [u8]> = &Align([], *include_bytes!("custom.bin"));

let buf = Buf::new(&BYTES.1);

// Construct a pointer into the buffer.
let custom = Ref::new(Offset::<u32>::new(BYTES.1.len() - size_of::<Custom>()));

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

Example of using an [`Offset`] larger than `2^32` causing a panic:

```rust
Offset::<u32>::new(1usize << 32);
```

Example panic using a [`Slice`] with a length larger than `2^32`:

```rust
Slice::<u32>::new(Offset::ZERO, 1usize << 32);
```

Example panic using an [`Unsized`] value with a size larger than `2^32`:

```rust
Unsized::<str>::new(Offset::ZERO, 1usize << 32);
```

If you want to address data larger than this limit, it is recommended that
you partition your dataset into 32-bit addressable chunks.

If you really want to change this limit, you can modify it by setting the
default `O` parameter on the various [`TargetSize`]-dependent types:

The available [`TargetSize`] implementations are:
* `u32` for 32-bit sized pointers (the default).
* `usize` for target-dependently sized pointers.

```rust
// These no longer panic:
let offset = Offset::<usize>::new(1usize << 32);
let slice = Slice::<u32, usize>::new(Offset::ZERO, 1usize << 32);
let unsize = Unsized::<str, usize>::new(Offset::ZERO, 1usize << 32);
```

[`AlignedBuf`] can also be initialized with a custom [`TargetSize`]:

To initialize an [`AlignedBuf`] with a custom [`TargetSize`] you simply
use this constructor while specifying one of the above parameters:

```rust
use musli_zerocopy::{AlignedBuf, DEFAULT_ALIGNMENT};

let mut buf = AlignedBuf::<usize>::with_capacity_and_alignment(0, DEFAULT_ALIGNMENT);
```

And to use a custom target size in a struct using the [`ZeroCopy`], you
simply specify the default parameter:

```rust
use musli_zerocopy::{ZeroCopy, Ref, Slice, Unsized, AlignedBuf};
use musli_zerocopy::DEFAULT_ALIGNMENT;

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    offset: Ref<u32, usize>,
    slice: Slice::<u32, usize>,
    unsize: Unsized::<str, usize>,
}

let mut buf = AlignedBuf::with_capacity_and_alignment(0, DEFAULT_ALIGNMENT);

let offset = buf.store(&42u32)?;
let slice = buf.store_slice(&[1, 2, 3, 4])?;
let unsize = buf.store_unsized("Hello World")?;

buf.store(&Custom { offset, slice, unsize })?;
```

[`requested()`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.AlignedBuf.html#method.requested
[`Ref`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Ref.html
[`Offset`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Offset.html
[`Slice`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Slice.html
[`Unsized`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.Unsized.html
[`AlignedBuf`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.AlignedBuf.html
[`TargetSize`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/trait.TargetSize.html
[`ZeroCopy`]:
    https://docs.rs/musli-zerocopy/latest/musli_zerocopy/derive.ZeroCopy.html
