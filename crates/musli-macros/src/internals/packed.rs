use proc_macro2::Span;
use quote::quote;

use super::attr::Packing;
use super::build::{Body, Build};
use super::{Import, Only, Tokens};

pub(crate) fn packed(
    e: &Build<'_>,
    st: &Body<'_>,
    trait_t: Import<'_>,
    packed_field: &str,
    only: Only,
) -> syn::Expr {
    let Tokens {
        offset_of,
        size_of,
        needs_drop,
        ..
    } = e.tokens;

    let base = match only {
        Only::Encode => st.all_fields.iter().all(|f| f.encode_path.1.is_default()),
        Only::Decode => st.all_fields.iter().all(|f| f.decode_path.1.is_default()),
    };

    match st.packing {
        Packing::Packed if base && st.all_fields.len() == st.unskipped_fields.len() => {
            let packed_field = syn::Ident::new(packed_field, Span::call_site());
            let mode_ident = e.expansion.mode_path(&e.tokens);

            let mut offsets = Vec::with_capacity(st.all_fields.len().saturating_sub(1));
            let mut sizes = Vec::with_capacity(st.all_fields.len());
            let mut packed = Vec::with_capacity(st.all_fields.len());

            // We check that one field *strictly* follow the other. This means
            // that even if we have sneakily introduced fields, they have to be
            // zero-sized to pass this test.
            for w in st.all_fields.windows(2) {
                let [a, b] = w else {
                    continue;
                };

                let ty = a.ty;
                let a = &a.member;
                let b = &b.member;

                offsets.push(
                    quote!(#offset_of!(Self, #a) + #size_of::<#ty>() == #offset_of!(Self, #b)),
                );
            }

            for f in &st.all_fields {
                let ty = &f.ty;
                sizes.push(quote!(#size_of::<#ty>()));
                packed.push(quote!(<#ty as #trait_t<#mode_ident>>::#packed_field));
            }

            syn::parse_quote! {
                const {
                    !#needs_drop::<Self>() && #size_of::<Self>() == (0 #(+ #sizes)*) #(&& #offsets)* #(&& #packed)*
                }
            }
        }
        _ => {
            syn::parse_quote!(false)
        }
    }
}
