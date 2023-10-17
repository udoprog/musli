use std::cell::RefCell;
use std::mem::take;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parenthesized, Token};

#[derive(Default)]
struct Ctxt {
    errors: RefCell<Vec<syn::Error>>,
}

impl Ctxt {
    fn error(&self, error: syn::Error) {
        self.errors.borrow_mut().push(error);
    }
}

pub struct Expander {
    input: syn::DeriveInput,
}

impl Expander {
    pub fn new(input: syn::DeriveInput) -> Self {
        Self { input }
    }
}

impl Expander {
    pub fn expand(self) -> Result<TokenStream, Vec<syn::Error>> {
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

fn expand(cx: &Ctxt, input: syn::DeriveInput) -> Result<TokenStream, ()> {
    let (attrs, name, mut generics, data) = (input.attrs, input.ident, input.generics, input.data);

    let mut r = ReprAttr::default();
    let mut krate: syn::Path = syn::parse_quote!(musli_zerocopy);

    for attr in &attrs {
        if attr.path().is_ident("repr") {
            r.parse_attr(cx, attr);
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

                Err(syn::Error::new(
                    meta.input.span(),
                    "ZeroCopy: Unsupported attribute",
                ))
            });

            if let Err(error) = result {
                cx.error(error);
            }
        }
    }

    let Some((repr_span, repr)) = r.repr else {
        cx.error(syn::Error::new(
            Span::call_site(),
            "ZeroCopy: struct must be marked with repr(C), repr(transparent), repr(u*), or repr(i*)",
        ));
        return Err(());
    };

    let buf_mut: syn::Path = syn::parse_quote!(#krate::buf::BufMut);
    let error: syn::Path = syn::parse_quote!(#krate::Error);
    let mem: syn::Path = syn::parse_quote!(#krate::__private::mem);
    let padder: syn::Path = syn::parse_quote!(#krate::buf::Padder);
    let ptr: syn::Path = syn::parse_quote!(#krate::__private::ptr);
    let result: syn::Path = syn::parse_quote!(#krate::__private::result::Result);
    let validator: syn::Path = syn::parse_quote!(#krate::buf::Validator);
    let zero_copy: syn::Path = syn::parse_quote!(#krate::traits::ZeroCopy);
    let zero_sized: syn::Path = syn::parse_quote!(#krate::traits::ZeroSized);

    let store_to;
    let pad;
    let validate;
    let impl_zero_sized;
    let any_bits;
    let padded;
    // Expands to an expression which is not executed, but ensures that the type
    // expands only to the fields visible to the proc macro or causes a compile
    // error.
    let check_fields;
    let mut check_zero_sized = Vec::new();

    match &data {
        syn::Data::Struct(st) => {
            // Field types.
            let mut output = process_fields(cx, &st.fields);
            check_zero_sized.append(&mut output.check_zero_sized);

            match (repr, &output.first_field) {
                (Repr::Transparent, Some((ty, member))) => {
                    store_to = quote! {
                        <#ty as #zero_copy>::store_unaligned(#ptr::addr_of!((*this).#member), buf);
                    };

                    pad = quote! {
                        <#ty as #zero_copy>::pad(#ptr::addr_of!((*this).#member), #padder::transparent::<#ty>(padder));
                    };

                    validate = quote! {
                        <#ty as #zero_copy>::validate(#validator::transparent::<#ty>(validator))?;
                    };
                }
                _ => {
                    let members = &output.members;
                    let types = &output.types;

                    match r.repr_packed {
                        Some((_, align)) => {
                            pad = quote! {
                                #(#padder::pad_with(padder, #ptr::addr_of!((*this).#members), #align);)*
                            };

                            store_to = quote! {
                                // SAFETY: We've systematically ensured to pad all
                                // fields on the struct.
                                let mut padder = #buf_mut::store_struct(buf, this);
                                #(#padder::pad_with::<#types>(&mut padder, #ptr::addr_of!((*this).#members), #align);)*
                                #padder::remaining(padder);
                            };

                            validate = quote! {
                                // SAFETY: We've systematically ensured that we're
                                // only validating over fields within the size of
                                // this type.
                                #(#validator::validate_with::<#types>(validator, #align)?;)*
                            };
                        }
                        _ => {
                            store_to = quote! {
                                // SAFETY: We've systematically ensured to pad all
                                // fields on the struct.
                                let mut padder = #buf_mut::store_struct(buf, this);
                                #(#padder::pad::<#types>(&mut padder, #ptr::addr_of!((*this).#members));)*
                                #padder::remaining(padder);
                            };

                            pad = quote! {
                                #(#padder::pad(padder, #ptr::addr_of!((*this).#members));)*
                            };

                            validate = quote! {
                                // SAFETY: We've systematically ensured that we're
                                // only validating over fields within the size of
                                // this type.
                                #(#validator::validate::<#types>(validator)?;)*
                            };
                        }
                    }
                }
            }

            let mut field_sizes = Vec::new();
            let mut field_padded = Vec::new();

            for ty in output.types.iter() {
                field_sizes.push(quote!(#mem::size_of::<#ty>()));
                field_padded.push(quote!(<#ty as #zero_copy>::PADDED));
            }

            // ZSTs are padded if their alignment is not 1, any other type is
            // padded if the sum of all their field sizes does not match the
            // size of the type itself.
            padded = quote!((#mem::size_of::<Self>() == 0 && #mem::align_of::<Self>() > 1) || #mem::size_of::<Self>() != (0 #(+ #field_sizes)*) #(|| #field_padded)*);

            let types = &output.types;

            impl_zero_sized = types.is_empty().then(|| {
                let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

                quote! {
                    // SAFETY: Type has no fields and has the necessary `repr`.
                    unsafe impl #impl_generics #zero_sized for #name #ty_generics #where_clause {}
                }
            });

            any_bits = quote!(true #(&& <#types as #zero_copy>::ANY_BITS)*);

            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            // NB: We go through a pointer here to allow for padded structs.
            check_fields = {
                let pat = build_field_exhaustive_pattern([name.clone()], &output, &st.fields);

                quote!(
                    const _: () = {
                        #[allow(unused)]
                        fn check_struct_fields #impl_generics(this: #name #ty_generics) #where_clause {
                            let #pat = this;
                        }
                    };
                )
            };
        }
        syn::Data::Enum(en) => {
            if let Some((span, _)) = r.repr_packed {
                cx.error(syn::Error::new(
                    span,
                    "ZeroCopy: repr(packed) is only supported on structs",
                ));

                return Err(());
            }

            let Some((span, num)) = repr.as_numerical_repr() else {
                cx.error(syn::Error::new(
                    repr_span,
                    "ZeroCopy: only supported for repr(i*) or repr(u*) enums",
                ));

                return Err(());
            };

            let ty = syn::Ident::new(num.as_ty(), span);

            let mut validate_variants = Vec::new();
            let mut store_to_variants = Vec::new();
            let mut pad_variants = Vec::new();
            let mut padded_variants = Vec::new();
            let mut variant_fields = Vec::new();

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
                let variables = &output.variables;
                let assigns = &output.assigns;

                validate_variants.push(quote! {
                    #discriminator => {
                        #(#validator::validate::<#types>(validator)?;)*
                    }
                });

                store_to_variants.push(quote! {
                    Self::#ident { #(#assigns,)* .. } => {
                        #(#padder::pad(&mut padder, #variables);)*
                    }
                });

                pad_variants.push(quote! {
                    Self::#ident { #(#assigns,)* .. } => {
                        #(#padder::pad(padder, #variables);)*
                    }
                });

                let mut field_sizes = Vec::new();
                let mut field_padded = Vec::new();

                for ty in output.types.iter() {
                    field_sizes.push(quote!(#mem::size_of::<#ty>()));
                    field_padded.push(quote!(<#ty as #zero_copy>::PADDED));
                }

                let base_size = num.size(&mem);

                // Struct does not need to be padded if all elements are the
                // same size and alignment.
                padded_variants.push(quote!((#mem::size_of::<Self>() != (#base_size #(+ #field_sizes)*) #(|| #field_padded)*)));

                variant_fields.push({
                    let pat = build_field_exhaustive_pattern(
                        [name.clone(), variant.ident.clone()],
                        &output,
                        &variant.fields,
                    );
                    quote!(#pat => ())
                });
            }

            store_to = quote! {
                // SAFETY: We've systematically ensured to pad all fields on the
                // struct.
                let mut padder = #buf_mut::store_struct(buf, this);
                #padder::pad_primitive::<#ty>(&mut padder);

                // NOTE: this is assumed to be properly, since enums cannot
                // be packed.
                match &*this {
                    #(#store_to_variants,)*
                }

                #padder::remaining(padder);
            };

            pad = quote! {
                #padder::pad_primitive::<#ty>(padder);

                // NOTE: this is assumed to be properly, since enums cannot be
                // packed.
                match &*this {
                    #(#pad_variants,)*
                }
            };

            let illegal_enum = quote::format_ident!("__illegal_enum_{}", num.as_ty());

            validate = quote! {
                // SAFETY: We've systematically ensured that we're only
                // validating over fields within the size of this type.
                let discriminator = *#validator::field::<#ty>(validator)?;

                match discriminator {
                    #(#validate_variants,)*
                    value => return #result::Err(#error::#illegal_enum::<Self>(value)),
                }
            };

            impl_zero_sized = None;
            any_bits = quote!(false);
            padded = quote!(false #(|| #padded_variants)*);

            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            // NB: Since enums can't be packed we can use a reference.
            check_fields = quote!(
                const _: () = {
                    #[allow(unused)]
                    fn check_variants_fields #impl_generics(this: &#name #ty_generics) #where_clause {
                        match this {
                            #(#variant_fields,)*
                        }
                    }
                };
            );
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

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #check_zero_sized

        #check_fields

        #impl_zero_sized

        unsafe impl #impl_generics #zero_copy for #name #ty_generics #where_clause {
            const ANY_BITS: bool = #any_bits;
            const PADDED: bool = #padded;

            #[inline]
            unsafe fn store_unaligned(this: *const Self, buf: &mut #buf_mut<'_>) {
                #store_to
            }

            #[inline]
            unsafe fn pad(this: *const Self, padder: &mut #padder<'_, Self>) {
                #pad
            }

            #[inline]
            unsafe fn validate(validator: &mut #validator<'_, Self>) -> #result<(), #error> {
                #validate
                #result::Ok(())
            }
        }
    })
}

/// Construct a match pattern with carefully assigned spans to improve
/// diagnostics as much as possible.
fn build_field_exhaustive_pattern<const N: usize>(
    steps: [syn::Ident; N],
    output: &Fields<'_>,
    fields: &syn::Fields,
) -> syn::Pat {
    let mut path = syn::Path {
        leading_colon: None,
        segments: syn::punctuated::Punctuated::default(),
    };

    for step in steps {
        path.segments.push(syn::PathSegment::from(step));
    }

    match fields {
        syn::Fields::Named(named) => {
            let mut fields = syn::punctuated::Punctuated::new();

            for member in &output.exhaustive {
                fields.push(syn::FieldPat {
                    attrs: Vec::new(),
                    member: member.clone(),
                    colon_token: None,
                    pat: Box::new(syn::Pat::Verbatim(quote!(#member))),
                });
            }

            syn::Pat::Struct(syn::PatStruct {
                attrs: Vec::new(),
                qself: None,
                path: path.clone(),
                brace_token: named.brace_token,
                fields,
                rest: None,
            })
        }
        syn::Fields::Unnamed(unnamed) => {
            let mut elems = syn::punctuated::Punctuated::new();

            for _ in &output.exhaustive {
                elems.push(syn::Pat::Wild(syn::PatWild {
                    attrs: Vec::new(),
                    underscore_token: <syn::Token![_]>::default(),
                }));
            }

            syn::Pat::TupleStruct(syn::PatTupleStruct {
                attrs: Vec::new(),
                qself: None,
                path: path.clone(),
                paren_token: unnamed.paren_token,
                elems,
            })
        }
        syn::Fields::Unit => syn::Pat::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: path.clone(),
        }),
    }
}

#[derive(Default)]
struct Fields<'a> {
    types: Vec<&'a syn::Type>,
    members: Vec<syn::Member>,
    exhaustive: Vec<syn::Member>,
    variables: Vec<syn::Ident>,
    assigns: Vec<syn::FieldValue>,
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

                    Err(syn::Error::new(
                        meta.input.span(),
                        "ZeroCopy: Unsupported attribute",
                    ))
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

        // Ignored fields are ignored in bindings.
        output.exhaustive.push(member.clone());

        if ignore.is_some() {
            output.check_zero_sized.push(ty);
            continue;
        }

        output.types.push(ty);
        output.members.push(member.clone());

        let variable = match &field.ident {
            Some(ident) => ident.clone(),
            None => quote::format_ident!("_{}", index),
        };

        if matches!(&member, syn::Member::Named(..)) {
            output.assigns.push(syn::parse_quote!(#member));
        } else {
            output.assigns.push(syn::parse_quote!(#member: #variable));
        }

        output.variables.push(variable);

        if output.first_field.is_none() {
            output.first_field = Some((ty, member));
        }
    }

    output
}

#[derive(Default)]
struct ReprAttr {
    repr: Option<(Span, Repr)>,
    repr_align: Option<usize>,
    repr_packed: Option<(Span, usize)>,
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

                        self.repr = Some((meta.path.span(), $variant));
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

            // #[repr(packed)] or #[repr(packed(N))], omitted N means 1
            if meta.path.is_ident("packed") {
                if meta.input.peek(syn::token::Paren) {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit: syn::LitInt = content.parse()?;
                    let n: usize = lit.base10_parse()?;
                    self.repr_packed = Some((meta.input.span(), n));
                } else {
                    self.repr_packed = Some((meta.input.span(), 1));
                }
                return Ok(());
            }

            Err(syn::Error::new_spanned(
                meta.path,
                "ZeroCopy: unsupported #[repr(..)]",
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

enum NumSize<'a> {
    Fixed(usize),
    Pointer(&'a syn::Path),
}

impl ToTokens for NumSize<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NumSize::Fixed(size) => size.to_tokens(tokens),
            NumSize::Pointer(mem) => {
                tokens.extend(quote!(#mem::size_of::<usize>()));
            }
        }
    }
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
    fn size(self, mem: &syn::Path) -> NumSize<'_> {
        match self {
            Num::U8 => NumSize::Fixed(1),
            Num::U16 => NumSize::Fixed(2),
            Num::U32 => NumSize::Fixed(4),
            Num::U64 => NumSize::Fixed(8),
            Num::U128 => NumSize::Fixed(16),
            Num::I8 => NumSize::Fixed(1),
            Num::I16 => NumSize::Fixed(2),
            Num::I32 => NumSize::Fixed(4),
            Num::I64 => NumSize::Fixed(8),
            Num::I128 => NumSize::Fixed(16),
            Num::Isize => NumSize::Pointer(mem),
            Num::Usize => NumSize::Pointer(mem),
        }
    }

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
