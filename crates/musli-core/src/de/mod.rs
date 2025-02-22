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
/// use musli::Decode;
///
/// #[derive(Decode)]
/// struct MyType {
///     data: [u8; 128],
/// }
/// ```
///
/// When using through [`musli_core`][crate], the crate needs to be specified:
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
pub use musli_macros::Decode;

/// This is an attribute macro that must be used when implementing a
/// [`Decoder`].
///
/// It is required to use because a [`Decoder`] implementation might introduce
/// new associated types in the future, and this is [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
///
/// Note that if the `Cx` or `Mode` associated types are not specified, they
/// will be defaulted to any type parameters which starts with the uppercase `C`
/// or `M` respectively.
///
/// Note that using derives directly from `musli_core` requires you to use the
/// `#[musli_core::de::decoder(crate = musli_core)]` attribute.
///
/// # Examples
///
/// ```
/// use std::fmt;
/// use std::marker::PhantomData;
///
/// use musli_core::Context;
/// use musli_core::de::{Decoder, Decode};
///
/// struct MyDecoder<C, M> {
///     cx: C,
///     _marker: PhantomData<M>,
/// }
///
/// #[musli_core::de::decoder(crate = musli_core)]
/// impl<'de, C, M> Decoder<'de> for MyDecoder<C, M>
/// where
///     C: Context,
///     M: 'static,
/// {
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     #[inline]
///     fn decode_u32(self) -> Result<u32, Self::Error> {
///         Ok(42)
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_macros::decoder;

/// This is an attribute macro that must be used when implementing a
/// [`Visitor`].
///
/// It is required to use because a [`Visitor`] implementation might introduce
/// new associated types in the future, and this is [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// Note that using derives directly from `musli_core` requires you to use the
/// `#[musli_core::visitor(crate = musli_core)]` attribute.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
/// [`Visitor`]: crate::de::Visitor
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli_core::Context;
/// use musli_core::de::Visitor;
///
/// struct AnyVisitor;
///
/// #[musli_core::de::visitor(crate = musli_core)]
/// impl<'de, C> Visitor<'de, C> for AnyVisitor
/// where
///     C: Context,
/// {
///     type Ok = ();
///
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(
///             f,
///             "a value that can be decoded into dynamic container"
///         )
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_macros::visitor;

/// This is an attribute macro that must be used when implementing a
/// [`UnsizedVisitor`].
///
/// It is required to use because a [`UnsizedVisitor`] implementation might
/// introduce new associated types in the future, and this is [not yet
/// supported] on a language level in Rust. So this attribute macro polyfills
/// any missing types automatically.
///
/// Note that using derives directly from `musli_core` requires you to use the
/// `#[musli_core::visitor(crate = musli_core)]` attribute.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
/// [`UnsizedVisitor`]: crate::de::UnsizedVisitor
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli_core::Context;
/// use musli_core::de::UnsizedVisitor;
///
/// struct Visitor;
///
/// #[musli_core::de::unsized_visitor(crate = musli_core)]
/// impl<'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor
/// where
///     C: Context,
/// {
///     type Ok = ();
///
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(
///             f,
///             "a reference of bytes"
///         )
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_macros::unsized_visitor;

#[doc(inline)]
pub use self::traits::*;

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
pub mod traits {
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
