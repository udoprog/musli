use std::collections::BTreeMap;

use proc_macro2::{Group, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::Token;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    SelfCx,
    GenericCx,
}

pub(super) struct Attr {
    crate_path: Option<syn::Path>,
}

impl Parse for Attr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
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

enum ItemKind {
    Type(syn::ImplItemType),
    Fn(syn::Signature, Group),
    Verbatim(TokenStream),
}

struct Item {
    attrs: Vec<syn::Attribute>,
    kind: ItemKind,
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }

        match &self.kind {
            ItemKind::Type(impl_item) => impl_item.to_tokens(tokens),
            ItemKind::Fn(sig, body) => {
                sig.to_tokens(tokens);
                body.to_tokens(tokens);
            }
            ItemKind::Verbatim(stream) => stream.to_tokens(tokens),
        }
    }
}

pub(super) struct Types {
    impl_token: Token![impl],
    generics: syn::Generics,
    trait_: syn::Path,
    for_token: Token![for],
    self_ty: syn::Type,
    where_clause: Option<syn::WhereClause>,
    brace_token: syn::token::Brace,
    items: Vec<Item>,
}

#[inline]
pub(super) fn parse(input: ParseStream<'_>, attr: Attr, default_crate: &str) -> syn::Result<Types> {
    let crate_path = match attr.crate_path {
        Some(path) => path,
        None => ident_path(syn::Ident::new(default_crate, Span::call_site())),
    };

    let impl_token = input.parse::<Token![impl]>()?;
    let mut generics = input.parse::<syn::Generics>()?;
    let trait_ = input.parse::<syn::Path>()?;
    let for_token = input.parse::<Token![for]>()?;
    let self_ty = input.parse::<syn::Type>()?;
    let mut where_clause = input.parse::<Option<syn::WhereClause>>()?;

    let types;
    let hint;
    let kind;

    'done: {
        if let Some(ident) = trait_.segments.last().map(|s| &s.ident) {
            if ident == "Decoder" {
                types = DECODER_TYPES;
                hint = syn::Ident::new("__UseMusliDecoderAttributeMacro", Span::call_site());
                kind = Kind::SelfCx;
                break 'done;
            }

            if ident == "Visitor" {
                types = VISITOR_TYPES;
                hint = syn::Ident::new("__UseMusliVisitorAttributeMacro", Span::call_site());
                kind = Kind::GenericCx;
                break 'done;
            }

            if ident == "UnsizedVisitor" {
                types = UNSIZED_VISITOR_TYPES;
                hint = syn::Ident::new("__UseMusliUnsizedVisitorAttributeMacro", Span::call_site());
                kind = Kind::GenericCx;
                break 'done;
            }

            if ident == "Encoder" {
                types = ENCODER_TYPES;
                hint = syn::Ident::new("__UseMusliEncoderAttributeMacro", Span::call_site());
                kind = Kind::SelfCx;
                break 'done;
            }
        }

        return Err(syn::Error::new_spanned(
            trait_,
            "Could not determine the type being implemented",
        ));
    };

    let mut missing = types
        .iter()
        .map(|(ident, extra)| (syn::Ident::new(ident, Span::call_site()), *extra))
        .collect::<BTreeMap<_, _>>();

    let mut items = Vec::new();

    let content;
    let brace_token = syn::braced!(content in input);

    while !content.is_empty() {
        let attrs = content.call(syn::Attribute::parse_outer)?;

        if content.peek(Token![type]) {
            let impl_type = content.parse::<syn::ImplItemType>()?;

            let Some(extra) = missing.remove(&impl_type.ident) else {
                items.push(Item {
                    attrs,
                    kind: ItemKind::Type(impl_type),
                });

                continue;
            };

            let mut has_cfg = false;

            for attr in &attrs {
                if !attr.path().is_ident("cfg") {
                    continue;
                }

                if has_cfg {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format_args!(
                            "#[{default_crate}::default_traits]: only one cfg attribute is supported"
                        ),
                    ));
                }

                has_cfg = true;
            }

            // Since the associated type has a `cfg` statement it can be
            // configured out, if it is it needs to be replaced with a default
            // value.
            if has_cfg {
                let mut attrs = attrs.clone();

                for attr in &mut attrs {
                    if !attr.path().is_ident("cfg") {
                        continue;
                    }

                    if let syn::Meta::List(m) = &mut attr.meta {
                        let tokens = syn::Meta::List(syn::MetaList {
                            path: ident_path(syn::Ident::new("not", Span::call_site())),
                            delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                            tokens: m.tokens.clone(),
                        });

                        m.tokens = tokens.into_token_stream();
                    }
                }

                let impl_type = syn::ImplItemType {
                    attrs: impl_type.attrs.clone(),
                    vis: impl_type.vis.clone(),
                    defaultness: impl_type.defaultness,
                    type_token: impl_type.type_token,
                    ident: impl_type.ident.clone(),
                    generics: impl_type.generics.clone(),
                    eq_token: impl_type.eq_token,
                    ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: never_type(&crate_path, extra)?,
                    }),
                    semi_token: impl_type.semi_token,
                };

                items.push(Item {
                    attrs,
                    kind: ItemKind::Type(impl_type),
                });
            }

            items.push(Item {
                attrs,
                kind: ItemKind::Type(impl_type),
            });
            continue;
        }

        if content.peek(Token![fn]) {
            let sig = content.parse::<syn::Signature>()?;
            let body = content.parse::<Group>()?;
            missing.remove(&sig.ident);
            items.push(Item {
                attrs,
                kind: ItemKind::Fn(sig, body),
            });
            continue;
        }

        return Err(syn::Error::new_spanned(
            content.parse::<TokenStream>()?,
            format_args!(
                "#[{default_crate}::default_traits]: only associated types and functions are supported"
            ),
        ));
    }

    let mut found_cx = None;
    let mut found_mode = None;

    for p in generics.type_params() {
        let name = p.ident.to_string();

        if name.starts_with("C") {
            found_cx = Some(found_cx.unwrap_or_else(|| p.ident.clone()));
        } else if name.starts_with("M") {
            found_mode = Some(found_mode.unwrap_or_else(|| p.ident.clone()));
        }
    }

    let maker_kind = missing
        .iter()
        .find_map(|(_, extra)| match extra {
            Extra::Cx => Some(Kind::GenericCx),
            _ => None,
        })
        .unwrap_or(kind);

    let mut maker = Maker {
        generics: &mut generics,
        where_clause: &mut where_clause,
        crate_path: &crate_path,
        kind: maker_kind,
        found_cx,
        stored_cx: None,
        found_mode,
        stored_mode: None,
    };

    for (ident, extra) in missing {
        let stream = match extra {
            Extra::CxFn => {
                quote! {
                    #[inline]
                    fn cx(&self) -> Self::Cx {
                        self.cx
                    }
                }
            }
            Extra::TryCloneFn => {
                quote! {
                    #[inline]
                    fn try_clone(&self) -> Option<Self::TryClone> {
                        None
                    }
                }
            }
            Extra::Cx => {
                let value = maker.make_cx();

                quote! {
                    type #ident = #value;
                }
            }
            Extra::Mode => {
                let value = maker.make_mode();

                quote! {
                    type #ident = #value;
                }
            }
            Extra::Allocator => {
                let value = maker.make_cx();

                quote! {
                    type #ident = <#value as #crate_path::Context>::Allocator;
                }
            }
            Extra::Error => {
                let value = maker.make_cx();

                quote! {
                    type #ident = <#value as #crate_path::Context>::Error;
                }
            }
            _ => {
                let never_type = never_type(&crate_path, extra)?;

                quote! {
                    type #ident = #never_type;
                }
            }
        };

        items.push(Item {
            attrs: Vec::new(),
            kind: ItemKind::Verbatim(stream),
        });
    }

    items.push(Item {
        attrs: Vec::new(),
        kind: ItemKind::Verbatim(quote!(type #hint = ();)),
    });

    Ok(Types {
        impl_token,
        generics,
        trait_,
        for_token,
        self_ty,
        where_clause,
        brace_token,
        items,
    })
}

impl ToTokens for Types {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.impl_token.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.trait_.to_tokens(tokens);
        self.for_token.to_tokens(tokens);
        self.self_ty.to_tokens(tokens);
        self.where_clause.to_tokens(tokens);

        self.brace_token.surround(tokens, |tokens| {
            for item in &self.items {
                item.to_tokens(tokens);
            }
        });
    }
}

fn never_type(crate_path: &syn::Path, extra: Extra) -> syn::Result<syn::Path> {
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
            Extra::None => {
                args.push(syn::parse_quote!(Self::Cx));
                args.push(syn::parse_quote!(Self::Mode));
            }
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

struct Maker<'a> {
    generics: &'a mut syn::Generics,
    where_clause: &'a mut Option<syn::WhereClause>,
    crate_path: &'a syn::Path,
    kind: Kind,
    found_cx: Option<syn::Ident>,
    stored_cx: Option<syn::Path>,
    found_mode: Option<syn::Ident>,
    stored_mode: Option<syn::Ident>,
}

impl Maker<'_> {
    fn make_cx(&mut self) -> syn::Path {
        if let Some(cx) = self.stored_cx.clone() {
            return cx;
        }

        match self.kind {
            Kind::SelfCx => {
                let parsed: syn::Path = syn::parse_quote!(Self::Cx);
                self.stored_cx = Some(parsed.clone());
                parsed
            }
            Kind::GenericCx => {
                if let Some(found) = self.found_cx.take() {
                    let found = syn::Path::from(found.clone());
                    self.stored_cx = Some(found.clone());
                    return found;
                }

                let type_param = syn::Ident::new("__C", Span::call_site());

                self.generics
                    .params
                    .push(syn::GenericParam::Type(syn::TypeParam::from(
                        type_param.clone(),
                    )));

                let crate_path = self.crate_path;

                self.where_clause
                    .get_or_insert_with(|| syn::WhereClause {
                        where_token: Token![where](Span::call_site()),
                        predicates: Punctuated::new(),
                    })
                    .predicates
                    .push(syn::parse_quote!(#type_param: #crate_path::Context));

                let c = syn::Path::from(type_param);
                self.stored_cx = Some(c.clone());
                c
            }
        }
    }

    // Make mode and make sure it's part of generics.
    fn make_mode(&mut self) -> syn::Ident {
        if let Some(mode) = self.stored_mode.clone() {
            return mode;
        }

        if let Some(found) = self.found_mode.take() {
            self.stored_mode = Some(found.clone());
            return found;
        }

        let m = syn::Ident::new("__M", Span::call_site());

        self.generics
            .params
            .push(syn::GenericParam::Type(syn::TypeParam::from(m.clone())));

        self.stored_mode = Some(m.clone());
        m
    }
}
