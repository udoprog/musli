//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy)
//!
//! Refreshingly simple zero copy primitives by Müsli.
//!
//! This provides a basic set of tools to deal with types which do not require
//! copying during deserialization. You define the `T`, and we provide the safe
//! `&[u8]` to `&T` conversion.
//!
//! For a detailed overview of how this works, see the [`ZeroCopy`] derive.
//! There's also a high level guide just below.
//!
//! Reading a zero-copy structure has full `#[no_std]` support. Constructing
//! ones currently require the `alloc` feature to be enabled.
//!
//! This crate also includes a couple of neat and very efficient high level data
//! structures:
//! * [`phf`] provides maps and sets based on [`phf` crate], or perfect hash
//!   functions.
//! * [`swiss`] is a port of the [`hashbrown` crate] which is a Google
//!   SwissTable implementation.
//!
//! <br>
//!
//! ## Why should I consider `musli-zerocopy` over X?
//!
//! Since this is the first question anyone will ask, here is how we differ from
//! other popular libraries:
//! * [`zerocopy`](https://docs.rs/zerocopy) doesn't support padded
//!   structs[^padded], bytes to reference conversions, or conversions which do
//!   not permit all bit patterns to be inhabited by zeroes[^zeroes]. We still
//!   want to provide more of a complete toolkit that you'd need to build and
//!   interact with complex data structures, such as the [`phf`] and [`swiss`]
//!   modules. This crate might indeed at some point make use of `zerocopy`'s
//!   traits.
//! * [`rkyv`](https://docs.rs/rkyv) operates on `#[repr(Rust)]` types and from
//!   this derives an optimized `Archived` variation for you. This library
//!   operates on the equivalent of `Archived` variant directly instead that you
//!   should build and interact with by hand. `rkyv` is also explicitly not
//!   sound unless you build it with the `strict` feature, and even with the
//!   feature enabled it doesn't pass Miri. Neither of these are strict blockers
//!   for users of the library, but do constrain its safe applicability in ways
//!   this library does not.
//!
//! [^padded]: This is on their roadmap.
//! [^zeroes]: FromBytes extends FromZeroes.
//!
//! The `tests` test suite has been extended to support some level of
//! zerocopy types, all though since the feature sets vary so widely between the
//! crates it's hard to compare all of them against each other, but the
//! following showcases how they currently compare to `musli-zerocopy`.
//!
//! Encoding a packed variant of the `Primitives` model:
//!
//! [^musli-zerocopy]: Generated with `cargo bench -p tests --no-default-features --features no-rt,musli-zerocopy,zerocopy -- primpacked`
//! [^musli-rkyv]: Generated with `cargo bench -p tests --no-default-features --features no-rt,musli-zerocopy,rkyv -- primitives`
//!
//! <br>
//!
//! ## Guide
//!
//! Zero-copy in this library refers to the act of interacting with data
//! structures that reside directly in `&[u8]` memory without the need to first
//! decode them.
//!
//! Conceptually it works a bit like this.
//!
//! Say you want to store the string `"Hello World!"`.
//!
//! ```rust
//! use musli_zerocopy::OwnedBuf;
//!
//! let mut buf = OwnedBuf::new();
//! let string = buf.store_unsized("Hello World!");
//! let reference = buf.store(&string);
//!
//! assert_eq!(reference.offset(), 12);
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! This would result in the following buffer:
//!
//! ```text
//! 0000: "Hello World!"
//! // Might get padded to ensure that the size is aligned by 4 bytes.
//! 0012: offset -> 0000
//! 0016: size -> 12
//! ```
//!
//! What we see at offset `0016` is an 8 byte [`Unsized<str>`]. The first field
//! stores the offset where to fetch the string, and the second field the length
//! of the string.
//!
//! Let's have a look at a [`Slice<u32>`] next:
//!
//! ```rust
//! use musli_zerocopy::OwnedBuf;
//!
//! let mut buf = OwnedBuf::new();
//! let slice = buf.store_slice(&[1u32, 2, 3, 4]);
//! let reference = buf.store(&slice);
//!
//! assert_eq!(reference.offset(), 16);
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! This would result in the following buffer:
//!
//! ```text
//! 0000: u32 -> 1
//! 0004: u32 -> 2
//! 0008: u32 -> 3
//! 0012: u32 -> 4
//! 0016: offset -> 0000
//! 0020: length -> 4
//! ```
//!
//! At address `0016` we store two fields which corresponds to a [`Slice<u32>`].
//!
//! Next lets investigate an example using a `Custom` struct:
//!
//! ```
//! # use anyhow::Context;
//! use core::mem::size_of;
//! use musli_zerocopy::{OwnedBuf, ZeroCopy};
//! use musli_zerocopy::pointer::Unsized;
//!
//! #[derive(ZeroCopy)]
//! #[repr(C)]
//! struct Custom {
//!     field: u32,
//!     string: Unsized<str>,
//! }
//!
//! let mut buf = OwnedBuf::new();
//!
//! let string = buf.store_unsized("Hello World!");
//! let custom = buf.store(&Custom { field: 42, string });
//!
//! // The buffer stores both the unsized string and the Custom element.
//! assert!(buf.len() >= 24);
//! // We assert that the produced alignment is smaller or equal to 8
//! // since we'll be relying on this below.
//! assert!(buf.requested() <= 8);
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! This would result in the following buffer:
//!
//! ```text
//! 0000: "Hello World!"
//! 0012: u32 -> 42
//! 0016: offset -> 0000
//! 0020: size -> 12
//! ```
//!
//! Our struct starts at address `0012`, first we have the `u32` field, and
//! immediately after that we have the string.
//!
//! <br>
//!
//! ## Reading data
//!
//! Later when we want to use the type, we take the buffer we've generated and
//! include it somewhere else.
//!
//! There's a few pieces of data (lets call it DNA) we need to have to read a
//! type back from a raw buffer:
//! * The *alignment* of the buffer. Which you can read through the
//!   [`requested()`]. On the receiving end we need to ensure that the buffer
//!   follow this alignment. Dynamically this can be achieved by loading the
//!   buffer using [`aligned_buf(bytes, align)`]. Other tricks include embedding
//!   a static buffer inside of an aligned newtype which we'll showcase below.
//!   Networked applications might simply agree to use a particular alignment up
//!   front. This alignment has to be compatible with the types being coerced.
//! * The *endianness* of the machine which produced the buffer. Any numerical
//!   elements will in native endian ordering, so they would have to be adjusted
//!   on the read side if it differ.
//! * The type definition which is being read which implements [`ZeroCopy`].
//!   This is `Custom` above. The [`ZeroCopy`] derive ensures that we can safely
//!   coerce a buffer into a reference of the type. The data might at worst be
//!   garbled, but we can never do anything unsound while using safe APIs.
//! * The offset at where the [`ZeroCopy`] structure is read. To read a
//!   structure we combine a pointer and a type into a [`Ref`] instance.
//!
//! If the goal is to both produce and read the buffer on the same system
//! certain assumptions can be made. And if those assumptions turn out to be
//! wrong the worst outcome will only ever be an error as long as you're using
//! the safe APIs or abide by the safety documentation of the unsafe APIs.
//!
//! > **Info** A note on sending data over the network. This is perfectly doable
//! > as long as you include the alignment of the buffer and the endianness of
//! > the data structure. Both of these can be retrieved:
//! >
//! > ```no_run
//! > use musli_zerocopy::OwnedBuf;
//! > let buf: OwnedBuf = todo!();
//! >
//! > let is_little_endian = cfg!(target_endian = "little");
//! > let alignment = buf.requested();
//! > ```
//!
//! The following is an example of reading the type directly out of a newtype
//! aligned `&'static [u8]` buffer:
//!
//! ```
//! # use musli_zerocopy::ZeroCopy;
//! # use musli_zerocopy::pointer::Unsized;
//! # macro_rules! include_bytes {
//! # ($path:literal) => { &[
//! #    b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd', b'!',
//! #    42, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0,
//! # ] };
//! # }
//! # #[derive(ZeroCopy)]
//! # #[repr(C)]
//! # struct Custom { field: u32, string: Unsized<str> }
//! use core::mem::size_of;
//! use musli_zerocopy::{Buf, Ref};
//!
//! // Helper to force the static buffer to be aligned like `A`.
//! #[repr(C)]
//! struct Align<A, T: ?Sized>([A; 0], T);
//!
//! static BYTES: &Align<u64, [u8]> = &Align([], *include_bytes!("custom.bin"));
//!
//! let buf = Buf::new(&BYTES.1);
//!
//! // Construct a pointer into the buffer.
//! let custom = Ref::<Custom>::new(BYTES.1.len() - size_of::<Custom>());
//!
//! let custom: &Custom = buf.load(custom)?;
//! assert_eq!(custom.field, 42);
//! assert_eq!(buf.load(custom.string)?, "Hello World!");
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! <br>
//!
//! ## Writing data at offset zero
//!
//! Most of the time you want to write data where the first element in the
//! buffer is the element currently being written.
//!
//! This is useful because it satisfies the last requirement above, *the offset*
//! at where the struct can be read will then simply be zero, and all the data
//! it depends on are stored at subsequent offsets.
//!
//! ```
//! # use musli_zerocopy::ZeroCopy;
//! # use musli_zerocopy::pointer::Unsized;
//! # #[derive(ZeroCopy)]
//! # #[repr(C)]
//! # struct Custom { field: u32, string: Unsized<str> }
//! use musli_zerocopy::{OwnedBuf, Ref};
//! use musli_zerocopy::mem::MaybeUninit;
//!
//! let mut buf = OwnedBuf::new();
//! let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();
//!
//! let string = buf.store_unsized("Hello World!");
//!
//! buf.load_uninit_mut(reference).write(&Custom { field: 42, string });
//!
//! let reference = reference.assume_init();
//! assert_eq!(reference.offset(), 0);
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! <br>
//!
//! ## Limits
//!
//! Offset, the size of unsized values, and slice lengths are all limited to
//! 32-bit. The system you're using must have a `usize` type which is at least
//! 32-bits wide. This is done to save space by default.
//!
//! The pointer width on the system is checked at compile time, while trying to
//! use an offset or a size larger than `2^32` will result in a panic.
//!
//! Example of using an address larger than `2^32` causing a panic:
//!
//! ```should_panic
//! # use musli_zerocopy::{ZeroCopy, Ref};
//! # #[derive(ZeroCopy)]
//! # #[repr(C)]
//! # struct Custom;
//! Ref::<Custom>::new(1usize << 32);
//! ```
//!
//! Example panic using a [`Slice`] with a length larger than `2^32`:
//!
//! ```should_panic
//! # use musli_zerocopy::ZeroCopy;
//! # use musli_zerocopy::pointer::Slice;
//! # #[derive(ZeroCopy)]
//! # #[repr(C)]
//! # struct Custom;
//! Slice::<Custom>::new(0, 1usize << 32);
//! ```
//!
//! Example panic using an [`Unsized`] value with a size larger than `2^32`:
//!
//! ```should_panic
//! # use musli_zerocopy::pointer::Unsized;
//! Unsized::<str>::new(0, 1usize << 32);
//! ```
//!
//! If you want to address data larger than this limit, it is recommended that
//! you partition your dataset into 32-bit addressable chunks.
//!
//! If you really want to change this limit, you can modify it by setting the
//! default `O` parameter on the various [`Size`]-dependent types:
//!
//! The available [`Size`] implementations are:
//! * `u32` for 32-bit sized pointers (the default).
//! * `usize` for target-dependently sized pointers.
//!
//! ```
//! # use musli_zerocopy::{Ref, ZeroCopy};
//! # use musli_zerocopy::pointer::{Slice, Unsized};
//! # #[derive(ZeroCopy)]
//! # #[repr(C)]
//! # struct Custom;
//! // These no longer panic:
//! let reference = Ref::<Custom, usize>::new(1usize << 32);
//! let slice = Slice::<Custom, usize>::new(0, 1usize << 32);
//! let unsize = Unsized::<str, usize>::new(0, 1usize << 32);
//! ```
//!
//! To initialize an [`OwnedBuf`] with a custom [`Size`] you use this
//! constructor with a custom parameter and :
//!
//! ```
//! use musli_zerocopy::OwnedBuf;
//! use musli_zerocopy::buf::DefaultAlignment;
//!
//! let mut buf = OwnedBuf::<usize>::with_capacity_and_alignment::<DefaultAlignment>(0);
//! ```
//!
//! The [`Size`] you've specified during construction of an [`OwnedBuf`] will
//! then carry into any pointers it return:
//!
//! ```
//! use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
//! use musli_zerocopy::buf::DefaultAlignment;
//! use musli_zerocopy::pointer::{Slice, Unsized};
//!
//! #[derive(ZeroCopy)]
//! #[repr(C)]
//! struct Custom { reference: Ref<u32, usize>, slice: Slice::<u32, usize>, unsize: Unsized::<str, usize> }
//!
//! let mut buf = OwnedBuf::with_capacity_and_alignment::<DefaultAlignment>(0);
//!
//! let reference = buf.store(&42u32);
//! let slice = buf.store_slice(&[1, 2, 3, 4]);
//! let unsize = buf.store_unsized("Hello World");
//!
//! buf.store(&Custom { reference, slice, unsize });
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! [`swiss`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/swiss/index.html
//! [`phf`]: https://docs.rs/musli-zerocopy/latest/musli_zerocopy/phf/index.html
//! [`phf` crate]: https://docs.rs/phf
//! [`hashbrown` crate]: https://docs.rs/phf
//! [`requested()`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/struct.OwnedBuf.html#method.requested
//! [`ZeroCopy`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/derive.ZeroCopy.html
//! [`Ref`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Ref.html
//! [`Slice`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Slice.html
//! [`Slice<u32>`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Slice.html
//! [`Unsized`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Unsized.html
//! [`Unsized<str>`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/struct.Unsized.html
//! [`OwnedBuf`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/buf/struct.OwnedBuf.html
//! [`Size`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/trait.Size.html
//! [`aligned_buf(bytes, align)`]:
//!     https://docs.rs/musli-zerocopy/latest/musli_zerocopy/pointer/trait.Size.html

#![no_std]
#![allow(clippy::module_inception)]
#![deny(missing_docs)]
#![cfg_attr(all(feature = "nightly", test), feature(repr128))]
#![cfg_attr(all(feature = "nightly", test), allow(incomplete_features))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[doc(inline)]
pub use self::buf::Buf;
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use self::buf::OwnedBuf;
pub mod buf;

pub mod mem;

#[doc(inline)]
pub use self::error::Error;
mod error;

/// `Result` alias provided for convenience.
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(inline)]
pub use self::traits::ZeroCopy;
pub mod traits;

pub(crate) mod sip;

pub mod phf;
pub mod swiss;

#[doc(inline)]
pub use self::pointer::Ref;
pub mod pointer;

/// Derive macro to implement [`ZeroCopy`].
///
/// Implementing this trait ensures that the type can safely be coerced to and
/// from initialized bytes.
///
/// <br>
///
/// # Using with structs
///
/// The following are the requirements for deriving structs:
/// * The struct must either be `#[repr(C)]` or `[repr(transparent)]`.
/// * All fields in the struct must either implement [`ZeroCopy`] or be
///   [`ZeroSized`] and marked as `#[zero_copy(ignore)]`.
///
/// If the struct is zero-sized, it will implement [`ZeroSized`] along with the
/// [`ZeroCopy`] trait.
///
/// [`ZeroSized`]: crate::traits::ZeroSized
///
/// ```
/// use musli_zerocopy::{OwnedBuf, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(C, align(128))]
/// struct Custom { field: u32 }
///
/// let mut buf = OwnedBuf::new();
/// let ptr = buf.store(&Custom { field: 10 });
/// let buf = buf.into_aligned();
/// assert_eq!(buf.load(ptr)?, &Custom { field: 10 });
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// [`ZeroCopy`]: trait@crate::zero_copy::ZeroCopy
///
/// <br>
///
/// # Using with enums
///
/// The following are the requirements for deriving for enums:
/// * The enum must be marked with a valid, fixed representation. Such as
///   `#[repr(u8)]`, or `#[repr(usize)]`.
/// * If custom discriminators are used, only constant values can be used.
///
/// ```
/// use musli_zerocopy::{OwnedBuf, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(u32)]
/// enum Flags {
///     First = 1,
///     Second, // will be automatically assigned 2
///     Third = 5,
/// }
///
/// let mut buf = OwnedBuf::new();
/// let ptr = buf.store(&Flags::First);
/// let buf = buf.into_aligned();
/// assert_eq!(buf.load(ptr)?, &Flags::First);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// <br>
///
/// # Padding
///
/// The constant [`ZeroCopy::PADDING`] determines whether the derives struct
/// uses padding or not. This derive currently uses a fairly conservative
/// algorithm:
///
/// The constant [`ZeroCopy::PADDING`] will be set to `true` if:
/// * The size of the type is 0, and the alignment is larger than 1. This
///   indicates a zero-sized type with an explicit `#[repr(align(N))]` that is
///   not set to 1.
/// * The sum of the size of all the fields is not the same as the size of the
///   type.
/// * Any of the fields has its [`ZeroCopy::PADDING`] set to `true`.
/// * For enums, we test every variant with the same rules, except each variant
///   is treated as a struct where the discriminant (`u32` in `#[repr(u32)]`) is
///   treated like [a leading hidden field].
///
/// [`ZeroCopy::PADDING`]: crate::traits::ZeroCopy::PADDING
/// [a first hidden field]: https://doc.rust-lang.org/beta/reference/type-layout.html#primitive-representation-of-enums-with-fields
///
/// ```
/// use musli_zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Zst;
/// const _: () = assert!(!Zst::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(C, align(1))]
/// struct ZstAlign1;
/// const _: () = assert!(!ZstAlign1::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(C, align(128))]
/// struct ZstPadded;
/// const _: () = assert!(ZstPadded::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(u8)]
/// enum ZstEnum { EmptyField }
/// const _: () = assert!(!ZstEnum::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(u8)]
/// enum SameEnum { Variant1(u8), Variant2(u8) }
/// const _: () = assert!(!SameEnum::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(u16)]
/// enum PaddedU16 { Variant1(u8), Variant2(u8) }
/// const _: () = assert!(PaddedU16::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(u16)]
/// enum NotPaddedU16 { Variant1(u8, u8), Variant2([u8; 2]), Variant3(u16) }
/// const _: () = assert!(!NotPaddedU16::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(C, packed)]
/// struct Packed { inner: u8, inner2: u32 }
/// const _: () = assert!(!Packed::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(C, packed(2))]
/// struct Packed1 { inner: u8, inner2: u32 }
/// const _: () = assert!(Packed1::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Inner { inner: u8, inner2: u32 }
/// const _: () = assert!(Inner::PADDED);
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Outer { first: u8, inner: Inner }
/// const _: () = assert!(Outer::PADDED);
/// ```
///
/// <br>
///
/// # Supported attributes
///
/// <br>
///
/// ## Type attributes
///
/// The following `repr` attributes are supported:
/// * `#[repr(C)]` - Ensures that the type has the mandatory represention.
/// * `#[repr(transparent)]` - If there is a single field inside of the marked
///   struct which implements `ZeroCopy`.
/// * `#[repr(align(N))]` - Allows for control over the type's alignment.
/// * `#[repr(packed)]` or `#[repr(packed(N))]` - Allows for control over the
///   struct alignment. Namely to lower it. This is not supported for enums.
///
/// The following `zero_copy(..)` attribute are supported:
///
/// <br>
///
/// ### `zero_copy(bounds = {<bound>,*})`
///
/// Allows for adding additional bounds to implement `ZeroCopy` for generic
/// types:
///
/// ```
/// use musli_zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// #[zero_copy(bounds = {A: ZeroCopy, B: ZeroCopy})]
/// struct Pair<A, B> {
///     left: A,
///     right: B,
/// }
/// ```
///
/// <br>
///
/// ### `zero_copy(crate = <path>)`
///
/// Allows for specifying a custom path to the `musli_zerocopy`` crate
/// (default).
///
/// ```
/// # use musli_zerocopy as zerocopy;
/// use zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// #[zero_copy(crate = zerocopy)]
/// struct Custom { field: u32 }
/// ```
#[doc(inline)]
pub use musli_macros::ZeroCopy;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub mod __private {
    pub mod result {
        pub use ::core::result::Result;
    }

    pub mod mem {
        pub use ::core::mem::{align_of, size_of};
    }

    pub mod ptr {
        pub use ::core::ptr::addr_of;
    }
}
