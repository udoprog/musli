use std::mem::take;
use std::slice;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Token;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

enum AsBytes {
    Default,
    Disabled,
    Method(syn::Path),
}

pub(super) struct Attributes {
    as_bytes: AsBytes,
    disabled: bool,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut end = false;
        let mut as_bytes = AsBytes::Default;
        let mut disabled = false;

        while !input.is_empty() {
            let path: syn::Path = input.parse()?;

            if path.is_ident("as_bytes_disabled") {
                as_bytes = AsBytes::Disabled;
            } else if path.is_ident("as_bytes") {
                input.parse::<Token![=]>()?;
                as_bytes = AsBytes::Method(input.parse()?);
            } else if path.is_ident("disabled") {
                disabled = true;
            } else {
                return Err(syn::Error::new_spanned(path, "Unsupported attribute"));
            }

            if end {
                break;
            }

            end = input.parse::<Option<Token![,]>>()?.is_none();
        }

        Ok(Attributes { as_bytes, disabled })
    }
}

pub(super) struct Benchmarker {
    module_impl: syn::ItemMod,
}

impl Parse for Benchmarker {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            module_impl: input.parse()?,
        })
    }
}

impl Benchmarker {
    pub(crate) fn expand(mut self, attrs: &Attributes) -> syn::Result<TokenStream> {
        let Some((brace, content)) = self.module_impl.content.take() else {
            return Err(syn::Error::new_spanned(
                self.module_impl,
                "Expected module content",
            ));
        };

        let mut buffer_fn = None;
        let mut reset_fn = None;
        let mut encode_fn = None;
        let mut decode_fn = None;
        let mut new_content = Vec::new();
        let mut providers = Vec::new();

        for item in content {
            match item {
                syn::Item::Fn(mut f) => {
                    let attrs = function_attrs(&mut f.attrs)?;

                    if attrs.is_provider {
                        let ty = match &f.sig.output {
                            syn::ReturnType::Type(_, ty) => (**ty).clone(),
                            _ => syn::parse_quote!(()),
                        };

                        providers.push(Provider { f, ty });
                        continue;
                    }

                    if f.sig.ident == "buffer" {
                        buffer_fn = Some(f);
                        continue;
                    }

                    if f.sig.ident == "reset" {
                        let mut reset_arguments = Vec::new();

                        for (index, arg) in f.sig.inputs.iter_mut().enumerate() {
                            argument_attrs(index, arg, &mut reset_arguments, true)?;
                        }

                        reset_fn = Some((f, reset_arguments));
                        continue;
                    }

                    if f.sig.ident == "encode" {
                        let mut arguments = Vec::new();

                        for (index, arg) in f.sig.inputs.iter_mut().enumerate() {
                            argument_attrs(index, arg, &mut arguments, false)?;
                        }

                        encode_fn = Some((f, arguments));
                        continue;
                    }

                    if f.sig.ident == "decode" {
                        let mut arguments = Vec::new();

                        for (index, arg) in f.sig.inputs.iter_mut().enumerate() {
                            argument_attrs(index, arg, &mut arguments, false)?;
                        }

                        decode_fn = Some((f, arguments));
                        continue;
                    }

                    new_content.push(syn::Item::Fn(f));
                }
                item => {
                    new_content.push(item);
                }
            }
        }

        let fns = [
            buffer_fn.as_mut(),
            reset_fn.as_mut().map(|(f, _)| f),
            encode_fn.as_mut().map(|(f, _)| f),
            decode_fn.as_mut().map(|(f, _)| f),
        ];

        // Apply #[inline(always)] to all provided functions.
        for f in fns.into_iter().flatten() {
            f.attrs.push(syn::parse_quote!(#[inline(always)]));
        }

        let Some((encode_fn, encode_args)) = encode_fn else {
            return Err(syn::Error::new_spanned(
                self.module_impl,
                "Expected `encode` function",
            ));
        };

        let Some((decode_fn, decode_args)) = decode_fn else {
            return Err(syn::Error::new_spanned(
                self.module_impl,
                "Expected `decode` function",
            ));
        };

        let visibility = self.module_impl.vis.clone();

        let Some((encode_return, encode_error)) = unpack_output_result(&encode_fn.sig.output)
        else {
            return Err(syn::Error::new_spanned(
                encode_fn.sig.output,
                "Expected `encode` function to return a `Result<T, E>`",
            ));
        };

        let encode_lifetime = find_buf_param(&encode_fn, None);
        let decode_lifetime = find_buf_param(&decode_fn, encode_lifetime.map(|p| &p.lifetime));

        let mut extra_lts = Vec::new();
        let mut extra_markers = Vec::new();

        let (buf_lt, buf_param) = match decode_lifetime.or(encode_lifetime) {
            Some(param) => {
                for lt in param.bounds.iter() {
                    extra_lts.push(lt.clone());
                    extra_markers.push(syn::Ident::new(&format!("_{}", lt.ident), lt.span()))
                }

                (param.lifetime.clone(), param.clone())
            }
            None => (syn::parse_quote!('__buf), syn::parse_quote!('__buf)),
        };

        if let Some(decode_fn) = public_decode_mangling(&decode_fn, &decode_args) {
            new_content.push(syn::parse_quote! {
                #decode_fn
            });
        } else {
            new_content.push(syn::parse_quote! {
                #decode_fn
            });
        }

        let buffer_ty = match buffer_fn.as_ref().map(|f| &f.sig.output) {
            Some(ret @ syn::ReturnType::Type(_, ty)) => {
                if let Some((ty, _)) = unpack_output_result(ret) {
                    ty.clone()
                } else {
                    (**ty).clone()
                }
            }
            _ => syn::parse_quote!(()),
        };

        let provider_field = providers.iter().map(|p| &p.f.sig.ident);
        let provider_ty = providers.iter().map(|p| &p.ty);

        new_content.push(syn::parse_quote! {
            #visibility struct Benchmarker {
                buffer: #buffer_ty,
                #(#provider_field: #provider_ty,)*
            }
        });

        let provider_field = providers.iter().map(|p| &p.f.sig.ident);
        let provider_ty = providers.iter().map(|p| &p.ty);

        new_content.push(syn::parse_quote! {
            #visibility struct State<#buf_lt> {
                buffer: &#buf_lt mut #buffer_ty,
                #(#provider_field: &#buf_lt mut #provider_ty,)*
            }
        });

        let provider_field = providers.iter().map(|p| &p.f.sig.ident);
        let provider_ty = providers.iter().map(|p| &p.ty);

        new_content.push(syn::parse_quote! {
            #visibility struct EncodeState<#buf_param, #(#extra_lts,)*> {
                buffer: #encode_return,
                #(#provider_field: &#buf_lt mut #provider_ty,)*
                _buf: ::core::marker::PhantomData<&#buf_lt ()>,
                #(#extra_markers: ::core::marker::PhantomData<&#extra_lts ()>,)*
            }
        });

        let provider_field = providers.iter().map(|p| &p.f.sig.ident);
        let provider_fns = providers.iter().map(|p| &p.f);

        if let Some(buffer_fn) = buffer_fn {
            let ident = &buffer_fn.sig.ident;

            if let Some((_, error)) = unpack_output_result(&buffer_fn.sig.output) {
                new_content.push(syn::parse_quote! {
                    #[inline(always)]
                    #visibility fn setup() -> Result<Benchmarker, #error> {
                        #buffer_fn
                        #(#provider_fns)*

                        Ok(Benchmarker {
                            buffer: #ident()?,
                            #(#provider_field: #provider_field(),)*
                        })
                    }
                });
            } else {
                new_content.push(syn::parse_quote! {
                    #[inline(always)]
                    #visibility fn setup() -> Result<Benchmarker, ::core::convert::Infallible> {
                        #buffer_fn
                        #(#provider_fns)*

                        Ok(Benchmarker {
                            buffer: #ident(),
                            #(#provider_field: #provider_field(),)*
                        })
                    }
                });
            }
        } else {
            new_content.push(syn::parse_quote! {
                #[inline(always)]
                #visibility fn setup() -> Result<Benchmarker, ::core::convert::Infallible> {
                    Ok(Benchmarker {
                        buffer: (),
                        #(#provider_field: #provider_field(),)*
                    })
                }
            });
        }

        let provider_field = providers.iter().map(|p| &p.f.sig.ident);

        new_content.push(syn::parse_quote! {
            impl Benchmarker {
                #[inline(always)]
                pub fn state(&mut self) -> State<'_> {
                    State {
                        buffer: &mut self.buffer,
                        #(#provider_field: &mut self.#provider_field,)*
                    }
                }
            }
        });

        let reset_item_fn;

        if let Some((reset_fn, args)) = reset_fn {
            let (size_hint, value) = reset_idents(&args);
            let reset_args = convert_arguments(args, ReferenceType::Encode);
            let (reset_generics, reset_param) = mangle_reset_lifetimes(&reset_fn, encode_lifetime);

            let mut item = reset_fn.clone();
            item.sig.inputs =
                syn::parse_quote!(&mut self, #size_hint: usize, #value: &#reset_param);
            item.sig.generics = reset_generics;

            let reset_inner = &reset_fn.sig.ident;

            if unpack_output_result(&reset_fn.sig.output).is_some() {
                item.block = syn::parse_quote! {{
                    #reset_fn
                    #reset_inner(#reset_args)
                }};
            } else {
                item.sig.output = syn::parse_quote!(-> Result<(), ::core::convert::Infallible>);

                item.block = syn::parse_quote! {{
                    #reset_fn
                    #reset_inner(#reset_args);
                    Ok(())
                }};
            }

            reset_item_fn = item;
        } else {
            reset_item_fn = syn::parse_quote! {
                #visibility fn reset<T>(&mut self, _: usize, _: &T) -> Result<(), ::core::convert::Infallible> {
                    Ok(())
                }
            };
        };

        let encode_args = convert_arguments(encode_args, ReferenceType::Encode);
        let (provided_field, provided_expr) = convert_provided(&providers);

        let encode_generics = without_lifetime(
            &encode_fn,
            encode_lifetime
                .as_ref()
                .map(|p| slice::from_ref(&p.lifetime))
                .unwrap_or(&[]),
        );

        let mut encode_inner = syn::PathSegment::from(encode_fn.sig.ident.clone());
        encode_inner.arguments = generics_to_path_arguments(&encode_generics);

        let empty_lts = extra_lts
            .iter()
            .map(|_| syn::Lifetime::new("'_", Span::call_site()));

        let mut encode_item_fn = encode_fn.clone();
        encode_item_fn.sig.generics = encode_generics;
        encode_item_fn.sig.inputs = syn::parse_quote!(&mut self, value: &T);
        encode_item_fn.sig.output =
            syn::parse_quote!(-> Result<EncodeState<'_, #(#empty_lts,)*>, #encode_error>);
        encode_item_fn.block = syn::parse_quote! {
            {
                #encode_fn

                Ok(EncodeState {
                    buffer: #encode_inner(#encode_args)?,
                    #(#provided_field: #provided_expr,)*
                    _buf: ::core::marker::PhantomData,
                    #(#extra_markers: ::core::marker::PhantomData,)*
                })
            }
        };

        new_content.push(syn::parse_quote! {
            impl<'buf> State<'buf> {
                #reset_item_fn
                #encode_item_fn
            }
        });

        let decode_args = convert_arguments(decode_args, ReferenceType::Decode);

        let mut decode_item_fn = decode_fn.clone();

        let mut decode_inner = syn::PathSegment::from(decode_fn.sig.ident.clone());
        decode_inner.arguments = generics_to_path_arguments(&decode_fn.sig.generics);

        decode_item_fn.sig.inputs = syn::parse_quote!(&mut self);
        decode_item_fn.sig.generics =
            mangle_decode_lifetimes(&decode_fn, encode_lifetime, &extra_lts);
        decode_item_fn.block = syn::parse_quote! {{
            #decode_fn
            #decode_inner(#decode_args)
        }};

        let as_bytes_fn = match &attrs.as_bytes {
            AsBytes::Disabled => {
                quote::quote! {
                    #[inline(always)]
                    #visibility fn as_bytes(&self) -> Option<&[u8]> {
                        None
                    }
                }
            }
            AsBytes::Default => {
                quote::quote! {
                    #[inline(always)]
                    #visibility fn as_bytes(&self) -> Option<&[u8]> {
                        Some(&self.buffer)
                    }
                }
            }
            AsBytes::Method(path) => {
                quote::quote! {
                    #[inline(always)]
                    #visibility fn as_bytes(&self) -> Option<&[u8]> {
                        Some(#path(&self.buffer))
                    }
                }
            }
        };

        new_content.push(syn::parse_quote! {
            impl<#buf_param, #(#extra_lts,)*> EncodeState<#buf_lt, #(#extra_lts,)*> {
                #decode_item_fn

                #as_bytes_fn

                #[allow(clippy::len_without_is_empty)]
                #[inline(always)]
                #visibility fn len(&self) -> usize {
                    self.as_bytes().map(|bytes| bytes.len()).unwrap_or_default()
                }
            }
        });

        let enabled = !attrs.disabled;

        new_content.push(syn::parse_quote! {
            /// Indicates if the framework is enabled.
            #visibility fn is_enabled() -> bool {
                #enabled
            }
        });

        self.module_impl.content = Some((brace, new_content));
        Ok(self.module_impl.into_token_stream())
    }
}

fn generics_to_path_arguments(encode_generics: &syn::Generics) -> syn::PathArguments {
    let mut params = Punctuated::<syn::GenericArgument, Token![,]>::new();

    for p in &encode_generics.params {
        let arg = match p {
            syn::GenericParam::Lifetime(..) => continue,
            syn::GenericParam::Type(p) => {
                syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: p.ident.clone().into(),
                }))
            }
            syn::GenericParam::Const(p) => {
                syn::GenericArgument::Const(syn::Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: p.ident.clone().into(),
                }))
            }
        };

        params.push(arg);
    }

    if params.is_empty() {
        return syn::PathArguments::None;
    }

    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
        colon2_token: Some(<Token![::]>::default()),
        lt_token: <Token![<]>::default(),
        args: params,
        gt_token: <Token![>]>::default(),
    })
}

fn reset_idents(arguments: &[Argument]) -> (syn::Ident, syn::Ident) {
    let mut size_hint = syn::Ident::new("size_hint", Span::call_site());
    let mut value = syn::Ident::new("value", Span::call_site());

    for a in arguments {
        match a {
            Argument::SizeHint(ident) => {
                size_hint = ident.clone();
            }
            Argument::Value(ident) => {
                value = ident.clone();
            }
            _ => {}
        }
    }

    (size_hint, value)
}

enum ReferenceType {
    Encode,
    Decode,
}

fn convert_arguments(
    arguments: Vec<Argument>,
    reference: ReferenceType,
) -> Punctuated<syn::Expr, Token![,]> {
    let mut output = Punctuated::<syn::Expr, _>::new();

    for a in arguments {
        match a {
            Argument::Buffer(ident) => match reference {
                ReferenceType::Encode => {
                    output.push(syn::parse_quote_spanned! { ident.span() => self.buffer });
                }
                ReferenceType::Decode => {
                    output.push(syn::parse_quote_spanned! { ident.span() => &self.buffer });
                }
            },
            Argument::Value(value) => {
                output.push(syn::parse_quote! { #value });
            }
            Argument::SizeHint(size_hint) => {
                output.push(syn::parse_quote! { #size_hint });
            }
            Argument::Provided(ident) => match reference {
                ReferenceType::Encode => {
                    output.push(syn::parse_quote_spanned! { ident.span() => &mut self.#ident });
                }
                ReferenceType::Decode => {
                    output.push(syn::parse_quote_spanned! { ident.span() => &mut self.#ident });
                }
            },
        }
    }

    output
}

struct Provider {
    f: syn::ItemFn,
    ty: syn::Type,
}

fn convert_provided(providers: &[Provider]) -> (Vec<syn::Ident>, Vec<syn::Expr>) {
    let mut fields = Vec::new();
    let mut exprs = Vec::new();

    for p in providers {
        let ident = &p.f.sig.ident;
        fields.push(ident.clone());
        exprs.push(syn::parse_quote! { &mut self.#ident });
    }

    (fields, exprs)
}

/// Extract lifetimes in encode function calls so they can be moved to the struct definition.
fn find_buf_param<'item>(
    item_fn: &'item syn::ItemFn,
    found: Option<&syn::Lifetime>,
) -> Option<&'item syn::LifetimeParam> {
    let mut lifetime = None;

    for p in &item_fn.sig.generics.params {
        match p {
            syn::GenericParam::Lifetime(p) if lifetime.is_none() => {
                if found.is_some_and(|lt| *lt != p.lifetime) {
                    continue;
                }

                lifetime = Some(p);
            }
            _ => {}
        }
    }

    lifetime
}

/// Extract lifetimes in reset function calls so they can be moved to the struct definition.
fn mangle_reset_lifetimes(
    item_fn: &syn::ItemFn,
    lifetime: Option<&syn::LifetimeParam>,
) -> (syn::Generics, syn::TypeParam) {
    let mut generics = item_fn.sig.generics.clone();

    let mut input_type = None;

    for p in take(&mut generics.params) {
        match p {
            syn::GenericParam::Lifetime(lt) => {
                if lifetime.is_some_and(|p| p.lifetime == lt.lifetime) {
                    continue;
                }

                generics.params.push(syn::GenericParam::Lifetime(lt));
            }
            syn::GenericParam::Type(ty) => {
                if input_type.is_none() {
                    input_type = Some(ty.clone());
                }

                generics.params.push(syn::GenericParam::Type(ty));
            }
            p => {
                generics.params.push(p);
            }
        }
    }

    let ty = match input_type {
        Some(ty) => ty,
        None => {
            let ty: syn::TypeParam = syn::parse_quote! { T };
            generics.params.push(syn::GenericParam::Type(ty.clone()));
            ty
        }
    };

    (generics, ty)
}

/// Extract lifetimes in decode function calls so they can be moved to the struct definition.
fn mangle_decode_lifetimes(
    item_fn: &syn::ItemFn,
    lifetime: Option<&syn::LifetimeParam>,
    extra_lts: &[syn::Lifetime],
) -> syn::Generics {
    let mut generics = item_fn.sig.generics.clone();

    for p in take(&mut generics.params) {
        match p {
            syn::GenericParam::Lifetime(lt) => {
                if lifetime.is_some_and(|p| p.lifetime == lt.lifetime) {
                    continue;
                }

                if extra_lts.iter().any(|l| l.ident == lt.lifetime.ident) {
                    continue;
                }

                generics.params.push(syn::GenericParam::Lifetime(lt));
            }
            p => {
                generics.params.push(p);
            }
        }
    }

    generics
}

fn without_lifetime(item_fn: &syn::ItemFn, filter_lts: &[syn::Lifetime]) -> syn::Generics {
    let mut generics = item_fn.sig.generics.clone();

    for p in take(&mut generics.params) {
        match p {
            syn::GenericParam::Lifetime(lt) => {
                if filter_lts.iter().any(|l| l.ident == lt.lifetime.ident) {
                    continue;
                }

                generics.params.push(syn::GenericParam::Lifetime(lt));
            }
            p => {
                generics.params.push(p);
            }
        }
    }

    generics
}

fn argument_attrs(
    index: usize,
    arg: &mut syn::FnArg,
    reset_arguments: &mut Vec<Argument>,
    size_hint: bool,
) -> syn::Result<()> {
    fn to_ident(ty: &syn::PatType) -> Option<syn::Ident> {
        let syn::Pat::Ident(ident) = &*ty.pat else {
            return None;
        };

        Some(ident.ident.clone())
    }

    let syn::FnArg::Typed(ty) = arg else {
        return Err(syn::Error::new_spanned(
            arg,
            "Expected argument to be typed",
        ));
    };

    let mut new_attrs = Vec::new();
    let mut argument = None;

    let ident = match to_ident(ty) {
        Some(ident) => ident,
        None => quote::format_ident!("var{}", index),
    };

    for attr in ty.attrs.drain(..) {
        if attr.path().is_ident("value") {
            argument = Some(Argument::Value(ident.clone()));
            continue;
        }

        if attr.path().is_ident("buffer") {
            argument = Some(Argument::Buffer(ident.clone()));
            continue;
        }

        if attr.path().is_ident("size_hint") && size_hint {
            argument = Some(Argument::SizeHint(ident.clone()));
            continue;
        }

        new_attrs.push(attr);
    }

    if argument.is_none()
        && let syn::Pat::Ident(ident) = &*ty.pat
    {
        let ident = &ident.ident;

        if ident == "buf" || ident == "buffer" {
            argument = Some(Argument::Buffer(ident.clone()));
        } else if ident == "size_hint" {
            if size_hint {
                argument = Some(Argument::SizeHint(ident.clone()));
            }
        } else if ident == "value" {
            argument = Some(Argument::Value(ident.clone()));
        } else {
            argument = Some(Argument::Provided(ident.clone()));
        }
    }

    let Some(argument) = argument else {
        return Err(syn::Error::new_spanned(arg, "Unsupported argument"));
    };

    ty.attrs = new_attrs;
    reset_arguments.push(argument);
    Ok(())
}

#[derive(Default)]
struct FunctionAttrs {
    is_provider: bool,
}

fn function_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<FunctionAttrs> {
    let mut output = FunctionAttrs::default();
    let mut new_attrs = Vec::with_capacity(attrs.len());

    for attr in attrs.drain(..) {
        if attr.path().is_ident("provider") {
            output.is_provider = true;
            continue;
        }

        new_attrs.push(attr);
    }

    *attrs = new_attrs;
    Ok(output)
}

enum Argument {
    Buffer(syn::Ident),
    Value(syn::Ident),
    SizeHint(syn::Ident),
    Provided(syn::Ident),
}

fn unpack_output_result(ret: &syn::ReturnType) -> Option<(&syn::Type, &syn::Type)> {
    let syn::ReturnType::Type(_, ty) = ret else {
        return None;
    };

    let syn::Type::Path(syn::TypePath { path, .. }) = &**ty else {
        return None;
    };

    let syn::Path { segments, .. } = path;
    let syn::PathSegment { ident, arguments } = segments.first()?;

    if ident != "Result" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) =
        arguments
    else {
        return None;
    };

    let mut it = args.iter();

    let syn::GenericArgument::Type(a) = it.next()? else {
        return None;
    };

    let syn::GenericArgument::Type(b) = it.next()? else {
        return None;
    };

    Some((a, b))
}

fn public_decode_mangling(decode_fn: &syn::ItemFn, arguments: &[Argument]) -> Option<syn::ItemFn> {
    if arguments.is_empty() {
        return None;
    }

    let mut new_inputs = Punctuated::new();
    let mut inner_arguments = Punctuated::<syn::Expr, Token![,]>::new();
    let mut needs_b = false;

    for (a, i) in arguments.iter().zip(&decode_fn.sig.inputs) {
        match a {
            Argument::Provided(ident) => {
                needs_b = true;
                inner_arguments.push(syn::parse_quote!(&mut b.#ident));
                continue;
            }
            Argument::Buffer(..) => {}
            _ => {
                return None;
            }
        }

        let mut new_input = i.clone();

        let syn::FnArg::Typed(ty) = &mut new_input else {
            return None;
        };

        let syn::Pat::Ident(syn::PatIdent {
            ident, mutability, ..
        }) = &mut *ty.pat
        else {
            return None;
        };

        inner_arguments.push(syn::parse_quote!(#ident));
        *mutability = None;
        new_inputs.push(new_input);
    }

    let mut inner_fn_ident = syn::PathSegment::from(decode_fn.sig.ident.clone());
    inner_fn_ident.arguments = generics_to_path_arguments(&decode_fn.sig.generics);

    let needs_b = needs_b.then(|| {
        quote::quote! {
            let mut b = match self::setup() {
                Ok(b) => b,
                Err(error) => match error {},
            };
        }
    });

    let mut outer_fn = decode_fn.clone();

    outer_fn.sig.inputs = new_inputs;
    outer_fn.block = syn::parse_quote! {{
        #decode_fn
        #needs_b;
        #inner_fn_ident(#inner_arguments)
    }};

    Some(outer_fn)
}
