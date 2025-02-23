//! Traits for generically dealing with an encoding framework.
//!
//! The central traits are [Encode] and [Encoder].
//!
//! A type implementing [Encode] can use an [Encoder] to encode itself. This
//! also comes with a derive allowing you to derive high performance encoding
//! associated with native Rust types.
//!
//! ```
//! use musli::Encode;
//!
//! #[derive(Encode)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

/// Derive which automatically implements the [`Encode` trait].
///
/// See the [`derives` module] for detailed documentation.
///
/// [`derives` module]: crate::_help::derives
/// [`Encode` trait]: trait@Encode
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
pub use musli_core::__macros::Encode;

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
/// use musli::Context;
/// use musli::en::{Encoder, Encode};
///
/// struct MyEncoder<'a, C, M> {
///     value: &'a mut Option<u32>,
///     cx: C,
///     _marker: PhantomData<M>,
/// }
///
/// #[musli::encoder]
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
pub use musli_core::__macros::encoder;

#[doc(inline)]
pub use musli_core::en::__traits::{
    Encode, EncodeBytes, EncodePacked, EncodeTrace, Encoder, EntriesEncoder, EntryEncoder,
    MapEncoder, SequenceEncoder, TryFastEncode, VariantEncoder,
};

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
pub(crate) use musli_core::en::utils;
