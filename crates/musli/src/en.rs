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
