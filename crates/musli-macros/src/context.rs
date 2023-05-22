use core::mem::replace;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Token;

pub(super) struct Context {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    sig: syn::Signature,
    rest: TokenStream,
}

impl Parse for Context {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(syn::Attribute::parse_outer)?,
            vis: input.parse()?,
            sig: input.parse()?,
            rest: input.parse()?,
        })
    }
}

impl Context {
    pub(crate) fn expand(mut self) -> syn::Result<TokenStream> {
        let c_param = syn::Ident::new("C", Span::call_site());
        let error_param = syn::Ident::new("Error", Span::call_site());
        let input_param = syn::Ident::new("Input", Span::call_site());
        let buf_lifetime = syn::Lifetime::new("'buf", Span::call_site());
        let context_param = syn::Ident::new("Context", Span::call_site());

        let found_param = self
            .sig
            .generics
            .params
            .iter()
            .any(|p| matches!(p, syn::GenericParam::Type(ty) if ty.ident == c_param));

        if !found_param {
            let span = if self.sig.generics.lt_token.is_some()
                || self.sig.generics.gt_token.is_some()
                || !self.sig.generics.params.is_empty()
            {
                self.sig.generics.to_token_stream()
            } else {
                self.sig.ident.to_token_stream()
            };

            return Err(syn::Error::new_spanned(
                span,
                "expected one parameter named 'C'",
            ));
        }

        let context_path = {
            let mut segments = Punctuated::default();

            segments.push(syn::PathSegment::from(context_param));

            syn::Path {
                leading_colon: None,
                segments,
            }
        };

        let c_type = syn::Type::Path(syn::TypePath {
            qself: None,
            path: {
                let mut segments = Punctuated::default();

                segments.push(syn::PathSegment::from(c_param.clone()));

                syn::Path {
                    leading_colon: None,
                    segments,
                }
            },
        });

        let c_return = syn::Type::Path(syn::TypePath {
            qself: None,
            path: {
                let mut segments = Punctuated::default();

                segments.push(syn::PathSegment::from(c_param.clone()));
                segments.push(syn::PathSegment::from(error_param.clone()));

                syn::Path {
                    leading_colon: None,
                    segments,
                }
            },
        });

        let Some(error_type) = modify_return(&mut self.sig, &c_return) else {
            return Err(syn::Error::new_spanned(&self.sig, "return type must end in 'Result<T, E>'"));
        };

        self.sig
            .generics
            .params
            .push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                attrs: Vec::new(),
                lifetime: buf_lifetime.clone(),
                colon_token: Some(<Token![:]>::default()),
                bounds: Punctuated::default(),
            }));

        if !found_param {
            self.sig
                .generics
                .params
                .push(syn::GenericParam::Type(syn::TypeParam {
                    attrs: Vec::new(),
                    ident: c_param,
                    colon_token: Some(<Token![:]>::default()),
                    bounds: Punctuated::default(),
                    eq_token: None,
                    default: None,
                }));
        }

        let where_clause = self.sig.generics.make_where_clause();

        where_clause
            .predicates
            .push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                bounded_ty: c_type,
                colon_token: <Token![:]>::default(),
                bounds: {
                    let mut path = context_path.clone();

                    if let Some(last) = path.segments.last_mut() {
                        let mut args = Punctuated::default();

                        args.push(syn::GenericArgument::Lifetime(buf_lifetime));

                        args.push(syn::GenericArgument::AssocType(syn::AssocType {
                            ident: input_param.clone(),
                            generics: None,
                            eq_token: <Token![=]>::default(),
                            ty: error_type,
                        }));

                        last.arguments = syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: <Token![<]>::default(),
                                args,
                                gt_token: <Token![>]>::default(),
                            },
                        );
                    }

                    let mut bounds = Punctuated::default();

                    bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path,
                    }));

                    bounds
                },
            }));

        let mut tokens = TokenStream::default();

        for attr in &self.attrs {
            attr.to_tokens(&mut tokens);
        }

        self.vis.to_tokens(&mut tokens);
        self.sig.to_tokens(&mut tokens);
        self.rest.to_tokens(&mut tokens);
        Ok(tokens)
    }
}

fn modify_return(sig: &mut syn::Signature, c_return: &syn::Type) -> Option<syn::Type> {
    let syn::ReturnType::Type(_, ret) = &mut sig.output else {
        return None;
    };

    let syn::Type::Path(ty) = &mut **ret else {
        return None;
    };

    let result = ty.path.segments.last_mut()?;

    if result.ident != "Result" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &mut result.arguments else {
        return None;
    };

    if args.args.len() != 2 {
        return None;
    }

    let syn::GenericArgument::Type(last) = args.args.last_mut()? else {
        return None;
    };

    Some(replace(last, c_return.clone()))
}
