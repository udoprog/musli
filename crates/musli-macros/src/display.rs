//! Expansion of `#[musli(display)]`, which encodes a type through its
//! `Display` implementation and decodes it through `FromStr`.

use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::expander::Expander;
use crate::internals::{ATTR, Result, Tokens};

/// Test if the type is `#[musli(display)]`, reporting attributes which can not
/// be combined with it.
pub(crate) fn is_display(e: &Expander<'_>) -> Result<bool> {
    let mut display = None;
    let mut others = Vec::new();

    e.type_attr.each_attribute(|name, span, scoped| {
        if name == "display" {
            if scoped {
                e.cx.error_span(
                    span,
                    format_args!(
                        "#[{ATTR}(display)] applies to every mode, and cannot be combined with \
                         #[{ATTR}(mode = ..)], #[{ATTR}(encode_only)] or #[{ATTR}(decode_only)]"
                    ),
                );
            }

            display = Some(span);
        } else if name != "krate" {
            others.push(span);
        }
    });

    if display.is_none() {
        return Ok(false);
    }

    for span in others {
        e.cx.error_span(
            span,
            format_args!(
                "#[{ATTR}(display)] cannot be combined with other type attributes, except for \
                 #[{ATTR}(crate)]"
            ),
        );
    }

    if e.cx.has_errors() {
        return Err(());
    }

    Ok(true)
}

/// Generic parameters which are introduced by the implementations.
struct Params {
    lt: syn::Lifetime,
    mode: syn::Ident,
    allocator: syn::Ident,
    encoder: syn::Ident,
    decoder: syn::Ident,
}

impl Params {
    fn new() -> Self {
        let ident = |name: &str| syn::Ident::new(name, Span::call_site());

        Self {
            lt: syn::Lifetime::new("'__de", Span::call_site()),
            mode: ident("__M"),
            allocator: ident("__A"),
            encoder: ident("__E"),
            decoder: ident("__D"),
        }
    }
}

pub(crate) fn expand_encode(e: &Expander<'_>, tokens: &Tokens<'_>) -> TokenStream {
    let Tokens {
        display_encode,
        encode_t,
        encoder_t,
        fmt,
        result,
        ..
    } = tokens;

    let Params {
        mode,
        encoder: e_param,
        ..
    } = Params::new();

    let ident = &e.input.ident;

    let mut generics = e.input.generics.clone();
    generics.params.push(syn::parse_quote!(#mode));
    generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote!(Self: #fmt::Display));

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, type_generics, _) = e.input.generics.split_for_impl();

    quote! {
        const _: () = {
            #[automatically_derived]
            impl #impl_generics #encode_t<#mode> for #ident #type_generics
            #where_clause
            {
                type Encode = Self;

                const IS_BITWISE_ENCODE: bool = false;

                #[inline]
                fn encode<#e_param>(&self, encoder: #e_param) -> #result<(), <#e_param as #encoder_t>::Error>
                where
                    #e_param: #encoder_t<Mode = #mode>,
                {
                    #display_encode(self, encoder)
                }

                #[inline]
                fn as_encode(&self) -> &Self::Encode {
                    self
                }
            }
        };
    }
}

pub(crate) fn expand_decode(e: &Expander<'_>, tokens: &Tokens<'_>) -> TokenStream {
    let Tokens {
        allocator_t,
        decode_t,
        decoder_t,
        display_decode,
        fmt,
        from_str_t,
        result,
        ..
    } = tokens;

    let Params {
        lt,
        mode,
        allocator,
        decoder: d_param,
        ..
    } = Params::new();

    let ident = &e.input.ident;

    let mut generics = e.input.generics.clone();
    generics.params.insert(0, syn::parse_quote!(#lt));
    generics.params.push(syn::parse_quote!(#mode));
    generics.params.push(syn::parse_quote!(#allocator));

    let predicates = &mut generics.make_where_clause().predicates;
    predicates.push(syn::parse_quote!(#allocator: #allocator_t));
    predicates.push(syn::parse_quote!(Self: #from_str_t));
    predicates.push(syn::parse_quote!(<Self as #from_str_t>::Err: #fmt::Display));

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, type_generics, _) = e.input.generics.split_for_impl();

    quote! {
        const _: () = {
            #[automatically_derived]
            impl #impl_generics #decode_t<#lt, #mode, #allocator> for #ident #type_generics
            #where_clause
            {
                const IS_BITWISE_DECODE: bool = false;

                #[inline]
                fn decode<#d_param>(decoder: #d_param) -> #result<Self, <#d_param as #decoder_t<#lt>>::Error>
                where
                    #d_param: #decoder_t<#lt, Mode = #mode, Allocator = #allocator>,
                {
                    #display_decode(decoder)
                }
            }
        };
    }
}
