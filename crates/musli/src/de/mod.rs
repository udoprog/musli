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

mod skip;
pub use self::skip::Skip;

mod as_decoder;
pub use self::as_decoder::AsDecoder;

mod decode;
pub use self::decode::{Decode, TraceDecode};

mod visit;
pub use self::visit::Visit;

mod visit_bytes;
pub use self::visit_bytes::VisitBytes;

mod decode_bytes;
pub use self::decode_bytes::DecodeBytes;

mod decoder;
pub use self::decoder::Decoder;

mod map_decoder;
pub use self::map_decoder::MapDecoder;

mod map_entries_decoder;
pub use self::map_entries_decoder::MapEntriesDecoder;

mod map_entry_decoder;
pub use self::map_entry_decoder::MapEntryDecoder;

mod number_visitor;
pub use self::number_visitor::NumberVisitor;

mod pack_decoder;
pub use self::pack_decoder::PackDecoder;

mod tuple_decoder;
pub use self::tuple_decoder::TupleDecoder;

mod sequence_decoder;
pub use self::sequence_decoder::SequenceDecoder;

mod struct_decoder;
pub use self::struct_decoder::StructDecoder;

mod struct_field_decoder;
pub use self::struct_field_decoder::StructFieldDecoder;

mod struct_fields_decoder;
pub use self::struct_fields_decoder::StructFieldsDecoder;

mod type_hint;
pub use self::type_hint::{NumberHint, SizeHint, TypeHint};

mod value_visitor;
pub use self::value_visitor::ValueVisitor;

mod variant_decoder;
pub use self::variant_decoder::VariantDecoder;

mod visitor;
pub use self::visitor::Visitor;

use crate::mode::DefaultMode;

/// Decode to an owned value.
///
/// This is a simpler bound to use than `for<'de> Decode<'de, M>`.
pub trait DecodeOwned<M = DefaultMode>: for<'de> Decode<'de, M> {}

impl<M, D> DecodeOwned<M> for D where D: for<'de> Decode<'de, M> {}
