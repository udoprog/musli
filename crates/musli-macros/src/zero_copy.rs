use std::cell::RefCell;

use proc_macro2::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parenthesized, DeriveInput, Token};

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
    let mut is_repr_transparent = false;
    let mut repr_align = None;
    let mut krate: syn::Path = syn::parse_quote!(musli_zerocopy);

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    is_repr_c = true;
                    return Ok(());
                }

                if meta.path.is_ident("transparent") {
                    is_repr_transparent = true;
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

                Err(syn::Error::new_spanned(
                    meta.path,
                    "ZeroCopy: only repr(C) is supported",
                ))
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

                    let predicates =
                        Punctuated::<syn::WherePredicate, Token![,]>::parse_terminated(&content)?;

                    generics.make_where_clause().predicates.extend(predicates);
                    return Ok(());
                }

                if meta.path.is_ident("crate") {
                    if meta.input.parse::<Option<Token![=]>>()?.is_some() {
                        krate = meta.input.parse()?;
                    } else {
                        krate = syn::parse_quote!(crate);
                    }

                    return Ok(());
                }

                Ok(())
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }
    }

    if !is_repr_c && !is_repr_transparent {
        cx.error(syn::Error::new_spanned(
            input,
            "ZeroCopy: struct must be marked with repr(C)",
        ));
        return Err(());
    }

    let buf_mut: syn::Path = syn::parse_quote!(#krate::BufMut);
    let store_struct: syn::Path = syn::parse_quote!(#krate::StoreStruct);
    let buf: syn::Path = syn::parse_quote!(#krate::Buf);
    let error: syn::Path = syn::parse_quote!(#krate::Error);
    let validator: syn::Path = syn::parse_quote!(#krate::Validator);
    let zero_copy: syn::Path = syn::parse_quote!(#krate::ZeroCopy);
    let zero_sized: syn::Path = syn::parse_quote!(#krate::ZeroSized);
    let result: syn::Path = syn::parse_quote!(#krate::__private::result::Result);

    // Field types.
    let mut fields = Vec::new();
    let mut first_field = None;
    let mut check_zero_sized = Vec::new();

    for (index, field) in st.fields.iter().enumerate() {
        let mut ignore = None;

        for attr in &field.attrs {
            if attr.path().is_ident("zero_copy") {
                let result = attr.parse_nested_meta(|meta: ParseNestedMeta| {
                    if meta.path.is_ident("ignore") {
                        ignore = Some(meta.path.span());
                        return Ok(());
                    }

                    Ok(())
                });

                if let Err(error) = result {
                    cx.error(error);
                }
            }
        }

        let ty = &field.ty;

        let member = match &field.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index::from(index)),
        };

        if ignore.is_some() {
            check_zero_sized.push(ty);
            continue;
        }

        fields.push(ty);

        if first_field.is_none() {
            first_field = Some((ty, member));
        }
    }

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let store_to;
    let coerce;
    let coerce_mut;
    let validate;

    if is_repr_transparent || first_field.is_none() {
        if let Some((ty, member)) = first_field {
            store_to = quote! {
                <#ty as #zero_copy>::store_to(&self.#member, buf)
            };

            coerce = quote! {
                unsafe {
                    <#ty as #zero_copy>::validate(buf)?;
                    #result::Ok(#buf::cast(buf))
                }
            };

            coerce_mut = quote! {
                unsafe {
                    <#ty as #zero_copy>::validate(buf)?;
                    #result::Ok(#buf::cast_mut(buf))
                }
            };

            validate = quote! {
                <#ty as #zero_copy>::validate(buf)
            };
        } else {
            store_to = quote! {
                #result::Ok(())
            };

            coerce = quote! {
                #result::Ok(unsafe { #buf::cast(buf) })
            };

            coerce_mut = quote! {
                #result::Ok(unsafe { #buf::cast_mut(buf) })
            };

            validate = quote! {
                #result::Ok(())
            };
        }
    } else {
        store_to = quote! {
            let mut writer = #buf_mut::store_struct(buf, self);

            #(#store_struct::pad::<#fields>(&mut writer);)*

            // SAFETY: We've systematically ensured to pad all fields on the
            // struct.
            unsafe {
                #store_struct::finish(writer)?;
            }

            #result::Ok(())
        };

        coerce = quote! {
            let mut validator = #buf::validate::<Self>(buf)?;
            #(#validator::field::<#fields>(&mut validator)?;)*
            #validator::end(validator)?;
            #result::Ok(unsafe { #buf::cast(buf) })
        };

        coerce_mut = quote! {
            let mut validator = #buf::validate::<Self>(buf)?;
            #(#validator::field::<#fields>(&mut validator)?;)*
            #validator::end(validator)?;
            #result::Ok(unsafe { #buf::cast_mut(buf) })
        };

        validate = quote! {
            let mut validator = #buf::validate::<Self>(buf)?;
            #(#validator::field::<#fields>(&mut validator)?;)*
            #validator::end(validator)?;
            #result::Ok(())
        };
    }

    let check_zero_sized = (!check_zero_sized.is_empty()).then(|| {
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        quote! {
            const _: () = {
                fn ensure_zero_sized<T: #zero_sized>() {}

                fn ensure_fields #impl_generics() #where_clause {
                    #(ensure_zero_sized::<#check_zero_sized>();)*
                }
            };
        }
    });

    let impl_zero_sized = fields.is_empty().then(|| {
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        quote! {
            // SAFETY: Type has no fields and has the necessary `repr`.
            unsafe impl #impl_generics #zero_sized for #name #ty_generics #where_clause {}
        }
    });

    Ok(quote::quote! {
        #check_zero_sized

        #impl_zero_sized

        unsafe impl #impl_generics #zero_copy for #name #ty_generics #where_clause {
            const ANY_BITS: bool = true #(&& <#fields as #zero_copy>::ANY_BITS)*;

            fn store_to<__B: ?Sized>(&self, buf: &mut __B) -> #result<(), #error>
            where
                __B: #buf_mut
            {
                #store_to
            }

            unsafe fn coerce(buf: &#buf) -> #result<&Self, #error> {
                #coerce
            }

            unsafe fn coerce_mut(buf: &mut #buf) -> #result<&mut Self, #error> {
                #coerce_mut
            }

            unsafe fn validate(buf: &#buf) -> #result<(), #error> {
                #validate
            }
        }
    })
}
