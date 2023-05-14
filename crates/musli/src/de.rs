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

pub use self::decode::Decode;
pub use self::decoder::{
    AsDecoder, Decoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, VariantDecoder,
};
pub use self::number_visitor::NumberVisitor;
pub use self::type_hint::{LengthHint, NumberHint, TypeHint};
pub use self::value_visitor::ValueVisitor;
use crate::mode::Mode;

/// Decode to a `'static` value.
pub trait DecodeOwned<M>: for<'de> Decode<'de, M>
where
    M: Mode,
{
}

impl<M, D> DecodeOwned<M> for D
where
    D: for<'de> Decode<'de, M>,
    M: Mode,
{
}
