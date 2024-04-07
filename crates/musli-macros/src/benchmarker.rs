use std::mem::take;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;

enum AsBytes {
    Default,
    Disabled,
    Method(syn::Path),
}

pub(super) struct Attributes {
    as_bytes: AsBytes,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut end = false;
        let mut as_bytes = AsBytes::Default;

        while !input.is_empty() {
            let path: syn::Path = input.parse()?;

            if path.is_ident("as_bytes_disabled") {
                as_bytes = AsBytes::Disabled;
            } else if path.is_ident("as_bytes") {
                input.parse::<Token![=]>()?;
                as_bytes = AsBytes::Method(input.parse()?);
            } else {
                return Err(syn::Error::new_spanned(path, "Unsupported attribute"));
            }

            if end {
                break;
            }

            end = input.parse::<Option<Token![,]>>()?.is_none();
        }

        Ok(Attributes { as_bytes })
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

        let (encode_generics, lifetime) = mangle_encode_lifetimes(&encode_fn);

        let type_lt = match &lifetime {
            Some(param) => param.lifetime.clone(),
            None => syn::parse_quote!('__buf),
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
            Some(syn::ReturnType::Type(_, ty)) => *ty.clone(),
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
            #visibility struct State<#type_lt> {
                buffer: &#type_lt mut #buffer_ty,
                #(#provider_field: &#type_lt mut #provider_ty,)*
            }
        });

        let provider_field = providers.iter().map(|p| &p.f.sig.ident);
        let provider_ty = providers.iter().map(|p| &p.ty);

        new_content.push(syn::parse_quote! {
            #visibility struct EncodeState<#type_lt> {
                buffer: #encode_return,
                #(#provider_field: &#type_lt mut #provider_ty,)*
                _marker: ::core::marker::PhantomData<&#type_lt ()>,
            }
        });

        let provider_field = providers.iter().map(|p| &p.f.sig.ident);
        let provider_fns = providers.iter().map(|p| &p.f);

        if let Some(buffer_fn) = buffer_fn {
            let buffer_fn_call = &buffer_fn.sig.ident;

            new_content.push(syn::parse_quote! {
                #[inline(always)]
                #visibility fn new() -> Benchmarker {
                    #buffer_fn
                    #(#provider_fns)*

                    Benchmarker {
                        buffer: #buffer_fn_call(),
                        #(#provider_field: #provider_field(),)*
                    }
                }
            });
        } else {
            new_content.push(syn::parse_quote! {
                #[inline(always)]
                #visibility fn new() -> Benchmarker {
                    Benchmarker {
                        buffer: (),
                        #(#provider_field: #provider_field(),)*
                    }
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

        let reset_item_fn = if let Some((reset_fn, reset_args)) = reset_fn {
            let reset_inner = &reset_fn.sig.ident;
            let (size_hint, value) = reset_idents(&reset_args);
            let reset_args = convert_arguments(reset_args, ReferenceType::Encode);
            let (reset_generics, reset_param) =
                mangle_reset_lifetimes(&reset_fn, lifetime.as_ref());
            let mut reset_item_fn = reset_fn.clone();
            reset_item_fn.sig.inputs =
                syn::parse_quote!(&mut self, #size_hint: usize, #value: &#reset_param);
            reset_item_fn.sig.generics = reset_generics;
            reset_item_fn.block = syn::parse_quote! {
                {
                    #reset_fn
                    #reset_inner(#reset_args)
                }
            };

            reset_item_fn
        } else {
            syn::parse_quote! {
                #visibility fn reset<T>(&mut self, _: usize, _: &T) {}
            }
        };

        let encode_inner = &encode_fn.sig.ident;
        let encode_args = convert_arguments(encode_args, ReferenceType::Encode);
        let (provided_field, provided_expr) = convert_provided(&providers);

        let mut encode_item_fn = encode_fn.clone();
        encode_item_fn.sig.generics = encode_generics;
        encode_item_fn.sig.inputs = syn::parse_quote!(&mut self, value: &T);
        encode_item_fn.sig.output = syn::parse_quote!(-> Result<EncodeState<'_>, #encode_error>);
        encode_item_fn.block = syn::parse_quote! {
            {
                #encode_fn

                Ok(EncodeState {
                    buffer: #encode_inner(#encode_args)?,
                    #(#provided_field: #provided_expr,)*
                    _marker: ::core::marker::PhantomData,
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

        let decode_inner = &decode_fn.sig.ident;
        let mut decode_item_fn = decode_fn.clone();

        decode_item_fn.sig.inputs = syn::parse_quote!(&mut self);
        decode_item_fn.sig.generics = mangle_decode_lifetimes(&decode_fn, lifetime.as_ref());
        decode_item_fn.block = syn::parse_quote! {
            {
                #decode_fn
                #decode_inner(#decode_args)
            }
        };

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
            impl<'buf> EncodeState<'buf> {
                #decode_item_fn

                #as_bytes_fn

                #[allow(clippy::len_without_is_empty)]
                #[inline(always)]
                #visibility fn len(&self) -> usize {
                    self.as_bytes().map(|bytes| bytes.len()).unwrap_or_default()
                }
            }
        });

        self.module_impl.content = Some((brace, new_content));
        Ok(self.module_impl.into_token_stream())
    }
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
fn mangle_encode_lifetimes(item_fn: &syn::ItemFn) -> (syn::Generics, Option<syn::LifetimeParam>) {
    let mut generics = item_fn.sig.generics.clone();
    let mut lifetime = None;

    for p in take(&mut generics.params) {
        match p {
            syn::GenericParam::Lifetime(lt) if lifetime.is_none() => {
                lifetime = Some(lt);
            }
            p => {
                generics.params.push(p);
            }
        }
    }

    (generics, lifetime)
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
                if lifetime.map_or(false, |p| p.lifetime == lt.lifetime) {
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
) -> syn::Generics {
    let mut generics = item_fn.sig.generics.clone();

    for p in take(&mut generics.params) {
        match p {
            syn::GenericParam::Lifetime(lt) => {
                if lifetime.map_or(false, |p| p.lifetime == lt.lifetime) {
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

    if argument.is_none() {
        if let syn::Pat::Ident(ident) = &*ty.pat {
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

fn public_decode_mangling(item_fn: &syn::ItemFn, arguments: &[Argument]) -> Option<syn::ItemFn> {
    if arguments.is_empty() {
        return None;
    }

    let mut new_inputs = Punctuated::new();
    let mut inner_arguments = Punctuated::<syn::Expr, Token![,]>::new();

    for (a, i) in arguments.iter().zip(&item_fn.sig.inputs) {
        match a {
            Argument::Provided(ident) => {
                inner_arguments.push(syn::parse_quote!(&mut b.#ident));
                continue;
            }
            Argument::Buffer(..) => {}
            _ => {
                return None;
            }
        }

        new_inputs.push(i.clone());

        let syn::FnArg::Typed(ty) = i else {
            return None;
        };

        let syn::Pat::Ident(syn::PatIdent { ident, .. }) = &*ty.pat else {
            return None;
        };

        inner_arguments.push(syn::parse_quote!(#ident));
    }

    let inner_fn_ident = &item_fn.sig.ident;

    let mut outer_fn = item_fn.clone();
    outer_fn.sig.inputs = new_inputs;
    outer_fn.block = syn::parse_quote! {
        {
            #item_fn

            let mut b = new();
            #inner_fn_ident(#inner_arguments)
        }
    };

    Some(outer_fn)
}
