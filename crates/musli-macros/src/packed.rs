use proc_macro2::Span;
use quote::quote;

use crate::internals::attr::{Packed, Packing};
use crate::internals::build::{Body, Build};
use crate::internals::tokens::Tokens;

pub(super) fn packed(
    e: &Build<'_>,
    st: &Body<'_>,
    trait_t: &syn::Path,
    packed_field: &str,
) -> syn::Expr {
    let Tokens {
        offset_of, size_of, ..
    } = e.tokens;

    match st.packing {
        Packing::Packed(Packed::Bitwise) if st.all_fields.len() == st.unskipped_fields.len() => {
            let packed_field = syn::Ident::new(packed_field, Span::call_site());
            let mode_ident = e.expansion.mode_path(e.tokens).as_path();

            let mut offsets = Vec::with_capacity(st.all_fields.len().saturating_sub(1));
            let mut sizes = Vec::with_capacity(st.all_fields.len());
            let mut packed = Vec::with_capacity(st.all_fields.len());

            for w in st.all_fields.windows(2) {
                let [a, b] = w else {
                    continue;
                };

                let a = &a.member;
                let b = &b.member;

                offsets.push(quote!(#offset_of!(Self, #a) <= #offset_of!(Self, #b)));
            }

            for f in &st.all_fields {
                let ty = &f.ty;
                sizes.push(quote!(#size_of::<#ty>()));
                packed.push(quote!(<#ty as #trait_t<#mode_ident>>::#packed_field));
            }

            syn::parse_quote!(#size_of::<Self>() == (0 #(+ #sizes)*) #(&& #offsets)* #(&& #packed)*)
        }
        _ => {
            syn::parse_quote!(false)
        }
    }
}
