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
mod number_visitor;
mod type_hint;
mod value_visitor;
mod visitor;

pub use self::decode::{Decode, TraceDecode};
pub use self::decoder::{
    AsDecoder, Decoder, MapDecoder, MapEntryDecoder, MapPairsDecoder, PackDecoder, SequenceDecoder,
    StructDecoder, StructFieldDecoder, StructPairsDecoder, VariantDecoder,
};
pub use self::number_visitor::NumberVisitor;
pub use self::type_hint::{NumberHint, SizeHint, TypeHint};
pub use self::value_visitor::ValueVisitor;
pub use self::visitor::Visitor;

use crate::mode::DefaultMode;

/// Decode to an owned value.
///
/// This is a simpler bound to use than `for<'de> Decode<'de, M>`.
pub trait DecodeOwned<M = DefaultMode>: for<'de> Decode<'de, M> {}

impl<M, D> DecodeOwned<M> for D where D: for<'de> Decode<'de, M> {}
