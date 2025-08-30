use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
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
    /// `type Type = Never;`
    None,
    /// `fn cx(&self) -> Self::Cx`.
    CxFn,
    /// `type Cx = C;`.
    Cx,
    /// `type Error = <Self::Cx as Context>::Error;`
    Error,
    /// `type Mode = M;`
    Mode,
    /// `fn try_clone(&self) -> Option<Self::TryClone>`.
    TryCloneFn,
    /// `type Allocator = <Self::Cx as Context>::Allocator;`
    Allocator,
    /// An associated visitor type.
    Visitor(Ty),
}

pub(super) const ENCODER_TYPES: &[(&str, Extra)] = &[
    ("Cx", Extra::Cx),
    ("cx", Extra::CxFn),
    ("Error", Extra::Error),
    ("Mode", Extra::Mode),
    ("EncodeSome", Extra::None),
    ("EncodePack", Extra::None),
    ("EncodeSequence", Extra::None),
    ("EncodeMap", Extra::None),
    ("EncodeMapEntries", Extra::None),
    ("EncodeVariant", Extra::None),
    ("EncodeSequenceVariant", Extra::None),
    ("EncodeMapVariant", Extra::None),
];

pub(super) const DECODER_TYPES: &[(&str, Extra)] = &[
    ("Cx", Extra::Cx),
    ("cx", Extra::CxFn),
    ("Error", Extra::Error),
    ("Mode", Extra::Mode),
    ("TryClone", Extra::None),
    ("try_clone", Extra::TryCloneFn),
    ("Allocator", Extra::Allocator),
    ("DecodeBuffer", Extra::None),
    ("DecodeSome", Extra::None),
    ("DecodePack", Extra::None),
    ("DecodeSequence", Extra::None),
    ("DecodeMap", Extra::None),
    ("DecodeMapEntries", Extra::None),
    ("DecodeVariant", Extra::None),
];

pub(super) const VISITOR_TYPES: &[(&str, Extra)] = &[
    ("Error", Extra::Error),
    ("Allocator", Extra::Allocator),
    ("String", Extra::Visitor(Ty::Str)),
    ("Bytes", Extra::Visitor(Ty::Bytes)),
];

pub(super) const UNSIZED_VISITOR_TYPES: &[(&str, Extra)] =
    &[("Error", Extra::Error), ("Allocator", Extra::Allocator)];

#[derive(Clone, Copy)]
pub(super) enum Kind {
    SelfCx,
    GenericCx,
}

pub(super) struct Attr {
    crate_path: Option<syn::Path>,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut crate_path = None;
        let mut done = false;

        while !input.is_empty() {
            let path = input.parse::<syn::Path>()?;

            if !path.is_ident("crate") {
                return Err(syn::Error::new_spanned(path, "Unexpected attribute"));
            }

            if let Some(existing) = &crate_path {
                return Err(syn::Error::new_spanned(
                    existing,
                    "Duplicate crate paths specified",
                ));
            }

            if input.parse::<Option<Token![=]>>()?.is_some() {
                crate_path = Some(input.parse()?);
            } else {
                crate_path = Some(path);
            }

            if done {
                break;
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            } else {
                done = true;
            }
        }

        Ok(Self { crate_path })
    }
}

pub(super) struct Types {
    item_impl: syn::ItemImpl,
    what: &'static str,
    types: &'static [(&'static str, Extra)],
    hint: &'static str,
    kind: Kind,
}

impl Parse for Types {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_impl = input.parse::<syn::ItemImpl>()?;

        let what;
        let types;
        let hint;
        let kind;

        'done: {
            match &item_impl.trait_ {
                Some((_, path, _)) => {
                    if let Some(ident) = path.segments.last().map(|s| &s.ident) {
                        if ident == "Decoder" {
                            what = "decoder";
                            types = DECODER_TYPES;
                            hint = "__UseMusliDecoderAttributeMacro";
                            kind = Kind::SelfCx;
                            break 'done;
                        }

                        if ident == "Visitor" {
                            what = "visitor";
                            types = VISITOR_TYPES;
                            hint = "__UseMusliVisitorAttributeMacro";
                            kind = Kind::GenericCx;
                            break 'done;
                        }

                        if ident == "UnsizedVisitor" {
                            what = "unsized_visitor";
                            types = UNSIZED_VISITOR_TYPES;
                            hint = "__UseMusliUnsizedVisitorAttributeMacro";
                            kind = Kind::GenericCx;
                            break 'done;
                        }

                        if ident == "Encoder" {
                            what = "encoder";
                            types = ENCODER_TYPES;
                            hint = "__UseMusliEncoderAttributeMacro";
                            kind = Kind::SelfCx;
                            break 'done;
                        }
                    }

                    return Err(syn::Error::new_spanned(
                        path,
                        "Could not determine the type being implemented",
                    ));
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        item_impl.self_ty,
                        "Could not determine what is being implemented",
                    ));
                }
            }
        };

        Ok(Self {
            item_impl,
            what,
            types,
            hint,
            kind,
        })
    }
}

impl Types {
    /// Expand encoder types.
    pub(crate) fn expand(self, default_crate: &str, attr: &Attr) -> syn::Result<TokenStream> {
        let Self {
            mut item_impl,
            what,
            types,
            hint,
            kind,
        } = self;

        let default_crate_path;

        let crate_path = match &attr.crate_path {
            Some(path) => path,
            None => {
                default_crate_path = ident_path(syn::Ident::new(default_crate, Span::call_site()));
                &default_crate_path
            }
        };

        let mut missing = types
            .iter()
            .map(|(ident, extra)| (syn::Ident::new(ident, Span::call_site()), *extra))
            .collect::<BTreeMap<_, _>>();

        let mut found_cx = None;
        let mut found_mode = None;

        for p in item_impl.generics.type_params() {
            let name = p.ident.to_string();

            if name.starts_with("C") {
                found_cx = Some(found_cx.unwrap_or_else(|| p.ident.clone()));
            } else if name.starts_with("M") {
                found_mode = Some(found_mode.unwrap_or_else(|| p.ident.clone()));
            }
        }

        // List of associated types which are specified, but under a `cfg`
        // attribute so its conditions need to be inverted.
        let mut not_attribute_ty = Vec::new();

        for item in &item_impl.items {
            match item {
                syn::ImplItem::Fn(impl_fn) => {
                    missing.remove(&impl_fn.sig.ident);
                }
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
                path: never_type(crate_path, extra, kind)?,
            });

            item_impl.items.push(syn::ImplItem::Type(impl_type));
        }

        let immediate_cx: syn::Path = match &found_cx {
            Some(p) => syn::parse_quote!(#p),
            None => syn::parse_quote!(__C),
        };

        let path_cx: syn::Path = match kind {
            Kind::SelfCx => {
                syn::parse_quote!(Self::Cx)
            }
            Kind::GenericCx => syn::parse_quote!(#immediate_cx),
        };

        for (ident, extra) in missing {
            let ty;

            match extra {
                Extra::CxFn => {
                    item_impl.items.push(syn::parse_quote! {
                        #[inline]
                        fn cx(&self) -> Self::Cx {
                            self.cx
                        }
                    });

                    continue;
                }
                Extra::TryCloneFn => {
                    item_impl.items.push(syn::parse_quote! {
                        #[inline]
                        fn try_clone(&self) -> Option<Self::TryClone> {
                            None
                        }
                    });

                    continue;
                }
                Extra::Cx => {
                    ty = syn::parse_quote!(#immediate_cx);
                }
                Extra::Mode => {
                    ty = match &found_mode {
                        Some(p) => syn::parse_quote!(#p),
                        None => syn::parse_quote!(__M),
                    };
                }
                Extra::Allocator => {
                    ty = syn::parse_quote!(<#path_cx as #crate_path::Context>::Allocator);
                }
                Extra::Error => {
                    ty = syn::parse_quote!(<#path_cx as #crate_path::Context>::Error);
                }
                _ => {
                    ty = syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: never_type(crate_path, extra, kind)?,
                    });
                }
            };

            let ty = syn::ImplItemType {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                defaultness: None,
                type_token: <Token![type]>::default(),
                ident,
                generics: syn::Generics::default(),
                eq_token: <Token![=]>::default(),
                ty,
                semi_token: <Token![;]>::default(),
            };

            item_impl.items.push(syn::ImplItem::Type(ty));
        }

        item_impl.items.push(syn::ImplItem::Type(syn::ImplItemType {
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

        Ok(item_impl.into_token_stream())
    }
}

fn never_type(crate_path: &syn::Path, extra: Extra, kind: Kind) -> syn::Result<syn::Path> {
    let mut never = crate_path.clone();

    never.segments.push(syn::PathSegment::from(syn::Ident::new(
        "__priv",
        Span::call_site(),
    )));

    never.segments.push({
        let mut s = syn::PathSegment::from(syn::Ident::new("Never", Span::call_site()));

        let mut args = Vec::<syn::GenericArgument>::new();

        match extra {
            Extra::Visitor(ty) => {
                args.push(syn::parse_quote!(Self::Ok));

                match ty {
                    Ty::Str => {
                        args.push(syn::parse_quote!(str));
                    }
                    Ty::Bytes => {
                        args.push(syn::parse_quote!([u8]));
                    }
                }
            }
            Extra::None => match kind {
                Kind::SelfCx => {
                    args.push(syn::parse_quote!(Self::Cx));
                    args.push(syn::parse_quote!(Self::Mode));
                }
                Kind::GenericCx => {}
            },
            _ => {}
        }

        if !args.is_empty() {
            s.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote!(<(#(#args,)*)>));
        }

        s
    });

    Ok(never)
}

fn ident_path(ident: syn::Ident) -> syn::Path {
    let mut path = syn::Path {
        leading_colon: None,
        segments: Punctuated::default(),
    };

    path.segments.push(syn::PathSegment::from(ident));
    path
}
