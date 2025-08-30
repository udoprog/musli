//! Traits for generically dealing with a decoding framework.
//!
//! The central traits are [Decode] and [Decoder].
//!
//! A type implementing [Decode] can use an [Decoder] to decode an instance of
//! itself. This also comes with a derive allowing you to derive high
//! performance decoding associated with native Rust types.
//!
//! ```
//! use musli::Decode;
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
/// [`derives` module]: crate::_help::derives
/// [`Decode` trait]: trait@Decode
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
#[doc(inline)]
pub use musli_core::__macros::Decode;

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
/// # Examples
///
/// ```
/// use std::fmt;
/// use std::marker::PhantomData;
///
/// use musli::Context;
/// use musli::de::{Decoder, Decode};
///
/// struct MyDecoder<C, M> {
///     cx: C,
///     _marker: PhantomData<M>,
/// }
///
/// #[musli::decoder]
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
pub use musli_core::__macros::musli_decoder as decoder;

/// This is an attribute macro that must be used when implementing a
/// [`Visitor`].
///
/// It is required to use because a [`Visitor`] implementation might introduce
/// new associated types in the future, and this is [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
/// [`Visitor`]: crate::de::Visitor
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::Visitor;
///
/// struct AnyVisitor;
///
/// #[musli::visitor]
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
pub use musli_core::__macros::musli_visitor as visitor;

/// This is an attribute macro that must be used when implementing a
/// [`UnsizedVisitor`].
///
/// It is required to use because a [`UnsizedVisitor`] implementation might
/// introduce new associated types in the future, and this is [not yet
/// supported] on a language level in Rust. So this attribute macro polyfills
/// any missing types automatically.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
/// [`UnsizedVisitor`]: crate::de::UnsizedVisitor
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::UnsizedVisitor;
///
/// struct Visitor;
///
/// #[musli::unsized_visitor]
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
pub use musli_core::__macros::musli_unsized_visitor as unsized_visitor;

#[doc(inline)]
pub use musli_core::de::__traits::{
    AsDecoder, Decode, DecodeBytes, DecodeOwned, DecodePacked, DecodeSliceBuilder, DecodeTrace,
    DecodeUnsized, DecodeUnsizedBytes, Decoder, EntriesDecoder, EntryDecoder, MapDecoder,
    SequenceDecoder, SizeHint, Skip, TryFastDecode, UnsizedVisitor, VariantDecoder, Visitor,
};

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
pub(crate) use musli_core::de::utils;
