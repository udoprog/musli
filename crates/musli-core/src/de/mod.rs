//! Traits for generically dealing with a decoding framework.
//!
//! The central traits are [Decode] and [Decoder].
//!
//! A type implementing [Decode] can use an [Decoder] to decode an instance of
//! itself. This also comes with a derive allowing you to derive high
//! performance decoding associated with native Rust types.
//!
//! Note that using derives directly from `musli_core` requires you to use the
//! `#[musli(crate = musli_core)]` attribute.
//!
//! # Examples
//!
//! ```
//! use musli_core::Decode;
//!
//! #[derive(Decode)]
//! #[musli(crate = musli_core)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

pub use musli_macros::Decode;

mod as_decoder;
pub use self::as_decoder::AsDecoder;

mod decode;
pub use self::decode::Decode;

mod decode_slice_builder;
pub use self::decode_slice_builder::DecodeSliceBuilder;

mod decode_bytes;
pub use self::decode_bytes::DecodeBytes;

mod decode_packed;
pub use self::decode_packed::DecodePacked;

mod decode_trace;
pub use self::decode_trace::DecodeTrace;

mod decode_unsized;
pub use self::decode_unsized::DecodeUnsized;

mod decode_unsized_bytes;
pub use self::decode_unsized_bytes::DecodeUnsizedBytes;

mod decoder;
pub use self::decoder::{Decoder, TryFastDecode};

mod entries_decoder;
pub use self::entries_decoder::EntriesDecoder;

mod entry_decoder;
pub use self::entry_decoder::EntryDecoder;

mod map_decoder;
pub use self::map_decoder::MapDecoder;

mod sequence_decoder;
pub use self::sequence_decoder::SequenceDecoder;

mod size_hint;
pub use self::size_hint::SizeHint;

mod skip;
pub use self::skip::Skip;

mod unsized_visitor;
pub use self::unsized_visitor::UnsizedVisitor;

mod variant_decoder;
pub use self::variant_decoder::VariantDecoder;

mod visitor;
pub use self::visitor::Visitor;

use crate::Allocator;

/// Decode to an owned value.
///
/// This is a simpler bound to use than `for<'de> Decode<'de, M, A>`.
pub trait DecodeOwned<M, A>: for<'de> Decode<'de, M, A>
where
    A: Allocator,
{
}

impl<M, D, A> DecodeOwned<M, A> for D
where
    D: for<'de> Decode<'de, M, A>,
    A: Allocator,
{
}

#[doc(hidden)]
pub mod utils;
