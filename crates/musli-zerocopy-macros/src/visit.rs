use std::cell::RefCell;

use proc_macro2::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Token};

#[derive(Default)]
struct Ctxt {
    errors: RefCell<Vec<syn::Error>>,
}

impl Ctxt {
    fn error(&self, error: syn::Error) {
        self.errors.borrow_mut().push(error);
    }
}

pub struct Expander<'a> {
    input: &'a DeriveInput,
}

impl<'a> Expander<'a> {
    pub fn new(input: &'a DeriveInput) -> Self {
        Self { input }
    }
}

impl Expander<'_> {
    pub fn expand(&self) -> Result<TokenStream, Vec<syn::Error>> {
        let cx = Ctxt::default();

        let Ok(output) = expand(&cx, self.input) else {
            return Err(cx.errors.into_inner());
        };

        let errors = cx.errors.into_inner();

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(output)
    }
}

fn expand(cx: &Ctxt, input: &DeriveInput) -> Result<TokenStream, ()> {
    let mut krate: syn::Path = syn::parse_quote!(musli_zerocopy);

    for attr in &input.attrs {
        if attr.path().is_ident("visit") {
            let result = attr.parse_nested_meta(|meta: ParseNestedMeta| {
                if meta.path.is_ident("crate") {
                    if meta.input.parse::<Option<Token![=]>>()?.is_some() {
                        krate = meta.input.parse()?;
                    } else {
                        krate = syn::parse_quote!(crate);
                    }

                    return Ok(());
                }

                Err(syn::Error::new(
                    meta.input.span(),
                    "Visit: Unsupported attribute",
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }
    }

    match &input.data {
        syn::Data::Struct(st) => {
            process_fields(cx, &st.fields);
        }
        syn::Data::Enum(en) => {
            for v in &en.variants {
                process_fields(cx, &v.fields);
            }
        }
        syn::Data::Union(u) => {
            process_fields(cx, &u.fields.named);
        }
    }

    let error: syn::Path = syn::parse_quote!(#krate::Error);
    let result: syn::Path = syn::parse_quote!(#krate::__private::result::Result);
    let buf: syn::Path = syn::parse_quote!(#krate::__private::Buf);
    let visit: syn::Path = syn::parse_quote!(#krate::__private::Visit);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let visit_impl = quote! {
        impl #impl_generics #visit for #name #ty_generics #where_clause {
            type Target = Self;

            #[inline]
            fn visit<__V, __O>(&self, _: &#buf, visitor: __V) -> #result<__O, #error>
            where
                __V: FnOnce(&Self::Target) -> __O,
            {
                Ok(visitor(self))
            }
        }
    };

    Ok(quote::quote! {
        #visit_impl
    })
}

fn process_fields<'a, I>(cx: &Ctxt, fields: I)
where
    I: IntoIterator<Item = &'a syn::Field>,
{
    for field in fields {
        for attr in &field.attrs {
            if attr.path().is_ident("visit") {
                let result = attr.parse_nested_meta(|meta: ParseNestedMeta| {
                    Err(syn::Error::new(
                        meta.input.span(),
                        "Visit: Unsupported attribute",
                    ))
                });

                if let Err(error) = result {
                    cx.error(error);
                }
            }
        }
    }
}
