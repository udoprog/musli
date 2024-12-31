//! Helper for determining the mode we're currently in.

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::Token;

use super::attr::{FieldEncoding, ModeKind};
use super::tokens::Import;
use super::Only;

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

pub(crate) struct Method<'a>(Import<'a>, ModePath<'a>, &'static str);

impl ToTokens for Method<'_> {
    #[inline]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Method(import, mode_path, name) = *self;

        import.to_tokens(tokens);
        <Token![::]>::default().to_tokens(tokens);
        <Token![<]>::default().to_tokens(tokens);
        mode_path.to_tokens(tokens);
        <Token![>]>::default().to_tokens(tokens);
        <Token![::]>::default().to_tokens(tokens);
        tokens.extend([TokenTree::Ident(Ident::new(name, Span::call_site()))]);
    }
}

#[derive(Clone, Copy)]
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
    pub(crate) fn encode_t_encode(&self, encoding: FieldEncoding) -> Method<'a> {
        let (encode_t, name) = match encoding {
            FieldEncoding::Packed => (self.encode_packed_t, "encode_packed"),
            FieldEncoding::Bytes => (self.encode_bytes_t, "encode_bytes"),
            FieldEncoding::Trace => (self.trace_encode_t, "trace_encode"),
            FieldEncoding::Default => (self.encode_t, "encode"),
        };

        Method(encode_t, self.mode_path, name)
    }

    /// Construct a typed decode call.
    pub(crate) fn decode_t_decode(&self, encoding: FieldEncoding) -> Method<'a> {
        let (decode_t, name) = match encoding {
            FieldEncoding::Packed => (self.decode_packed_t, "decode_packed"),
            FieldEncoding::Bytes => (self.decode_bytes_t, "decode_bytes"),
            FieldEncoding::Trace => (self.trace_decode_t, "trace_decode"),
            FieldEncoding::Default => (self.decode_t, "decode"),
        };

        Method(decode_t, self.mode_path, name)
    }
}
