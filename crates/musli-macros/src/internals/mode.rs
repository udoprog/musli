//! Helper for determining the mode we're currently in.

use syn::spanned::Spanned;

use crate::internals::tokens::Tokens;
use crate::internals::Only;

#[derive(Clone, Copy)]
pub(crate) struct Mode<'a> {
    pub(crate) ident: Option<&'a syn::Path>,
    pub(crate) tokens: &'a Tokens,
    pub(crate) only: Only,
}

impl<'a> Mode<'a> {
    /// Construct a typed encode call.
    pub(crate) fn encode_t_encode(&self, trace: bool) -> syn::Path {
        let (mut encode_t, name) = if trace {
            (self.tokens.trace_encode_t.clone(), "trace_encode")
        } else {
            (self.tokens.encode_t.clone(), "encode")
        };

        encode_t
            .segments
            .push(syn::PathSegment::from(syn::Ident::new(
                name,
                encode_t.span(),
            )));
        encode_t
    }

    /// Construct a typed encode call.
    pub(crate) fn decode_t_decode(&self, trace: bool) -> syn::Path {
        let (mut decode_t, method) = if trace {
            (self.tokens.trace_decode_t.clone(), "trace_decode")
        } else {
            (self.tokens.decode_t.clone(), "decode")
        };

        decode_t
            .segments
            .push(syn::PathSegment::from(syn::Ident::new(
                method,
                decode_t.span(),
            )));

        decode_t
    }
}
