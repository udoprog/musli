//! Traits for generically dealing with an encoding framework.
//!
//! The central traits are [Encode] and [Encoder].
//!
//! A type implementing [Encode] can use an [Encoder] to encode itself. This
//! also comes with a derive allowing you to derive high performance encoding
//! associated with native Rust types.
//!
//! Note that using derives directly from `musli_core` requires you to use the
//! `#[musli(crate = musli_core)]` attribute.
//!
//! # Examples
//!
//! ```
//! use musli_core::Encode;
//!
//! #[derive(Encode)]
//! #[musli(crate = musli_core)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

/// Derive which automatically implements the [`Encode` trait].
///
/// See the [`derives` module] for detailed documentation.
///
/// [`derives` module]: <https://docs.rs/musli/latest/musli/_help/derives/index.html>
/// [`Encode` trait]: <https://docs.rs/musli/latest/musli/trait.Encode.html>
///
/// # Examples
///
/// ```
/// use musli::Encode;
///
/// #[derive(Encode)]
/// struct MyType {
///     data: [u8; 128],
/// }
/// ```
///
/// When using through `musli_core`, the crate needs to be specified:
///
/// ```
/// use musli_core::Encode;
///
/// #[derive(Encode)]
/// #[musli(crate = musli_core)]
/// struct MyType {
///     data: [u8; 128],
/// }
/// ```
#[doc(inline)]
pub use musli_macros::Encode;

/// This is an attribute macro that must be used when implementing a
/// [`Encoder`].
///
/// It is required to use because a [`Encoder`] implementation might introduce
/// new associated types in the future, and this [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
///
/// Note that if the `Cx` or `Mode` associated types are not specified, they
/// will be defaulted to any type parameters which starts with the uppercase `C`
/// or `M` respectively.
///
/// Note that using this macro directly from `musli_core` requires you to use
/// the `#[musli_core::encoder(crate = musli_core)]` attribute.
///
/// # Examples
///
/// ```
/// use std::fmt;
/// use std::marker::PhantomData;
///
/// use musli_core::Context;
/// use musli_core::en::{Encoder, Encode};
///
/// struct MyEncoder<'a, C, M> {
///     value: &'a mut Option<u32>,
///     cx: C,
///     _marker: PhantomData<M>,
/// }
///
/// #[musli_core::encoder(crate = musli_core)]
/// impl<C, M> Encoder for MyEncoder<'_, C, M>
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
///     fn encode<T>(self, value: T) -> Result<(), C::Error>
///     where
///         T: Encode<Self::Mode>,
///     {
///         value.encode(self)
///     }
///
///     #[inline]
///     fn encode_u32(self, value: u32) -> Result<(), Self::Error> {
///         *self.value = Some(value);
///         Ok(())
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_macros::encoder;

#[doc(inline)]
pub use self::__traits::*;

mod encode;
mod encode_bytes;
mod encode_packed;
mod encode_trace;
mod encoder;
mod entries_encoder;
mod entry_encoder;
mod map_encoder;
mod sequence_encoder;
mod variant_encoder;

#[doc(hidden)]
pub mod __traits {
    pub use super::encode::Encode;
    pub use super::encode_bytes::EncodeBytes;
    pub use super::encode_packed::EncodePacked;
    pub use super::encode_trace::EncodeTrace;
    pub use super::encoder::{Encoder, TryFastEncode};
    pub use super::entries_encoder::EntriesEncoder;
    pub use super::entry_encoder::EntryEncoder;
    pub use super::map_encoder::MapEncoder;
    pub use super::sequence_encoder::SequenceEncoder;
    pub use super::variant_encoder::VariantEncoder;
}

#[doc(hidden)]
pub mod utils;
