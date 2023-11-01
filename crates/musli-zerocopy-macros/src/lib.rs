//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy-macros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy-macros)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy--macros-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy-macros)
//!
//! This crate provides the macros used in [Müsli zero-copy].
//!
//! Please refer to <https://docs.rs/musli> for documentation.
//!
//! [Müsli zero-copy]: <https://docs.rs/musli-zerocopy>

#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_late_init)]

use proc_macro::TokenStream;

#[cfg(feature = "sneaky-fields")]
mod sneaky_fields;
mod visit;
mod zero_copy;

#[proc_macro_derive(ZeroCopy, attributes(zero_copy))]
pub fn zero_copy(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let expander = zero_copy::Expander::new(input);

    match expander.expand() {
        Ok(stream) => stream.into(),
        Err(errors) => to_compile_errors(errors).into(),
    }
}

#[proc_macro_derive(Visit, attributes(visit))]
pub fn visit(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let expander = visit::Expander::new(&input);

    match expander.expand() {
        Ok(stream) => stream.into(),
        Err(errors) => to_compile_errors(errors).into(),
    }
}

// NB: Only used in UI tests.
#[proc_macro_attribute]
#[doc(hidden)]
#[cfg(feature = "sneaky-fields")]
pub fn sneaky_fields(attr: TokenStream, item: TokenStream) -> TokenStream {
    sneaky_fields::expand(attr, item)
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    for e in errors {
        output.extend(e.to_compile_error());
    }

    output
}
