//! Helper for determining the mode we're currently in.

use core::fmt;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::Token;

use super::attr::{FieldEncoding, ModeKind};
use super::tokens::Import;
use super::Only;
use super::ATTR;

#[derive(Clone, Copy)]
pub(crate) enum ModePath<'a> {
    Ident(&'a syn::Ident),
    Musli(&'a syn::Path, &'a syn::Ident),
}

impl ToTokens for ModePath<'_> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            ModePath::Ident(ident) => {
                ident.to_tokens(tokens);
            }
            ModePath::Musli(base, ident) => {
                base.to_tokens(tokens);
                <Token![::]>::default().to_tokens(tokens);
                tokens.extend([TokenTree::Ident(Ident::new("mode", Span::call_site()))]);
                <Token![::]>::default().to_tokens(tokens);
                ident.to_tokens(tokens);
            }
        }
    }
}

pub(crate) struct Trait<'a> {
    import: Import<'a>,
    mode: ModePath<'a>,
    allocator_ident: Option<syn::Ident>,
}

impl ToTokens for Trait<'_> {
    #[inline]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.import.to_tokens(tokens);
        <Token![::]>::default().to_tokens(tokens);
        <Token![<]>::default().to_tokens(tokens);
        self.mode.to_tokens(tokens);

        if let Some(ident) = &self.allocator_ident {
            <Token![,]>::default().to_tokens(tokens);
            ident.to_tokens(tokens);
        }

        <Token![>]>::default().to_tokens(tokens);
    }
}

pub(crate) struct ImportedMethod<'a> {
    trait_t: Trait<'a>,
    method: &'static str,
}

impl ToTokens for ImportedMethod<'_> {
    #[inline]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.trait_t.to_tokens(tokens);
        <Token![::]>::default().to_tokens(tokens);
        tokens.extend([TokenTree::Ident(Ident::new(self.method, Span::call_site()))]);
    }
}

pub(crate) struct Mode<'a> {
    pub(crate) kind: Option<&'a ModeKind>,
    pub(crate) mode_path: ModePath<'a>,
    pub(crate) encode_packed_t: Import<'a>,
    pub(crate) encode_bytes_t: Import<'a>,
    pub(crate) trace_encode_t: Import<'a>,
    pub(crate) encode_t: Import<'a>,
    pub(crate) decode_packed_t: Import<'a>,
    pub(crate) decode_bytes_t: Import<'a>,
    pub(crate) trace_decode_t: Import<'a>,
    pub(crate) decode_t: Import<'a>,
    pub(crate) only: Only,
}

impl<'a> Mode<'a> {
    /// Construct a typed encode call.
    pub(crate) fn encode_t_encode(&self, encoding: FieldEncoding) -> ImportedMethod<'a> {
        let (encode_t, name) = match encoding {
            FieldEncoding::Packed => (self.encode_packed_t, "encode_packed"),
            FieldEncoding::Bytes => (self.encode_bytes_t, "encode_bytes"),
            FieldEncoding::Trace => (self.trace_encode_t, "trace_encode"),
            FieldEncoding::Default => (self.encode_t, "encode"),
        };

        ImportedMethod {
            trait_t: Trait {
                import: encode_t,
                mode: self.mode_path,
                allocator_ident: None,
            },
            method: name,
        }
    }

    /// Construct a typed encode call.
    pub(crate) fn encode_t_size_hint(&self, encoding: FieldEncoding) -> Option<ImportedMethod<'a>> {
        let (encode_t, name) = match encoding {
            FieldEncoding::Default => (self.encode_t, "size_hint"),
            _ => return None,
        };

        Some(ImportedMethod {
            trait_t: Trait {
                import: encode_t,
                mode: self.mode_path,
                allocator_ident: None,
            },
            method: name,
        })
    }

    /// Get the fully expanded trait.
    pub(crate) fn as_trait_t(&self, allocator_ident: &syn::Ident) -> Trait<'a> {
        match self.only {
            Only::Encode => Trait {
                import: self.encode_t,
                mode: self.mode_path,
                allocator_ident: None,
            },
            Only::Decode => Trait {
                import: self.decode_t,
                mode: self.mode_path,
                allocator_ident: Some(allocator_ident.clone()),
            },
        }
    }

    /// Construct a typed decode call.
    pub(crate) fn decode_t_decode(
        &self,
        encoding: FieldEncoding,
        allocator_ident: &syn::Ident,
    ) -> ImportedMethod<'a> {
        let (decode_t, name) = match encoding {
            FieldEncoding::Packed => (self.decode_packed_t, "decode_packed"),
            FieldEncoding::Bytes => (self.decode_bytes_t, "decode_bytes"),
            FieldEncoding::Trace => (self.trace_decode_t, "trace_decode"),
            FieldEncoding::Default => (self.decode_t, "decode"),
        };

        ImportedMethod {
            trait_t: Trait {
                import: decode_t,
                mode: self.mode_path,
                allocator_ident: Some(allocator_ident.clone()),
            },
            method: name,
        }
    }
}

impl fmt::Display for Mode<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.mode_path {
            ModePath::Ident(ident) => write!(f, "#[{ATTR}(mode = {ident}, ..)]"),
            ModePath::Musli(_, ident) => write!(f, "#[{ATTR}({ident}, ..)]"),
        }
    }
}
