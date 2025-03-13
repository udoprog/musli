use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Token;

#[derive(Default)]
pub(super) struct Ctxt {
    pub(super) errors: Vec<syn::Error>,
}

pub(super) fn expand(cx: &mut Ctxt, mut input: syn::DeriveInput) -> Result<TokenStream, ()> {
    let rng = syn::Ident::new("__rng", Span::call_site());
    let generate = syn::Ident::new("Generate", Span::call_site());

    let ident = &input.ident;
    let mut generate_in = None;

    let out = match &input.data {
        syn::Data::Struct(st) => {
            let fields = build_fields(cx, &st.fields, &rng, &generate)?;

            quote! {
                Self {
                    #(#fields,)*
                }
            }
        }
        syn::Data::Enum(en) => {
            let mut variants = Vec::new();
            let mut values = Vec::new();
            let mut totals = Vec::new();
            let mut count = 0usize;

            for (n, variant) in en.variants.iter().enumerate() {
                let mut attrs = Vec::new();
                let mut all = Punctuated::<_, Token![,]>::new();

                // Transport cfg attributes, so we don't have to do that.
                for a in &variant.attrs {
                    if a.path().is_ident("cfg") {
                        attrs.push(a.clone());

                        if let syn::Meta::List(list) = &a.meta {
                            all.push(list.tokens.clone());
                        }
                    }
                }

                if !all.is_empty() {
                    totals.push(quote! {
                        total += usize::from(cfg!(all(#all)));
                    })
                } else {
                    count += 1;
                }

                let fields = build_fields(cx, &variant.fields, &rng, &generate)?;
                let variant = &variant.ident;

                let value = quote!(#ident::#variant {
                    #(#fields,)*
                });

                variants.push(quote! {
                    #(#attrs)*
                    #n => #value
                });

                values.push(quote! {
                    #(#attrs)*
                    {
                        __out(#value);
                    }
                });
            }

            generate_in = Some(quote! {
                fn generate_in<__R, __F>(#rng: &mut __R, mut __out: __F) where __R: rand::Rng, __F: FnMut(Self) {
                    #(#values)*
                }
            });

            quote! {
                let mut total = #count;
                #(#totals;)*

                match #rng.gen_range(0..total) {
                    #(#variants,)*
                    _ => unreachable!(),
                }
            }
        }
        syn::Data::Union(un) => {
            cx.errors.push(syn::Error::new_spanned(
                un.union_token,
                "Unions are not supported",
            ));
            return Err(());
        }
    };

    let types = input
        .generics
        .type_params()
        .map(|t| t.ident.clone())
        .collect::<Vec<_>>();

    let where_clause = input.generics.make_where_clause();

    for t in types {
        where_clause
            .predicates
            .push(syn::parse_quote!(#t: #generate));
    }

    let (impl_generics, type_generics, where_generics) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #generate for #ident #type_generics #where_generics {
            fn generate<__R>(#rng: &mut __R) -> Self where __R: rand::Rng {
                #out
            }

            #generate_in
        }
    })
}

fn build_fields(
    cx: &mut Ctxt,
    fields: &syn::Fields,
    rng: &syn::Ident,
    generate: &proc_macro2::Ident,
) -> Result<Vec<syn::FieldValue>, ()> {
    let mut out = Vec::new();

    for (n, field) in fields.iter().enumerate() {
        let attr = parse_attr(cx, &field.attrs)?;

        let member = match &field.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index::from(n)),
        };

        let mut attrs = Vec::new();

        // Transport cfg attributes, so we don't have to do that.
        for a in &field.attrs {
            if a.path().is_ident("cfg") {
                attrs.push(a.clone());
            }
        }

        let ty = &field.ty;

        let generate = if let Some(range) = attr.range {
            quote!(<#ty as #generate>::generate_range(#rng, #range))
        } else {
            quote!(<#ty as #generate>::generate(#rng))
        };

        out.push(syn::FieldValue {
            attrs,
            member,
            colon_token: Some(<Token![:]>::default()),
            expr: syn::Expr::Verbatim(generate),
        })
    }

    Ok(out)
}

#[derive(Default)]
struct Attr {
    range: Option<syn::Expr>,
}

fn parse_attr(cx: &mut Ctxt, attrs: &[syn::Attribute]) -> Result<Attr, ()> {
    let mut attr = Attr::default();

    for a in attrs {
        if !a.path().is_ident("generate") {
            continue;
        }

        let result = a.parse_nested_meta(|meta| {
            if meta.path.is_ident("range") {
                meta.input.parse::<Token![=]>()?;
                attr.range = Some(meta.input.parse()?);
                Ok(())
            } else {
                Err(syn::Error::new_spanned(meta.path, "Unsupported attribute"))
            }
        });

        if let Err(error) = result {
            cx.errors.push(error);
        }
    }

    if !cx.errors.is_empty() {
        return Err(());
    }

    Ok(attr)
}
