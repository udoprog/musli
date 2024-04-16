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
#[doc(inline)]
pub use self::skip::Skip;

mod size_hint;
#[doc(inline)]
pub use self::size_hint::SizeHint;

mod as_decoder;
#[doc(inline)]
pub use self::as_decoder::AsDecoder;

mod decode;
#[doc(inline)]
pub use self::decode::{Decode, TraceDecode};

mod decode_unsized;
#[doc(inline)]
pub use self::decode_unsized::DecodeUnsized;

mod decode_unsized_bytes;
#[doc(inline)]
pub use self::decode_unsized_bytes::DecodeUnsizedBytes;

mod decode_bytes;
#[doc(inline)]
pub use self::decode_bytes::DecodeBytes;

mod decoder;
#[doc(inline)]
pub use self::decoder::Decoder;

mod map_decoder;
#[doc(inline)]
pub use self::map_decoder::MapDecoder;

mod map_entries_decoder;
#[doc(inline)]
pub use self::map_entries_decoder::MapEntriesDecoder;

mod map_entry_decoder;
#[doc(inline)]
pub use self::map_entry_decoder::MapEntryDecoder;

mod number_visitor;
#[doc(inline)]
pub use self::number_visitor::NumberVisitor;

mod pack_decoder;
#[doc(inline)]
pub use self::pack_decoder::PackDecoder;

mod tuple_decoder;
#[doc(inline)]
pub use self::tuple_decoder::TupleDecoder;

mod sequence_decoder;
#[doc(inline)]
pub use self::sequence_decoder::SequenceDecoder;

mod value_visitor;
#[doc(inline)]
pub use self::value_visitor::ValueVisitor;

mod variant_decoder;
#[doc(inline)]
pub use self::variant_decoder::VariantDecoder;

mod visitor;
#[doc(inline)]
pub use self::visitor::Visitor;

use crate::mode::DefaultMode;

/// Decode to an owned value.
///
/// This is a simpler bound to use than `for<'de> Decode<'de, M>`.
pub trait DecodeOwned<M = DefaultMode>: for<'de> Decode<'de, M> {}

impl<M, D> DecodeOwned<M> for D where D: for<'de> Decode<'de, M> {}
