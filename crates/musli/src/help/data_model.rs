//! The data model of Müsli.
//!
//! Müsli supports the following fundamental types:
//!
//! * Empty[^empty].
//! * Boolean values.
//! * Unsigned integers (corresponding to [u8], [u16], [u32], [u64], and
//!   [u128]).
//! * Signed integers (corresponding to [i8], [i16], [i32], [i64], and [i128]).
//! * Floats (corresponding to [f32] and [f64]).
//! * Optional values[^option].
//! * Bytes, a raw byte sequence.
//! * Strings, a byte sequence known to be a valid utf-8 string.
//! * Sequences[^container].
//! * Maps[^container]. There is no restriction on the key, and they can contain
//!   duplicates.
//! * A variant[^container], which is a simple kind of container containing a
//!   key and a value. The key is the discriminant identifying the variant.
//!
//! These are used as the basis to serialize any Rust type.
//!
//! By default, Rust types are mapped like the following:
//!
//! * Structs are serialized as maps, where the key is the `#[musli(name =..)]`
//!   of the field.
//! * Tuples are serialized as sequences.
//! * Enums are serialized as variants, where the key is the `#[musli(name =
//!   ..)]` of the variant.
//!
//! To control the exact behavior of serialization, see the [`derives`] section.
//!
//! [^empty]: Empty values serve the purpose of acting as placeholder for things
//!     which have no value, such as the empty tuple `()` or `PhantomData<T>`.
//!     Encoders are free to treat them however they want to. For descriptive
//!     encoders where it's possible, it's typical for empty values to be
//!     skipped.
//!
//! [^option]: This directly corresponds to the `Option<T>` type in Rust. While
//!     many formats internally handles optionality since it is a requirement to
//!     skip over unknown fields, this type is given special treatment to ensure
//!     that formats which are not descriptive can handle them. Without this, it
//!     would be impossible for the non-packed [`storage`] format to provide
//!     partial upgrade safety.
//!
//! [^container]: There is no particular restriction that containers must
//!     contain uniform types. However, this is typically enforced by the types
//!     deriving [`Encode`] and [`Decode`] in Rust.
//!
//! [`storage`]: crate::storage
//! [`derives`]: super::derives
//! [`Encode`]: crate::Encode
//! [`Decode`]: crate::Decode
