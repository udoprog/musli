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

mod expecting;
mod impls;
mod internal;
mod never;
/// Automatic impl of Encode for Valid and Unvalidated types, and Decode for Valid types.
mod garde022;
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

#[doc(hidden)]
pub use musli_macros as __macros;

/// This is an attribute macro that must be used when implementing the following traits:
///
/// * [`Decoder`]
/// * [`de::Visitor`][crate::de::Visitor]
/// * [`de::UnsizedVisitor`][crate::de::UnsizedVisitor]
/// * [`Encoder`]
///
/// It is required to use because these traits might introduce new associated
/// types in the future, and this is [not yet supported] on a language level in
/// Rust. So this attribute macro polyfills any missing types automatically.
///
/// [not yet supported]: https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
///
/// Note that if the `Cx` or `Mode` associated types are not specified, they
/// will be defaulted to any type parameters which starts with the uppercase `C`
/// or `M` respectively if the trait uses them.
///
/// # Examples
///
/// Implementing `Decoder`:
///
/// ```
/// use std::fmt;
/// use std::marker::PhantomData;
///
/// use musli_core::Context;
/// use musli_core::de::Decoder;
///
/// struct MyDecoder<C, M> {
///     cx: C,
///     _marker: PhantomData<M>,
/// }
///
/// #[musli_core::trait_defaults]
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
///
/// Implementing `UnsizedVisitor`:
///
/// ```
/// use std::fmt;
///
/// use musli_core::Context;
/// use musli_core::de::UnsizedVisitor;
///
/// struct MyVisitor;
///
/// #[musli_core::trait_defaults]
/// impl<'de, C> UnsizedVisitor<'de, C, [u8]> for MyVisitor
/// where
///     C: Context,
/// {
///     type Ok = ();
///
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "a reference of bytes")
///     }
/// }
/// ```
///
/// Implementing `Visitor`:
///
/// ```
/// use std::fmt;
///
/// use musli_core::Context;
/// use musli_core::de::Visitor;
///
/// struct MyVisitor;
///
/// #[musli_core::trait_defaults]
/// impl<'de, C> Visitor<'de, C> for MyVisitor
/// where
///     C: Context,
/// {
///     type Ok = ();
///
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "a value that can be decoded into dynamic container")
///     }
/// }
/// ```
///
/// Implementing `Encoder`:
///
/// ```
/// use std::fmt;
/// use std::marker::PhantomData;
///
/// use musli_core::Context;
/// use musli_core::en::Encoder;
///
/// struct MyEncoder<'a, C, M> {
///     value: &'a mut Option<u32>,
///     cx: C,
///     _marker: PhantomData<M>,
/// }
///
/// #[musli_core::trait_defaults]
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
///     fn encode_u32(self, value: u32) -> Result<(), Self::Error> {
///         *self.value = Some(value);
///         Ok(())
///     }
/// }
/// ```
#[doc(inline)]
pub use musli_macros::musli_core_trait_defaults as trait_defaults;

/// Internal implementation details of musli.
///
/// Using these directly is not supported.
#[doc(hidden)]
pub mod __priv {
    use core::marker::PhantomData;

    pub use crate::alloc::Allocator;
    use crate::alloc::String;
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

    #[inline]
    pub fn default<T>() -> T
    where
        T: ::core::default::Default,
    {
        ::core::default::Default::default()
    }

    /// Note that this returns `true` if skipping was unsupported.
    #[inline]
    pub fn skip<'de, D>(decoder: D) -> Result<bool, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(decoder.try_skip()?.is_unsupported())
    }

    /// Note that this returns `true` if skipping was unsupported.
    #[inline]
    pub fn skip_field<'de, D>(decoder: D) -> Result<bool, D::Error>
    where
        D: EntryDecoder<'de>,
    {
        skip(decoder.decode_value()?)
    }

    /// Collect and allocate a string from a [`Display`] implementation.
    ///
    /// [`Display`]: fmt::Display
    #[inline]
    pub fn collect_string<C>(
        cx: C,
        value: impl fmt::Display,
    ) -> Result<String<C::Allocator>, C::Error>
    where
        C: Context,
    {
        match crate::alloc::collect_string(cx.alloc(), value) {
            Ok(string) => Ok(string),
            Err(error) => Err(cx.message(error)),
        }
    }

    /// Construct a map hint from an `Encode` implementation.
    #[inline]
    pub fn map_hint<M>(encode: &(impl Encode<M> + ?Sized)) -> impl MapHint + '_
    where
        M: 'static,
    {
        EncodeMapHint {
            encode,
            _marker: PhantomData,
        }
    }

    pub(crate) struct EncodeMapHint<'a, T, M>
    where
        T: ?Sized,
    {
        encode: &'a T,
        _marker: PhantomData<M>,
    }

    impl<T, M> MapHint for EncodeMapHint<'_, T, M>
    where
        T: ?Sized + Encode<M>,
    {
        #[inline]
        fn get(self) -> Option<usize> {
            self.encode.size_hint()
        }
    }

    /// Helper methods to report errors.
    pub mod m {
        use core::fmt;

        use crate::Context;

        /// Report that an invalid variant tag was encountered.
        #[inline]
        pub fn invalid_variant_tag<C>(
            cx: C,
            type_name: &'static str,
            tag: impl fmt::Debug,
        ) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} received invalid variant tag {tag:?}"
            ))
        }

        /// The value for the given tag could not be collected.
        #[inline]
        pub fn expected_tag<C>(cx: C, type_name: &'static str, tag: impl fmt::Debug) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!("Type {type_name} expected tag {tag:?}"))
        }

        /// Trying to decode an uninhabitable type.
        #[inline]
        pub fn uninhabitable<C>(cx: C, type_name: &'static str) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} cannot be decoded since it's uninhabitable"
            ))
        }

        /// Encountered an unsupported field tag.
        #[inline]
        pub fn invalid_field_tag<C>(
            cx: C,
            type_name: &'static str,
            tag: impl fmt::Debug,
        ) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} is missing invalid field tag {tag:?}"
            ))
        }

        /// Expected another field to decode.
        #[inline]
        pub fn expected_field_adjacent<C>(
            cx: C,
            type_name: &'static str,
            tag: impl fmt::Debug,
            content: impl fmt::Debug,
        ) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} expected adjacent field {tag:?} or {content:?}"
            ))
        }

        /// Missing adjacent tag when decoding.
        #[inline]
        pub fn missing_adjacent_tag<C>(
            cx: C,
            type_name: &'static str,
            tag: impl fmt::Debug,
        ) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} is missing adjacent tag {tag:?}"
            ))
        }

        /// Encountered an unsupported field tag.
        #[inline]
        pub fn invalid_field_string_tag<C>(
            cx: C,
            type_name: &'static str,
            field: impl fmt::Debug,
        ) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} received invalid field tag {field:?}"
            ))
        }

        /// Missing variant field required to decode.
        #[inline]
        pub fn tagged_enum_unsupported<C>(cx: C, type_name: &'static str) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Encoding format does not supported decoding type {type_name} as a tagged enum"
            ))
        }

        /// Missing variant field required to decode.
        #[inline]
        pub fn missing_variant_field<C>(
            cx: C,
            type_name: &'static str,
            tag: impl fmt::Debug,
        ) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} is missing variant field {tag:?}"
            ))
        }

        /// Encountered an unsupported variant field.
        #[inline]
        pub fn invalid_variant_field_tag<C>(
            cx: C,
            type_name: &'static str,
            variant: impl fmt::Debug,
            tag: impl fmt::Debug,
        ) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!(
                "Type {type_name} received invalid variant field tag {tag:?} for variant {variant:?}",
            ))
        }

        /// Untagged enum could not be decoded.
        #[inline]
        pub fn untagged_mismatch<C>(cx: C, type_name: &'static str) -> C::Error
        where
            C: Context,
        {
            cx.message(format_args!("No variant of {type_name} could be decoded"))
        }
    }
}
