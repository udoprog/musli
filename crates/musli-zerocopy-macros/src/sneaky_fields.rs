use proc_macro::TokenStream;
use quote::ToTokens;

/// Replace the fields of a struct.
pub(super) fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::Item);
    let ty = syn::parse_macro_input!(attr as syn::Type);

    match &mut input {
        syn::Item::Enum(e) => {
            for v in &mut e.variants {
                match &mut v.fields {
                    syn::Fields::Named(named) => {
                        let extra: syn::FieldsNamed = syn::parse_quote!({ sneaky_field: #ty });
                        named.named.extend(extra.named);
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        let extra: syn::FieldsUnnamed = syn::parse_quote!(( #ty ));
                        unnamed.unnamed.extend(extra.unnamed);
                    }
                    syn::Fields::Unit => {
                        v.fields = syn::Fields::Named(syn::parse_quote!({ sneaky_field: #ty }));
                    }
                }
            }
        }
        syn::Item::Struct(st) => match &mut st.fields {
            syn::Fields::Named(named) => {
                let extra: syn::FieldsNamed = syn::parse_quote!({ sneaky_field: #ty });
                named.named.extend(extra.named);
            }
            syn::Fields::Unnamed(unnamed) => {
                let extra: syn::FieldsUnnamed = syn::parse_quote!(( #ty ));
                unnamed.unnamed.extend(extra.unnamed);
            }
            syn::Fields::Unit => {
                st.fields = syn::Fields::Named(syn::parse_quote!({ sneaky_field: #ty }));
            }
        },
        _ => (),
    }

    input.to_token_stream().into()
}
