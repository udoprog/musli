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

/// Derive which automatically implements the [`Decode` trait].
///
/// See the [`derives` module] for detailed documentation.
///
/// [`derives` module]: <https://docs.rs/musli/latest/musli/_help/derives/index.html>
/// [`Decode` trait]: <https://docs.rs/musli/latest/musli/trait.Decode.html>
///
/// # Examples
///
/// ```
/// use musli_core::Decode;
///
/// #[derive(Decode)]
/// #[musli(crate = musli_core)]
/// struct MyType {
///     data: [u8; 128],
/// }
/// ```
#[doc(inline)]
pub use musli_macros::Decode;

#[doc(inline)]
pub use self::__traits::*;

mod as_decoder;
mod decode;
mod decode_bytes;
mod decode_owned;
mod decode_packed;
mod decode_slice_builder;
mod decode_trace;
mod decode_unsized;
mod decode_unsized_bytes;
mod decoder;
mod entries_decoder;
mod entry_decoder;
mod into_decoder;
mod map_decoder;
mod sequence_decoder;
mod size_hint;
mod skip;
mod unsized_visitor;
mod variant_decoder;
mod visitor;

#[doc(hidden)]
pub mod __traits {
    pub use super::as_decoder::AsDecoder;
    pub use super::decode::Decode;
    pub use super::decode_bytes::DecodeBytes;
    pub use super::decode_owned::DecodeOwned;
    pub use super::decode_packed::DecodePacked;
    pub use super::decode_slice_builder::DecodeSliceBuilder;
    pub use super::decode_trace::DecodeTrace;
    pub use super::decode_unsized::DecodeUnsized;
    pub use super::decode_unsized_bytes::DecodeUnsizedBytes;
    pub use super::decoder::{Decoder, TryFastDecode};
    pub use super::entries_decoder::EntriesDecoder;
    pub use super::entry_decoder::EntryDecoder;
    pub use super::map_decoder::MapDecoder;
    pub use super::sequence_decoder::SequenceDecoder;
    pub use super::size_hint::SizeHint;
    pub use super::skip::Skip;
    pub use super::unsized_visitor::UnsizedVisitor;
    pub use super::variant_decoder::VariantDecoder;
    pub use super::visitor::Visitor;
}

#[doc(hidden)]
pub mod utils;
