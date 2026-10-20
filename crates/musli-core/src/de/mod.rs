//! Traits for generically dealing with a decoding framework.
//!
//! The central traits are [Decode] and [Decoder].
//!
//! A type implementing [Decode] can use an [Decoder] to decode an instance of
//! itself. This also comes with a derive allowing you to derive high
//! performance decoding associated with native Rust types.
//!
//! The derive in this module generates code which refers to `musli_core`, so a
//! crate which depends on `musli_core` alone can use it without a
//! `#[musli(crate = ..)]` attribute. If the dependency is renamed in
//! `Cargo.toml`, point the derive at the new name with
//! `#[musli(crate = <name>)]`.
//!
//! # Examples
//!
//! ```
//! use musli_core::Decode;
//!
//! #[derive(Decode)]
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
/// struct MyType {
///     data: [u8; 128],
/// }
/// ```
///
/// A crate which renames `musli_core`, such as with `musli_core01 = { package =
/// "musli-core", .. }` in `Cargo.toml`, has to name it with `#[musli(crate =
/// musli_core01)]`.
#[doc(inline)]
pub use musli_macros::MusliCoreDecode as Decode;

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
