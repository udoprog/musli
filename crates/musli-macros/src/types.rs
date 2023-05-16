use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
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

pub(super) const ENCODER_TYPES: [(&str, Option<Ty>); 8] = [
    ("Error", None),
    ("Some", None),
    ("Pack", None),
    ("Sequence", None),
    ("Tuple", None),
    ("Map", None),
    ("Struct", None),
    ("Variant", None),
];

pub(super) const DECODER_TYPES: [(&str, Option<Ty>); 9] = [
    ("Error", None),
    ("Buffer", None),
    ("Some", None),
    ("Pack", None),
    ("Sequence", None),
    ("Tuple", None),
    ("Map", None),
    ("Struct", None),
    ("Variant", None),
];

pub(super) const VISITOR_TYPES: [(&str, Option<Ty>); 3] = [
    ("String", Some(Ty::Str)),
    ("Bytes", Some(Ty::Bytes)),
    ("Number", None),
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
        types: &[(&str, Option<Ty>)],
        arguments: [&str; N],
        hint: &str,
    ) -> syn::Result<TokenStream> {
        let mut missing = types
            .iter()
            .map(|(ident, ty)| (syn::Ident::new(ident, Span::call_site()), *ty))
            .collect::<BTreeMap<_, _>>();

        // List of associated types which are specified, but under a `cfg`
        // attribute so its conditions need to be inverted.
        let mut not_attribute_ty = Vec::new();

        for item in &self.item_impl.items {
            match item {
                syn::ImplItem::Type(impl_type) => {
                    let Some(ty) = missing.remove(&impl_type.ident) else {
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

                        not_attribute_ty.push((impl_type.clone(), ty));
                        has_cfg = true;
                    }
                }
                _ => continue,
            }
        }

        for (mut impl_type, ty) in not_attribute_ty {
            for attr in &mut impl_type.attrs {
                if !attr.path().is_ident("cfg") {
                    continue;
                }

                if let syn::Meta::List(m) = &mut attr.meta {
                    let tokens = syn::Meta::List(syn::MetaList {
                        path: not_path(),
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: m.tokens.clone(),
                    })
                    .into_token_stream();

                    m.tokens = tokens;
                }
            }

            impl_type.ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: never_type(arguments, ty),
            });

            self.item_impl.items.push(syn::ImplItem::Type(impl_type));
        }

        for (ident, ty) in missing {
            let never = never_type(arguments, ty);

            let ty = syn::ImplItemType {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                defaultness: None,
                type_token: <Token![type]>::default(),
                ident,
                generics: syn::Generics::default(),
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

fn not_path() -> syn::Path {
    let mut not_path = syn::Path {
        leading_colon: None,
        segments: Punctuated::default(),
    };

    not_path
        .segments
        .push(syn::PathSegment::from(syn::Ident::new(
            "not",
            Span::call_site(),
        )));

    not_path
}

fn never_type<const N: usize>(arguments: [&str; N], ty: Option<Ty>) -> syn::Path {
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

        if let Some(ty) = ty {
            match ty {
                Ty::Str => {
                    let mut path = syn::Path {
                        leading_colon: None,
                        segments: Punctuated::default(),
                    };

                    path.segments.push(syn::PathSegment::from(syn::Ident::new(
                        "str",
                        Span::call_site(),
                    )));

                    args.push(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                        qself: None,
                        path,
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
                            elem: Box::new(syn::Type::Path(syn::TypePath { qself: None, path })),
                        },
                    )));
                }
            }
        }

        s.arguments = syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: <Token![<]>::default(),
            args,
            gt_token: <Token![>]>::default(),
        });

        s
    });
    never
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
