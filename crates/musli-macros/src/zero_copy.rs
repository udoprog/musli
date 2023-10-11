use std::cell::RefCell;
use std::mem::take;

use proc_macro2::{Span, TokenStream};
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

#[derive(Default)]
struct ReprAttr {
    repr: Option<Repr>,
    repr_align: Option<usize>,
}

impl ReprAttr {
    fn parse_attr(&mut self, cx: &Ctxt, attr: &syn::Attribute) {
        let result = attr.parse_nested_meta(|meta| {
            macro_rules! repr {
                ($ident:ident, $variant:expr) => {
                    if meta.path.is_ident(stringify!($ident)) {
                        if self.repr.is_some() {
                            return Err(syn::Error::new_spanned(
                                meta.path,
                                "ZeroCopy: only one kind of repr is supported",
                            ));
                        }

                        self.repr = Some($variant);
                        return Ok(());
                    }
                };
            }

            repr!(C, Repr::C);
            repr!(transparent, Repr::Transparent);
            repr!(u8, Repr::Num(meta.path.span(), Num::U8));
            repr!(u16, Repr::Num(meta.path.span(), Num::U16));
            repr!(u32, Repr::Num(meta.path.span(), Num::U32));
            repr!(u64, Repr::Num(meta.path.span(), Num::U64));
            repr!(u128, Repr::Num(meta.path.span(), Num::U128));
            repr!(i8, Repr::Num(meta.path.span(), Num::I8));
            repr!(i16, Repr::Num(meta.path.span(), Num::I16));
            repr!(i32, Repr::Num(meta.path.span(), Num::I32));
            repr!(i64, Repr::Num(meta.path.span(), Num::I64));
            repr!(i128, Repr::Num(meta.path.span(), Num::I128));
            repr!(isize, Repr::Num(meta.path.span(), Num::Isize));
            repr!(Usize, Repr::Num(meta.path.span(), Num::Usize));

            // #[repr(align(N))]
            if meta.path.is_ident("align") {
                let content;
                parenthesized!(content in meta.input);
                let lit: syn::LitInt = content.parse()?;
                let n: usize = lit.base10_parse()?;
                self.repr_align = Some(n);
                return Ok(());
            }

            Err(syn::Error::new_spanned(
                meta.path,
                "ZeroCopy: not a support representation",
            ))
        });

        if let Err(error) = result {
            cx.error(error);
        }
    }
}

#[derive(Clone, Copy)]
enum Repr {
    C,
    Transparent,
    Num(Span, Num),
}

#[derive(Clone, Copy)]
enum Num {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    Usize,
}

impl Num {
    fn as_ty(self) -> &'static str {
        match self {
            Num::U8 => "u8",
            Num::U16 => "u16",
            Num::U32 => "u32",
            Num::U64 => "u64",
            Num::U128 => "u128",
            Num::I8 => "i8",
            Num::I16 => "i16",
            Num::I32 => "i32",
            Num::I64 => "i64",
            Num::I128 => "i128",
            Num::Isize => "isize",
            Num::Usize => "isize",
        }
    }

    fn next_index(self, (neg, lit): (bool, syn::LitInt)) -> syn::Result<(bool, syn::LitInt)> {
        macro_rules! arm {
            ($kind:ident, $parse:ty, $ty:ty) => {{
                macro_rules! handle_neg {
                    (signed, $lit:ident) => {{
                        if neg && $lit != 0 {
                            let Some($lit) = $lit.checked_sub(1) else {
                                return Err(syn::Error::new_spanned(
                                    $lit,
                                    "Discriminant overflow for representation",
                                ));
                            };

                            ($lit != 0, $lit)
                        } else {
                            let Some($lit) = $lit.checked_add(1) else {
                                return Err(syn::Error::new_spanned(
                                    $lit,
                                    "Discriminant overflow for representation",
                                ));
                            };

                            (false, $lit)
                        }
                    }};

                    (unsigned, $lit:ident) => {{
                        if neg {
                            return Err(syn::Error::new_spanned(
                                $lit,
                                "Unsigned types can't be negative",
                            ));
                        }

                        let Some($lit) = $lit.checked_add(1) else {
                            return Err(syn::Error::new_spanned(
                                $lit,
                                "Discriminant overflow for representation",
                            ));
                        };

                        (false, $lit)
                    }};
                }

                let lit = lit.base10_parse::<$parse>()?;
                let (neg, lit) = handle_neg!($kind, lit);

                Ok((
                    neg,
                    syn::LitInt::new(&format!("{lit}{}", stringify!($ty)), lit.span()),
                ))
            }};
        }

        match self {
            Num::U8 => arm!(unsigned, u8, u8),
            Num::U16 => arm!(unsigned, u16, u16),
            Num::U32 => arm!(unsigned, u32, u32),
            Num::U64 => arm!(unsigned, u64, u64),
            Num::U128 => arm!(unsigned, u128, u128),
            Num::Usize => arm!(unsigned, usize, usize),
            Num::I8 => arm!(signed, u8, i8),
            Num::I16 => arm!(signed, u16, i16),
            Num::I32 => arm!(signed, u32, i32),
            Num::I64 => arm!(signed, u64, i64),
            Num::I128 => arm!(signed, u128, i128),
            Num::Isize => arm!(signed, usize, isize),
        }
    }
}

impl Repr {
    fn as_numerical_repr(self) -> Option<(Span, Num)> {
        match self {
            Repr::C => None,
            Repr::Transparent => None,
            Repr::Num(span, num) => Some((span, num)),
        }
    }
}

fn expand(cx: &Ctxt, input: &DeriveInput) -> Result<TokenStream, ()> {
    let mut generics = input.generics.clone();

    let mut repr = ReprAttr::default();
    let mut krate: syn::Path = syn::parse_quote!(musli_zerocopy);

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            repr.parse_attr(cx, attr);
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

    let Some(repr) = repr.repr else {
        cx.error(syn::Error::new_spanned(
            input,
            "ZeroCopy: struct must be marked with repr(C), repr(transparent), repr(u*), or repr(i*)",
        ));
        return Err(());
    };

    let buf_mut: syn::Path = syn::parse_quote!(#krate::buf::BufMut);
    let buf: syn::Path = syn::parse_quote!(#krate::buf::Buf);
    let error: syn::Path = syn::parse_quote!(#krate::Error);
    let result: syn::Path = syn::parse_quote!(#krate::__private::result::Result);
    let store_struct: syn::Path = syn::parse_quote!(#krate::buf::StoreStruct);
    let validator: syn::Path = syn::parse_quote!(#krate::buf::Validator);
    let zero_copy: syn::Path = syn::parse_quote!(#krate::traits::ZeroCopy);
    let zero_sized: syn::Path = syn::parse_quote!(#krate::traits::ZeroSized);

    let store_to;
    let validate;
    let impl_zero_sized;
    let any_bits;
    let mut needs_padding = quote!(false);
    let mut check_zero_sized = Vec::new();

    match &input.data {
        syn::Data::Struct(st) => {
            // Field types.
            let mut output = process_fields(cx, &st.fields);
            check_zero_sized.append(&mut output.check_zero_sized);

            if matches!(repr, Repr::Transparent) || output.first_field.is_none() {
                if let Some((ty, member)) = output.first_field {
                    store_to = quote! {
                        <#ty as #zero_copy>::store_to(&self.#member, buf)
                    };

                    validate = quote! {
                        <#ty as #zero_copy>::validate(buf)
                    };

                    // We are either a `repr(C)` with a single field or
                    // `repr(transparent)`, in which case we can inherit that
                    // fields padding.
                    needs_padding = quote! {
                        <#ty as #zero_copy>::NEEDS_PADDING
                    };
                } else {
                    store_to = quote! {
                        #result::Ok(())
                    };

                    validate = quote! {
                        #result::Ok(())
                    };

                    // This is a ZST. No padding needed.
                    needs_padding = quote!(false);
                }
            } else {
                let types = &output.types;

                store_to = quote! {
                    let mut writer = #buf_mut::store_struct(buf, self);

                    #(#store_struct::pad::<#types>(&mut writer);)*

                    // SAFETY: We've systematically ensured to pad all fields on the
                    // struct.
                    unsafe {
                        #store_struct::finish(writer)?;
                    }

                    #result::Ok(())
                };

                validate = quote! {
                    let mut validator = #buf::validate::<Self>(buf)?;
                    #(#validator::field::<#types>(&mut validator)?;)*
                    #validator::end(validator)?;
                    #result::Ok(())
                };
            }

            let name = &input.ident;
            let types = &output.types;

            impl_zero_sized = output.types.is_empty().then(|| {
                let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

                quote! {
                    // SAFETY: Type has no fields and has the necessary `repr`.
                    unsafe impl #impl_generics #zero_sized for #name #ty_generics #where_clause {}
                }
            });

            any_bits = quote!(true #(&& <#types as #zero_copy>::ANY_BITS)*);
        }
        syn::Data::Enum(en) => {
            let Some((span, num)) = repr.as_numerical_repr() else {
                cx.error(syn::Error::new_spanned(
                    en.enum_token,
                    "ZeroCopy: only supported for repr(i*) or repr(u*) enums",
                ));

                return Err(());
            };

            let ty = syn::Ident::new(num.as_ty(), span);

            let mut variants = Vec::new();
            let mut store_to_variants = Vec::new();

            let mut current = (
                false,
                syn::LitInt::new(&format!("0{}", num.as_ty()), ty.span()),
            );
            let mut first = true;

            for variant in &en.variants {
                let first = take(&mut first);

                let mut output = process_fields(cx, &variant.fields);
                check_zero_sized.append(&mut output.check_zero_sized);

                fn as_int(expr: &syn::Expr) -> Option<(bool, &syn::LitInt)> {
                    match expr {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(int),
                            ..
                        }) => Some((false, int)),
                        syn::Expr::Group(syn::ExprGroup { expr, .. }) => as_int(expr),
                        syn::Expr::Unary(syn::ExprUnary {
                            op: syn::UnOp::Neg(..),
                            expr,
                            ..
                        }) => as_int(expr).map(|(neg, int)| (!neg, int)),
                        _ => None,
                    }
                }

                if let Some((_, expr)) = &variant.discriminant {
                    let Some((neg, int)) = as_int(expr) else {
                        cx.error(syn::Error::new_spanned(
                            expr,
                            format!("Only numerical discriminants are supported: {:?}", expr),
                        ));
                        continue;
                    };

                    current = (neg, int.clone());
                } else if !first {
                    current = match num.next_index(current) {
                        Ok(out) => out,
                        Err(error) => {
                            cx.error(error);
                            break;
                        }
                    };
                }

                let (neg, current) = &current;

                let expr = syn::Expr::Lit(syn::ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Int(current.clone()),
                });

                let discriminator = if *neg {
                    syn::Expr::Unary(syn::ExprUnary {
                        attrs: Vec::new(),
                        op: syn::UnOp::Neg(<Token![-]>::default()),
                        expr: Box::new(expr),
                    })
                } else {
                    expr
                };

                let ident = &variant.ident;
                let types = &output.types;

                variants.push(quote! {
                    #discriminator => {
                        #(#validator::field::<#types>(&mut validator)?;)*
                    }
                });

                store_to_variants.push(quote! {
                    Self::#ident { .. } => {
                        #(#store_struct::pad::<#types>(&mut writer);)*
                    }
                });
            }

            store_to = quote! {
                let mut writer = #buf_mut::store_struct(buf, self);

                #store_struct::pad::<#ty>(&mut writer);

                match self {
                    #(#store_to_variants,)*
                }

                // SAFETY: We've systematically ensured to pad all fields on the
                // struct.
                unsafe {
                    #store_struct::finish(writer)?;
                }

                #result::Ok(())
            };

            let illegal_enum = quote::format_ident!("__illegal_enum_{}", num.as_ty());

            validate = quote! {
                let mut validator = #buf::validate::<Self>(buf)?;
                let discriminator = #validator::field::<#ty>(&mut validator)?;

                match *discriminator {
                    #(#variants,)*
                    value => return #result::Err(#error::#illegal_enum::<Self>(value)),
                }

                #validator::end(validator)?;
                Ok(())
            };

            impl_zero_sized = None;
            any_bits = quote!(false);
        }
        syn::Data::Union(data) => {
            cx.error(syn::Error::new_spanned(
                data.union_token,
                "ZeroCopy: not supported for unions",
            ));
            return Err(());
        }
    };

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

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote::quote! {
        #check_zero_sized

        #impl_zero_sized

        unsafe impl #impl_generics #zero_copy for #name #ty_generics #where_clause {
            const ANY_BITS: bool = #any_bits;
            const NEEDS_PADDING: bool = #needs_padding;

            fn store_to<__B: ?Sized>(&self, buf: &mut __B) -> #result<(), #error>
            where
                __B: #buf_mut
            {
                #store_to
            }

            unsafe fn validate(buf: &#buf) -> #result<(), #error> {
                #validate
            }
        }
    })
}

#[derive(Default)]
struct Fields<'a> {
    types: Vec<&'a syn::Type>,
    first_field: Option<(&'a syn::Type, syn::Member)>,
    check_zero_sized: Vec<&'a syn::Type>,
}

fn process_fields<'a>(cx: &Ctxt, fields: &'a syn::Fields) -> Fields<'a> {
    let mut output = Fields::default();

    for (index, field) in fields.iter().enumerate() {
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
            output.check_zero_sized.push(ty);
            continue;
        }

        output.types.push(ty);

        if output.first_field.is_none() {
            output.first_field = Some((ty, member));
        }
    }

    output
}
