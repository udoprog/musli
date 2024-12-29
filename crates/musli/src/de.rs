//! Traits for generically dealing with a decoding framework.
//!
//! The central traits are [Decode] and [Decoder].
//!
//! A type implementing [Decode] can use an [Decoder] to decode an instance of
//! itself. This also comes with a derive allowing you to derive high
//! performance decoding associated with native Rust types.
//!
//! ```
//! use musli::Decode;
//!
//! #[derive(Decode)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

#[doc(inline)]
pub use musli_core::de::{
    AsDecoder, Decode, DecodeBytes, DecodeOwned, DecodePacked, DecodeSliceBuilder, DecodeTrace,
    DecodeUnsized, DecodeUnsizedBytes, Decoder, EntriesDecoder, EntryDecoder, MapDecoder,
    SequenceDecoder, SizeHint, Skip, UnsizedVisitor, VariantDecoder, Visitor,
};

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
pub(crate) use musli_core::de::utils;
