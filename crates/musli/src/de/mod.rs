//! Traits for generically dealing with a decoding framework.
//!
//! The central traits are [Decode] and [Decoder].
//!
//! A type implementing [Decode] can use an [Decoder] to decode an instance of
//! itself. This also comes with a derive allowing you to derive high
//! performance decoding associated with native Rust types.
//!
//! ```rust
//! use musli::Decode;
//!
//! #[derive(Decode)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

mod decode;
mod decoder;

pub use self::decode::Decode;
pub use self::decoder::{
    Decoder, MapDecoder, MapEntryDecoder, PackDecoder, SequenceDecoder, StructDecoder,
    StructFieldDecoder, TupleDecoder, TupleFieldDecoder, VariantDecoder,
};

/// An owned decoder.
pub trait DecodeOwned: for<'de> Decode<'de> {}

impl<D> DecodeOwned for D where D: for<'de> Decode<'de> {}
