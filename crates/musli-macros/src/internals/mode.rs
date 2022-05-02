//! Helper for determining the mode we're currently in.

use crate::internals::tokens::Tokens;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};

#[derive(Clone, Copy)]
pub(crate) enum ModePath<'a> {
    Ident(&'a syn::Ident),
    Path(&'a syn::ExprPath),
}

impl ToTokens for ModePath<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            ModePath::Ident(ident) => {
                ident.to_tokens(tokens);
            }
            ModePath::Path(path) => {
                path.to_tokens(tokens);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Mode<'a> {
    pub(crate) ident: Option<&'a syn::ExprPath>,
    pub(crate) mode_path: ModePath<'a>,
    pub(crate) tokens: &'a Tokens,
}

impl<'a> Mode<'a> {
    /// Get the mode identifier.
    pub(crate) fn mode_ident(&self) -> ModePath<'a> {
        self.mode_path
    }

    /// Construct a typed encode call.
    pub(crate) fn encode_t_encode(&self, span: Span) -> TokenStream {
        let moded_ident = &self.mode_path;
        let encode_t = &self.tokens.encode_t;
        quote_spanned!(span => #encode_t::<#moded_ident>::encode)
    }

    /// Construct a typed encode call.
    pub(crate) fn decode_t_decode(&self, span: Span) -> TokenStream {
        let moded_ident = &self.mode_path;
        let decode_t = &self.tokens.decode_t;
        quote_spanned!(span => #decode_t::<#moded_ident>::decode)
    }
}
