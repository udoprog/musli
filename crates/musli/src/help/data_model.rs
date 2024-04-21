//! The data model of Müsli.
//!
//! Müsli supports the following types natively:
//! * Boolean values.
//! * Integers (8 to 128 bits).
//! * Floats (32 and 64 bits).
//! * Optional values.
//! * Strings (a sequence of bytes known to be a valid utf-8 string).
//! * Bytes (a sequence of raw bytes).
//! * Sequences.
//! * Maps or a sequence of pairs. There is no restriction on the key, and they
//!   can contain duplicates.
//! * A variant, which is a simple pair of a key and a value, where the key is
//!   the discriminant identifying the type.
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
//! [`derives`]: super::derives
