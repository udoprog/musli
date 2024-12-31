//! Traits for generically dealing with an encoding framework.
//!
//! The central traits are [Encode] and [Encoder].
//!
//! A type implementing [Encode] can use an [Encoder] to encode itself. This
//! also comes with a derive allowing you to derive high performance encoding
//! associated with native Rust types.
//!
//! Note that using derives directly from `musli_core` requires you to use the
//! `#[musli(crate = musli_core)]` attribute.
//!
//! # Examples
//!
//! ```
//! use musli_core::Encode;
//!
//! #[derive(Encode)]
//! #[musli(crate = musli_core)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

pub use musli_macros::Encode;

mod encode;
pub use self::encode::Encode;

mod encode_bytes;
pub use self::encode_bytes::EncodeBytes;

mod encode_packed;
pub use self::encode_packed::EncodePacked;

mod encode_trace;
pub use self::encode_trace::EncodeTrace;

mod encoder;
pub use self::encoder::{Encoder, TryFastEncode};

mod entries_encoder;
pub use self::entries_encoder::EntriesEncoder;

mod entry_encoder;
pub use self::entry_encoder::EntryEncoder;

mod map_encoder;
pub use self::map_encoder::MapEncoder;

mod sequence_encoder;
pub use self::sequence_encoder::SequenceEncoder;

mod variant_encoder;
pub use self::variant_encoder::VariantEncoder;

#[doc(hidden)]
pub mod utils;
