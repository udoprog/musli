//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-core.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-core)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--core-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-core)
//!
//! Core traits for [Müsli].
//!
//! [Müsli]: https://docs.rs/musli

#![deny(missing_docs)]
#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc as rust_alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod alloc;
#[doc(inline)]
pub use self::alloc::Allocator;

mod context;
#[doc(inline)]
pub use self::context::Context;

pub mod de;
#[doc(inline)]
pub use self::de::{Decode, Decoder};

pub mod en;
#[doc(inline)]
pub use self::en::{Encode, Encoder};

pub mod hint;
pub mod mode;
pub mod no_std;

mod expecting;
mod impls;
mod internal;
mod never;

/// This is an attribute macro that must be used when implementing a
/// [`Encoder`].
///
/// It is required to use because a [`Encoder`] implementation might introduce
/// new associated types in the future, and this [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// Note that using derives directly from `musli_core` requires you to use the
/// `#[musli_core::encoder(crate = musli_core)]` attribute.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli_core::Context;
/// use musli_core::en::{Encoder, Encode};
///
/// struct MyEncoder<'a, C> {
///     value: &'a mut Option<u32>,
///     cx: C,
/// }
///
/// #[musli_core::encoder(crate = musli_core)]
/// impl<C> Encoder for MyEncoder<'_, C>
/// where
///     C: Context,
/// {
///     type Cx = C;
///     type Ok = ();
///
///     #[inline]
///     fn cx(&self) -> Self::Cx {
///         self.cx
///     }
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn encode<T>(self, value: T) -> Result<Self::Ok, C::Error>
///     where
///         T: Encode<Self::Mode>,
///     {
///         value.encode(self)
///     }
///
///     fn encode_u32(self, value: u32) -> Result<(), Self::Error> {
///         *self.value = Some(value);
///         Ok(())
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_macros::encoder;

/// This is an attribute macro that must be used when implementing a
/// [`Decoder`].
///
/// It is required to use because a [`Decoder`] implementation might introduce
/// new associated types in the future, and this is [not yet supported] on a
/// language level in Rust. So this attribute macro polyfills any missing types
/// automatically.
///
/// Note that using derives directly from `musli_core` requires you to use the
/// `#[musli_core::decoder(crate = musli_core)]` attribute.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use musli_core::Context;
/// use musli_core::de::{Decoder, Decode};
///
/// struct MyDecoder<C> {
///     cx: C,
/// }
///
/// #[musli_core::decoder(crate = musli_core)]
/// impl<'de, C> Decoder<'de> for MyDecoder<C>
/// where
///     C: Context,
/// {
///     type Cx = C;
///
///     #[inline]
///     fn cx(&self) -> Self::Cx {
///         self.cx
///     }
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
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
/// #[musli_core::visitor(crate = musli_core)]
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
///             "value that can be decoded into dynamic container"
///         )
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_macros::visitor;

/// Internal implementation details of musli.
///
/// Using these directly is not supported.
#[doc(hidden)]
pub mod __priv {
    pub use crate::alloc::Allocator;
    pub use crate::context::Context;
    pub use crate::de::{
        AsDecoder, Decode, DecodeBytes, DecodePacked, DecodeTrace, Decoder, EntryDecoder,
        MapDecoder, SequenceDecoder, TryFastDecode, VariantDecoder,
    };
    pub use crate::en::{
        Encode, EncodeBytes, EncodePacked, EncodeTrace, Encoder, EntryEncoder, MapEncoder,
        SequenceEncoder, TryFastEncode, VariantEncoder,
    };
    pub use crate::hint::MapHint;
    pub use crate::never::Never;

    pub use ::core::fmt;
    pub use ::core::mem::{needs_drop, offset_of, size_of};
    pub use ::core::option::Option;
    pub use ::core::result::Result;

    #[inline(always)]
    pub fn default<T>() -> T
    where
        T: ::core::default::Default,
    {
        ::core::default::Default::default()
    }

    /// Note that this returns `true` if skipping was unsupported.
    #[inline(always)]
    pub fn skip<'de, D>(decoder: D) -> Result<bool, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(decoder.try_skip()?.is_unsupported())
    }

    /// Note that this returns `true` if skipping was unsupported.
    #[inline(always)]
    pub fn skip_field<'de, D>(decoder: D) -> Result<bool, <D::Cx as Context>::Error>
    where
        D: EntryDecoder<'de>,
    {
        skip(decoder.decode_value()?)
    }
}
