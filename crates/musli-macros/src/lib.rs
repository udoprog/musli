//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-macros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-macros)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--macros-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-macros)
//!
//! This crate provides the macros used in [Müsli].
//!
//! Please refer to <https://docs.rs/musli> for documentation.
//!
//! [Müsli]: <https://docs.rs/musli>

#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_late_init)]
#![allow(missing_docs)]

mod de;
mod en;
mod expander;
mod internals;
mod types;

use proc_macro::TokenStream;

const CRATE_DEFAULT: &str = "musli";

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
#[proc_macro_derive(Encode, attributes(musli))]
pub fn musli_derive_encode(input: TokenStream) -> TokenStream {
    derive_encode(input, CRATE_DEFAULT)
}

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
#[proc_macro_derive(Decode, attributes(musli))]
pub fn musli_derive_decode(input: TokenStream) -> TokenStream {
    derive_decode(input, CRATE_DEFAULT)
}

fn derive_encode(input: TokenStream, crate_default: &str) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let expander = expander::Expander::new(&input, crate_default);

    match expander.expand_encode() {
        Ok(tokens) => tokens.into(),
        Err(()) => to_compile_errors(expander.into_errors()).into(),
    }
}

fn derive_decode(input: TokenStream, crate_default: &str) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let expander = expander::Expander::new(&input, crate_default);

    match expander.expand_decode() {
        Ok(tokens) => tokens.into(),
        Err(()) => to_compile_errors(expander.into_errors()).into(),
    }
}

#[proc_macro_attribute]
pub fn decoder(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as types::Attr);
    let input = syn::parse_macro_input!(input as types::Types);

    match input.expand(
        CRATE_DEFAULT,
        &attr,
        "decoder",
        types::DECODER_TYPES,
        None,
        "__UseMusliDecoderAttributeMacro",
        types::Kind::SelfCx,
    ) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn encoder(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as types::Attr);
    let input = syn::parse_macro_input!(input as types::Types);

    match input.expand(
        CRATE_DEFAULT,
        &attr,
        "encoder",
        types::ENCODER_TYPES,
        Some("Ok"),
        "__UseMusliEncoderAttributeMacro",
        types::Kind::SelfCx,
    ) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn visitor(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as types::Attr);
    let input = syn::parse_macro_input!(input as types::Types);

    match input.expand(
        CRATE_DEFAULT,
        &attr,
        "visitor",
        types::VISITOR_TYPES,
        Some("Ok"),
        "__UseMusliVisitorAttributeMacro",
        types::Kind::GenericCx,
    ) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    for e in errors {
        output.extend(e.to_compile_error());
    }

    output
}
