use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Token;

#[derive(Debug, Clone, Copy)]
pub(super) enum Ty {
    /// `str`.
    Str,
    /// `[u8]`.
    Bytes,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Extra {
    None,
    Context,
    Visitor(Option<Ty>),
}

pub(super) const ENCODER_TYPES: &[(&str, Extra)] = &[
    ("Error", Extra::None),
    ("Some", Extra::None),
    ("Pack", Extra::Context),
    ("Sequence", Extra::None),
    ("Tuple", Extra::None),
    ("Map", Extra::None),
    ("MapPairs", Extra::None),
    ("Struct", Extra::None),
    ("Variant", Extra::None),
    ("TupleVariant", Extra::None),
    ("StructVariant", Extra::None),
];

pub(super) const DECODER_TYPES: &[(&str, Extra)] = &[
    ("Error", Extra::None),
    ("Buffer", Extra::None),
    ("Some", Extra::None),
    ("Pack", Extra::None),
    ("Sequence", Extra::None),
    ("Tuple", Extra::None),
    ("Map", Extra::None),
    ("MapPairs", Extra::None),
    ("Struct", Extra::None),
    ("StructPairs", Extra::None),
    ("Variant", Extra::None),
];

pub(super) const VISITOR_TYPES: &[(&str, Extra)] = &[
    ("String", Extra::Visitor(Some(Ty::Str))),
    ("Bytes", Extra::Visitor(Some(Ty::Bytes))),
    ("Number", Extra::Visitor(None)),
];

pub(super) struct Types {
    item_impl: syn::ItemImpl,
}

impl Parse for Types {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            item_impl: input.parse()?,
        })
    }
}

impl Types {
    /// Expand encoder types.
    pub(crate) fn expand<const N: usize>(
        mut self,
        what: &str,
        types: &[(&str, Extra)],
        arguments: [&str; N],
        hint: &str,
    ) -> syn::Result<TokenStream> {
        let mut missing = types
            .iter()
            .map(|(ident, extra)| (syn::Ident::new(ident, Span::call_site()), *extra))
            .collect::<BTreeMap<_, _>>();

        // List of associated types which are specified, but under a `cfg`
        // attribute so its conditions need to be inverted.
        let mut not_attribute_ty = Vec::new();

        for item in &self.item_impl.items {
            match item {
                syn::ImplItem::Type(impl_type) => {
                    let Some(extra) = missing.remove(&impl_type.ident) else {
                        continue;
                    };

                    let mut has_cfg = false;

                    for attr in &impl_type.attrs {
                        if !attr.path().is_ident("cfg") {
                            continue;
                        }

                        if has_cfg {
                            return Err(syn::Error::new_spanned(
                                attr,
                                format_args!(
                                    "#[rune::{what}]: only one cfg attribute is supported"
                                ),
                            ));
                        }

                        not_attribute_ty.push((impl_type.clone(), extra));
                        has_cfg = true;
                    }
                }
                _ => continue,
            }
        }

        for (mut impl_type, extra) in not_attribute_ty {
            for attr in &mut impl_type.attrs {
                if !attr.path().is_ident("cfg") {
                    continue;
                }

                if let syn::Meta::List(m) = &mut attr.meta {
                    let tokens = syn::Meta::List(syn::MetaList {
                        path: ident_path(syn::Ident::new("not", Span::call_site())),
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: m.tokens.clone(),
                    })
                    .into_token_stream();

                    m.tokens = tokens;
                }
            }

            impl_type.ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: never_type(arguments, extra, &impl_type.generics)?,
            });

            self.item_impl.items.push(syn::ImplItem::Type(impl_type));
        }

        for (ident, extra) in missing {
            let generics = match extra {
                Extra::Context => {
                    let c_param = syn::Ident::new("C", Span::call_site());
                    let this_lifetime = syn::Lifetime::new("'this", Span::call_site());

                    let mut where_clause = syn::WhereClause {
                        where_token: <Token![where]>::default(),
                        predicates: Punctuated::default(),
                    };

                    where_clause.predicates.push(
                        syn::parse_quote!(#c_param: #this_lifetime + musli::context::Context),
                    );

                    let mut params = Punctuated::default();

                    params.push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                        attrs: Vec::new(),
                        lifetime: this_lifetime,
                        colon_token: None,
                        bounds: Punctuated::default(),
                    }));

                    params.push(syn::GenericParam::Type(syn::TypeParam {
                        attrs: Vec::new(),
                        ident: c_param,
                        colon_token: None,
                        bounds: Punctuated::default(),
                        eq_token: None,
                        default: None,
                    }));

                    syn::Generics {
                        lt_token: Some(<Token![<]>::default()),
                        params,
                        gt_token: Some(<Token![>]>::default()),
                        where_clause: Some(where_clause),
                    }
                }
                Extra::Visitor(..) => {
                    let mut where_clause = syn::WhereClause {
                        where_token: <Token![where]>::default(),
                        predicates: Punctuated::default(),
                    };

                    let c_param: syn::Ident = syn::Ident::new("C", Span::call_site());

                    let mut predicate = syn::PredicateType {
                        lifetimes: None,
                        bounded_ty: syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: ident_path(c_param.clone()),
                        }),
                        colon_token: <Token![:]>::default(),
                        bounds: Punctuated::default(),
                    };

                    predicate.bounds.push(syn::TypeParamBound::Verbatim(quote!(
                        musli::Context<Input = Self::Error>
                    )));

                    where_clause
                        .predicates
                        .push(syn::WherePredicate::Type(predicate));

                    let mut params = Punctuated::default();

                    params.push(syn::GenericParam::Type(syn::TypeParam {
                        attrs: Vec::new(),
                        ident: c_param,
                        colon_token: None,
                        bounds: Punctuated::default(),
                        eq_token: None,
                        default: None,
                    }));

                    syn::Generics {
                        lt_token: Some(<Token![<]>::default()),
                        params,
                        gt_token: Some(<Token![>]>::default()),
                        where_clause: Some(where_clause),
                    }
                }
                Extra::None => syn::Generics::default(),
            };

            let never = never_type(arguments, extra, &generics)?;

            let ty = syn::ImplItemType {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                defaultness: None,
                type_token: <Token![type]>::default(),
                ident,
                generics,
                eq_token: <Token![=]>::default(),
                ty: syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: never.clone(),
                }),
                semi_token: <Token![;]>::default(),
            };

            self.item_impl.items.push(syn::ImplItem::Type(ty));
        }

        self.item_impl
            .items
            .push(syn::ImplItem::Type(syn::ImplItemType {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                defaultness: None,
                type_token: <Token![type]>::default(),
                ident: syn::Ident::new(hint, Span::call_site()),
                generics: syn::Generics::default(),
                eq_token: <Token![=]>::default(),
                ty: syn::Type::Tuple(syn::TypeTuple {
                    paren_token: <syn::token::Paren>::default(),
                    elems: Punctuated::default(),
                }),
                semi_token: <Token![;]>::default(),
            }));

        Ok(self.item_impl.into_token_stream())
    }
}

fn ident_path(ident: syn::Ident) -> syn::Path {
    let mut not_path = syn::Path {
        leading_colon: None,
        segments: Punctuated::default(),
    };

    not_path.segments.push(syn::PathSegment::from(ident));

    not_path
}

fn never_type<const N: usize>(
    arguments: [&str; N],
    extra: Extra,
    generics: &syn::Generics,
) -> syn::Result<syn::Path> {
    let mut never = syn::Path {
        leading_colon: None,
        segments: Punctuated::default(),
    };

    never.segments.push(syn::PathSegment::from(syn::Ident::new(
        "musli",
        Span::call_site(),
    )));
    never.segments.push(syn::PathSegment::from(syn::Ident::new(
        "never",
        Span::call_site(),
    )));

    never.segments.push({
        let mut s = syn::PathSegment::from(syn::Ident::new("Never", Span::call_site()));

        let mut args = Punctuated::default();

        for arg in arguments {
            args.push(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                qself: None,
                path: self_type(arg),
            })));
        }

        if let Extra::Visitor(ty) = extra {
            let mut it = generics.params.iter();

            let Some(syn::GenericParam::Type(syn::TypeParam { ident: c_param, .. })) = it.next()
            else {
                return Err(syn::Error::new_spanned(
                    generics,
                    "Missing generic parameter in associated type (usually `C`)",
                ));
            };

            if let Some(ty) = ty {
                match ty {
                    Ty::Str => {
                        args.push(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: ident_path(syn::Ident::new("str", Span::call_site())),
                        })));
                    }
                    Ty::Bytes => {
                        let mut path = syn::Path {
                            leading_colon: None,
                            segments: Punctuated::default(),
                        };

                        path.segments.push(syn::PathSegment::from(syn::Ident::new(
                            "u8",
                            Span::call_site(),
                        )));

                        args.push(syn::GenericArgument::Type(syn::Type::Slice(
                            syn::TypeSlice {
                                bracket_token: syn::token::Bracket::default(),
                                elem: Box::new(syn::Type::Path(syn::TypePath {
                                    qself: None,
                                    path,
                                })),
                            },
                        )));
                    }
                }
            }

            args.push(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                qself: None,
                path: ident_path(c_param.clone()),
            })));
        }

        s.arguments = syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: <Token![<]>::default(),
            args,
            gt_token: <Token![>]>::default(),
        });

        s
    });

    Ok(never)
}

fn self_type(what: &str) -> syn::Path {
    let mut self_error = syn::Path {
        leading_colon: None,
        segments: Punctuated::default(),
    };

    self_error
        .segments
        .push(syn::PathSegment::from(syn::Ident::new(
            "Self",
            Span::call_site(),
        )));

    self_error
        .segments
        .push(syn::PathSegment::from(syn::Ident::new(
            what,
            Span::call_site(),
        )));

    self_error
}
