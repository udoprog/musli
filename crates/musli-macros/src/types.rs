use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Token;

pub(super) const ENCODER_TYPES: [&str; 8] = [
    "Error", "Some", "Pack", "Sequence", "Tuple", "Map", "Struct", "Variant",
];

pub(super) const DECODER_TYPES: [&str; 9] = [
    "Error", "Buffer", "Some", "Pack", "Sequence", "Tuple", "Map", "Struct", "Variant",
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
        types: &[&str],
        arguments: [&str; N],
        hint: &str,
    ) -> syn::Result<TokenStream> {
        let mut missing = types
            .iter()
            .map(|ident| syn::Ident::new(ident, Span::call_site()))
            .collect::<BTreeSet<_>>();

        // List of associated types which are specified, but under a `cfg`
        // attribute so its conditions need to be inverted.
        let mut not_attribute_ty = Vec::new();

        for item in &self.item_impl.items {
            match item {
                syn::ImplItem::Type(ty) => {
                    missing.remove(&ty.ident);

                    let mut has_cfg = false;

                    for attr in &ty.attrs {
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

                        not_attribute_ty.push(ty.clone());
                        has_cfg = true;
                    }
                }
                _ => continue,
            }
        }

        let never = never_type(arguments);

        for mut ty in not_attribute_ty {
            for attr in &mut ty.attrs {
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

            ty.ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: never.clone(),
            });

            self.item_impl.items.push(syn::ImplItem::Type(ty));
        }

        for ident in missing {
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

fn never_type<const N: usize>(arguments: [&str; N]) -> syn::Path {
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
