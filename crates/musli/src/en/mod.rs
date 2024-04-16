//! Traits for generically dealing with an encoding framework.
//!
//! The central traits are [Encode] and [Encoder].
//!
//! A type implementing [Encode] can use an [Encoder] to encode itself. This
//! also comes with a derive allowing you to derive high performance encoding
//! associated with native Rust types.
//!
//! ```rust
//! use musli::Encode;
//!
//! #[derive(Encode)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

mod encode;
#[doc(inline)]
pub use self::encode::{Encode, TraceEncode};

mod encode_bytes;
#[doc(inline)]
pub use self::encode_bytes::EncodeBytes;

mod encoder;
#[doc(inline)]
pub use self::encoder::Encoder;

mod sequence_encoder;
#[doc(inline)]
pub use self::sequence_encoder::SequenceEncoder;

mod pack_encoder;
#[doc(inline)]
pub use self::pack_encoder::PackEncoder;

mod map_encoder;
#[doc(inline)]
pub use self::map_encoder::MapEncoder;

mod map_entry_encoder;
#[doc(inline)]
pub use self::map_entry_encoder::MapEntryEncoder;

mod map_entries_encoder;
#[doc(inline)]
pub use self::map_entries_encoder::MapEntriesEncoder;

mod variant_encoder;
#[doc(inline)]
pub use self::variant_encoder::VariantEncoder;
