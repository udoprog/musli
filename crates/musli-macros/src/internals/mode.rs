//! Helper for determining the mode we're currently in.

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use super::attr::{FieldEncoding, ModeKind};
use super::tokens::Tokens;
use super::Only;

#[derive(Clone, Copy)]
pub(crate) enum ModePath<'a> {
    Ident(&'a syn::Ident),
    Musli(&'a syn::Path, &'a syn::Ident),
}

impl ModePath<'_> {
    pub(crate) fn as_path(self) -> syn::Path {
        match self {
            ModePath::Ident(ident) => syn::Path::from(ident.clone()),
            ModePath::Musli(base, ident) => {
                let mut path = base.clone();
                path.segments.push(syn::PathSegment::from(syn::Ident::new(
                    "mode",
                    Span::call_site(),
                )));
                path.segments.push(syn::PathSegment::from(ident.clone()));
                path
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Mode<'a> {
    pub(crate) kind: Option<&'a ModeKind>,
    pub(crate) mode_path: ModePath<'a>,
    pub(crate) tokens: &'a Tokens,
    pub(crate) only: Only,
}

impl<'a> Mode<'a> {
    /// Construct a typed encode call.
    pub(crate) fn encode_t_encode(&self, encoding: FieldEncoding) -> syn::Path {
        let (mut encode_t, name) = match encoding {
            FieldEncoding::Packed => (self.tokens.encode_packed_t.clone(), "encode_packed"),
            FieldEncoding::Bytes => (self.tokens.encode_bytes_t.clone(), "encode_bytes"),
            FieldEncoding::Trace => (self.tokens.trace_encode_t.clone(), "trace_encode"),
            FieldEncoding::Default => (self.tokens.encode_t.clone(), "encode"),
        };

        if let Some(segment) = encode_t.segments.last_mut() {
            add_mode_argument(&self.mode_path, segment);
        }

        encode_t
            .segments
            .push(syn::PathSegment::from(syn::Ident::new(
                name,
                encode_t.span(),
            )));

        encode_t
    }

    /// Construct a typed decode call.
    pub(crate) fn decode_t_decode(&self, encoding: FieldEncoding) -> syn::Path {
        let (mut decode_t, name) = match encoding {
            FieldEncoding::Packed => (self.tokens.decode_packed_t.clone(), "decode_packed"),
            FieldEncoding::Bytes => (self.tokens.decode_bytes_t.clone(), "decode_bytes"),
            FieldEncoding::Trace => (self.tokens.trace_decode_t.clone(), "trace_decode"),
            FieldEncoding::Default => (self.tokens.decode_t.clone(), "decode"),
        };

        if let Some(segment) = decode_t.segments.last_mut() {
            add_mode_argument(&self.mode_path, segment);
        }

        decode_t
            .segments
            .push(syn::PathSegment::from(syn::Ident::new(
                name,
                decode_t.span(),
            )));

        decode_t
    }
}

fn add_mode_argument(moded_ident: &ModePath<'_>, last: &mut syn::PathSegment) {
    let mut arguments = syn::AngleBracketedGenericArguments {
        colon2_token: Some(<Token![::]>::default()),
        lt_token: <Token![<]>::default(),
        args: Punctuated::default(),
        gt_token: <Token![>]>::default(),
    };

    arguments
        .args
        .push(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
            qself: None,
            path: moded_ident.as_path(),
        })));

    last.arguments = syn::PathArguments::AngleBracketed(arguments);
}
