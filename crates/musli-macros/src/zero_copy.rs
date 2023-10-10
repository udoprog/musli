use std::cell::RefCell;

use proc_macro2::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Token, parenthesized};

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

    let mut generics = input.generics.clone();

    let mut is_repr_c = false;
    let mut repr_align = None;

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    is_repr_c = true;
                    return Ok(());
                }
                
                // #[repr(align(N))]
                if meta.path.is_ident("align") {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit: syn::LitInt = content.parse()?;
                    let n: usize = lit.base10_parse()?;
                    repr_align = Some(n);
                    return Ok(());
                }

                Err(syn::Error::new_spanned(meta.path, "ZeroCopy: only repr(C) is supported"))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }

        if attr.path().is_ident("zero_copy") {
            let result = attr.parse_nested_meta(|meta: ParseNestedMeta| {
                if meta.path.is_ident("bounds") {
                    meta.input.parse::<Token![=]>()?;
                    let content;
                    syn::braced!(content in meta.input);
                    generics.make_where_clause().predicates.extend(Punctuated::<
                        syn::WherePredicate,
                        Token![,],
                    >::parse_terminated(
                        &content
                    )?);
                    return Ok(());
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

    let buf_mut: syn::Path = syn::parse_quote!(musli_zerocopy::BufMut);
    let struct_writer: syn::Path = syn::parse_quote!(musli_zerocopy::StructWriter);
    let buf: syn::Path = syn::parse_quote!(musli_zerocopy::Buf);
    let error: syn::Path = syn::parse_quote!(musli_zerocopy::Error);
    let validator: syn::Path = syn::parse_quote!(musli_zerocopy::Validator);
    let zero_copy: syn::Path = syn::parse_quote!(musli_zerocopy::ZeroCopy);

    let mut writes = Vec::new();
    let mut validates = Vec::new();

    let mut any_bits = Vec::new();

    for field in st.fields.iter() {
        let ty = &field.ty;

        writes.push(quote! {
            #struct_writer::pad::<#ty>(&mut writer);
        });

        validates.push(quote! {
            #validator::field::<#ty>(&mut validator)?;
        });

        any_bits.push(quote!(<#ty as #zero_copy>::ANY_BITS));
    }

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let validates = &validates[..];

    let any_bits = if any_bits.is_empty() {
        quote!(true)
    } else {
        quote!(true #(&& #any_bits)*)
    };

    Ok(quote::quote! {
        unsafe impl #impl_generics #zero_copy for #name #ty_generics #where_clause {
            const ANY_BITS: bool = #any_bits;

            fn write_to<__B: ?Sized>(&self, buf: &mut __B) -> Result<(), #error>
            where
                __B: #buf_mut
            {
                let mut writer = #buf_mut::writer(buf, self);

                #(#writes)*

                // SAFETY: We've systematically ensured to pad all fields on the
                // struct.
                unsafe {
                    #struct_writer::finish(writer)?;
                }

                Ok(())
            }

            fn coerce(buf: &#buf) -> Result<&Self, #error> {
                let mut validator = #buf::validate::<Self>(buf)?;
                #(#validates)*
                #validator::end(validator)?;
                Ok(unsafe { #buf::cast(buf) })
            }

            unsafe fn validate(buf: &#buf) -> Result<(), #error> {
                let mut validator = #buf::validate_unchecked::<Self>(buf)?;
                #(#validates)*
                #validator::end(validator)?;
                Ok(())
            }
        }
    })
}
