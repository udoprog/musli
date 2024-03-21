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
pub use self::encode::{Encode, TraceEncode};

mod encode_bytes;
pub use self::encode_bytes::EncodeBytes;

mod encoder;
pub use self::encoder::Encoder;

mod sequence_encoder;
pub use self::sequence_encoder::SequenceEncoder;

mod map_encoder;
pub use self::map_encoder::MapEncoder;

mod map_entry_encoder;
pub use self::map_entry_encoder::MapEntryEncoder;

mod map_entries_encoder;
pub use self::map_entries_encoder::MapEntriesEncoder;

mod struct_encoder;
pub use self::struct_encoder::StructEncoder;

mod struct_field_encoder;
pub use self::struct_field_encoder::StructFieldEncoder;

mod variant_encoder;
pub use self::variant_encoder::VariantEncoder;
