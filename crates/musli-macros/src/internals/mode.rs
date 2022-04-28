//! Helper for determining the mode we're currently in.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Clone, Copy)]
pub(crate) enum ModeIdent<'a> {
    Stream(&'a TokenStream),
    Ident(&'a syn::Ident),
}

impl ToTokens for ModeIdent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            ModeIdent::Stream(stream) => {
                tokens.extend(stream.clone());
            }
            ModeIdent::Ident(ident) => {
                ident.to_tokens(tokens);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Mode<'a> {
    pub(crate) ident: Option<&'a syn::Ident>,
    pub(crate) moded_ident: ModeIdent<'a>,
    pub(crate) encode_t: &'a TokenStream,
    pub(crate) decode_t: &'a TokenStream,
}

impl<'a> Mode<'a> {
    /// Get the mode identifier.
    pub(crate) fn mode_ident(&self) -> ModeIdent<'_> {
        self.moded_ident
    }

    /// Construct a typed encode call.
    pub(crate) fn encode_t_encode(&self) -> TokenStream {
        let moded_ident = &self.moded_ident;
        let encode_t = &self.encode_t;
        quote!(#encode_t::<#moded_ident>::encode)
    }

    /// Construct a typed encode call.
    pub(crate) fn decode_t_decode(&self) -> TokenStream {
        let moded_ident = &self.moded_ident;
        let decode_t = &self.decode_t;
        quote!(#decode_t::<#moded_ident>::decode)
    }
}
