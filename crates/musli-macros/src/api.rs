use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::ParseStream;
use syn::Token;

/// Expand endpoint.
pub(super) fn endpoint(
    input: syn::DeriveInput,
    crate_name: &str,
    module_name: &str,
) -> syn::Result<TokenStream> {
    let mut crate_path = None;
    let mut response = None;
    let mut response_lt = None;

    for attr in input.attrs {
        if !attr.path().is_ident("endpoint") {
            continue;
        }

        attr.parse_args_with(|p: ParseStream<'_>| {
            while !p.is_empty() {
                let path = p.parse::<syn::Path>()?;

                if let Some(lt) = as_ident(&path, "response") {
                    p.parse::<Token![=]>()?;
                    response = Some(p.parse::<syn::Type>()?);
                    response_lt = lt;
                } else if path.is_ident("crate") {
                    parse_crate(p, &mut crate_path)?;
                } else {
                    return Err(syn::Error::new_spanned(path, "unknown attribute"));
                }

                if p.parse::<Option<Token![,]>>()?.is_none() {
                    break;
                }
            }

            Ok(())
        })?;
    }

    let crate_path = match crate_path {
        Some(path) => path,
        None => syn::Path::from(syn::PathSegment::from(syn::Ident::new(
            crate_name,
            Span::call_site(),
        ))),
    };

    let endpoint_t = path(&crate_path, [module_name, "Endpoint"]);

    let Some(response) = response else {
        return Err(syn::Error::new(
            Span::call_site(),
            "missing `#[endpoint(response = <ty>)]` attribute",
        ));
    };

    let lt = match response_lt {
        Some(lt) => lt,
        None => syn::Lifetime::new("'__de", Span::call_site()),
    };

    let ident = &input.ident;
    let name = name_from_ident(&ident.to_string());

    Ok(quote! {
        #[automatically_derived]
        impl #endpoint_t for #ident {
            const KIND: &str = #name;
            type Response<#lt> = #response;
            fn __do_not_implement() {}
        }
    })
}

/// Expand request impl.
pub(super) fn request(
    input: syn::DeriveInput,
    crate_name: &str,
    module_name: &str,
) -> syn::Result<TokenStream> {
    let mut crate_path = None;
    let mut endpoint = None;

    for attr in input.attrs {
        if !attr.path().is_ident("request") {
            continue;
        }

        attr.parse_args_with(|p: ParseStream<'_>| {
            while !p.is_empty() {
                let path = p.parse::<syn::Path>()?;

                if let Some(lt) = as_ident(&path, "endpoint") {
                    if let Some(lt) = lt {
                        return Err(syn::Error::new_spanned(
                            lt,
                            "lifetimes are not supported for endpoints",
                        ));
                    }

                    p.parse::<Token![=]>()?;
                    endpoint = Some(p.parse::<syn::Type>()?);
                } else if path.is_ident("crate") {
                    parse_crate(p, &mut crate_path)?;
                } else {
                    return Err(syn::Error::new_spanned(path, "unknown attribute"));
                }

                if p.parse::<Option<Token![,]>>()?.is_none() {
                    break;
                }
            }

            Ok(())
        })?;
    }

    let crate_path = match crate_path {
        Some(path) => path,
        None => syn::Path::from(syn::PathSegment::from(syn::Ident::new(
            crate_name,
            Span::call_site(),
        ))),
    };

    let request_t = path(&crate_path, [module_name, "Request"]);

    let Some(endpoint) = endpoint else {
        return Err(syn::Error::new(
            Span::call_site(),
            "missing `#[request(endpoint = <ty>)]` attribute",
        ));
    };

    let ident = &input.ident;

    Ok(quote! {
        #[automatically_derived]
        impl #request_t for #ident {
            type Endpoint = #endpoint;
            fn __do_not_implement() {}
        }
    })
}

fn parse_crate(p: ParseStream<'_>, crate_path: &mut Option<syn::Path>) -> syn::Result<()> {
    if let Some(existing) = crate_path {
        return Err(syn::Error::new_spanned(
            existing,
            "duplicate `crate` attribute",
        ));
    }

    *crate_path = if p.parse::<Option<Token![=]>>()?.is_some() {
        Some(p.parse::<syn::Path>()?)
    } else {
        Some(syn::Path::from(syn::PathSegment::from(
            <Token![crate]>::default(),
        )))
    };

    Ok(())
}

fn as_ident(path: &syn::Path, expect: &str) -> Option<Option<syn::Lifetime>> {
    let one = path.segments.first()?;

    if path.segments.len() != 1 || path.leading_colon.is_some() {
        return None;
    }

    if one.ident != expect {
        return None;
    }

    match &one.arguments {
        syn::PathArguments::AngleBracketed(lt) => {
            let first = lt.args.first()?;

            if lt.args.len() != 1 {
                return None;
            }

            match first {
                syn::GenericArgument::Lifetime(lt) => Some(Some(lt.clone())),
                _ => None,
            }
        }
        syn::PathArguments::None => Some(None),
        _ => None,
    }
}

fn name_from_ident(ident: &str) -> String {
    let mut name = String::with_capacity(ident.len());

    for c in ident.chars() {
        if c.is_uppercase() && !name.is_empty() {
            name.push('-');
        }

        name.extend(c.to_lowercase());
    }

    name
}

fn path<const N: usize>(base: &syn::Path, segments: [&str; N]) -> syn::Path {
    let mut path = base.clone();

    for segment in segments {
        path.segments
            .push(syn::Ident::new(segment, Span::call_site()).into());
    }

    path
}
