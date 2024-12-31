//! Traits for generically dealing with an encoding framework.
//!
//! The central traits are [Encode] and [Encoder].
//!
//! A type implementing [Encode] can use an [Encoder] to encode itself. This
//! also comes with a derive allowing you to derive high performance encoding
//! associated with native Rust types.
//!
//! ```
//! use musli::Encode;
//!
//! #[derive(Encode)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

#[doc(inline)]
pub use musli_core::en::{
    Encode, EncodeBytes, EncodePacked, EncodeTrace, Encoder, EntriesEncoder, EntryEncoder,
    MapEncoder, SequenceEncoder, TryFastEncode, VariantEncoder,
};

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
pub(crate) use musli_core::en::utils;
