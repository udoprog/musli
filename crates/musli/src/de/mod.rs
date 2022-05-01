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
mod type_hint;

pub use self::decode::Decode;
pub use self::decoder::{
    Decoder, NumberVisitor, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, ValueVisitor,
    VariantDecoder,
};
pub use self::type_hint::{LengthHint, NumberHint, TypeHint};
use crate::mode::Mode;

/// An owned decoder.
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
