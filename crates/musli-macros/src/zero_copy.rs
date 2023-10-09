use std::cell::RefCell;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::ParseStream;
use syn::DeriveInput;

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

impl<'a> Expander<'a> {
    pub fn expand(&self) -> Result<TokenStream, Vec<syn::Error>> {
        let cx = Ctxt::default();

        let Ok(output) = expand(&cx, &self.input) else {
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
    let st = match &input.data {
        syn::Data::Struct(st) => st,
        syn::Data::Enum(data) => {
            cx.error(syn::Error::new_spanned(
                data.enum_token,
                "ZeroCopy: not supported for enums",
            ));
            return Err(());
        }
        syn::Data::Union(data) => {
            cx.error(syn::Error::new_spanned(
                data.union_token,
                "ZeroCopy: not supported for unions",
            ));
            return Err(());
        }
    };

    let mut is_repr_c = false;

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            let result = attr.parse_args_with(|input: ParseStream| {
                let ident: syn::Ident = input.parse()?;

                if ident == "C" {
                    is_repr_c = true;
                }

                Ok(())
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }
    }

    if !is_repr_c {
        cx.error(syn::Error::new_spanned(
            input,
            "ZeroCopy: struct must be marked with repr(C)",
        ));
        return Err(());
    }

    let zero_copy: syn::Path = syn::parse_quote!(musli_zerocopy::ZeroCopy);
    let owned_buf: syn::Path = syn::parse_quote!(musli_zerocopy::OwnedBuf);
    let error: syn::Path = syn::parse_quote!(musli_zerocopy::Error);
    let buf: syn::Path = syn::parse_quote!(musli_zerocopy::Buf);
    let validator: syn::Path = syn::parse_quote!(musli_zerocopy::Validator);

    let mut writes = Vec::new();
    let mut validates = Vec::new();

    for (index, field) in st.fields.iter().enumerate() {
        let member = match &field.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index::from(index)),
        };

        writes.push(quote! {
            #owned_buf::write(buf, &self.#member)?;
        });

        let ty = &field.ty;

        validates.push(quote! {
            #validator::validate::<#ty>(&mut validator)?;
        });
    }

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote::quote! {
        unsafe impl #impl_generics #zero_copy for #name #ty_generics #where_clause {
            fn write_to(&self, buf: &mut #owned_buf) -> Result<(), Error> {
                #(#writes)*
                Ok(())
            }

            fn validate(buf: &#buf) -> Result<&Self, #error> {
                let mut validator = #buf::validator::<Self>(buf)?;
                #(#validates)*
                #validator::finalize(validator)?;
                Ok(unsafe { #buf::cast(buf) })
            }
        }
    })
}
