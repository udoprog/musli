use std::collections::HashMap;
use std::mem;

use proc_macro2::Span;
use quote::ToTokens;
use syn::meta::ParseNestedMeta;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::{NameMethod, NameType};

use super::build;
use super::ATTR;
use super::{Ctxt, ImportedMethod, Mode, NameAll};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ModeKind {
    Binary,
    Text,
    Custom(Box<str>),
}

impl ModeKind {
    pub(crate) fn default_name_all(&self) -> Option<NameAll> {
        match self {
            ModeKind::Binary => Some(NameAll::Index),
            ModeKind::Text => Some(NameAll::Name),
            ModeKind::Custom(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ModeIdent {
    pub(crate) ident: syn::Ident,
    pub(crate) kind: ModeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Only {
    Encode,
    Decode,
}

#[derive(Default)]
struct OneOf<T> {
    encode: T,
    decode: T,
    any: T,
}

pub(crate) enum EnumTagging<'a> {
    /// Use the default tagging method, as provided by the encoder-specific
    /// method.
    Default,
    /// Only the tag is encoded.
    Empty,
    /// The type is internally tagged by the field given by the expression.
    Internal {
        tag_value: &'a syn::Expr,
        tag_type: NameType<'a>,
    },
    /// An enumerator is adjacently tagged.
    Adjacent {
        tag_value: &'a syn::Expr,
        tag_type: NameType<'a>,
        content_value: &'a syn::Expr,
        content_type: NameType<'a>,
    },
}

/// If the type is tagged or not.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Packing {
    #[default]
    Tagged,
    Packed,
    Transparent,
}

macro_rules! first {
    (, $b:expr) => {
        $b
    };
    ($a:expr, $_:expr) => {
        $a
    };
}

macro_rules! layer {
    ($attr:ident, $new:ident, $layer:ident {
        $(
            $(#[doc = $single_doc:literal])*
            $(#[example = $single_example:literal])?
            $single:ident: $single_ty:ty
        ),*
        $(
            , @multiple
            $($(#[$($multiple_meta:meta)*])*
            $multiple:ident: $multiple_ty:ty,)* $(,)?
        )?
        $(,)?
    }) => {
        #[derive(Default)]
        pub(crate) struct $attr {
            root: $layer,
            modes: HashMap<ModeKind, $layer>,
        }

        impl $attr {
            fn by_mode<A, O>(&self, mode: &Mode<'_>, access: A) -> Option<&O>
            where
                A: Copy + Fn(&$layer) -> Option<&O>,
                O: ?Sized,
            {
                if let Some(value) = mode.kind.and_then(|m| self.modes.get(m).and_then(access)) {
                    Some(value)
                } else {
                    access(&self.root)
                }
            }

            $(
                #[allow(unused)]
                pub(crate) fn $single(&self, mode: &Mode<'_>) -> Option<&(Span, $single_ty)> {
                    self.by_mode(mode, |m| {
                        match mode.only {
                            Only::Encode if m.$single.encode.is_some() => m.$single.encode.as_ref(),
                            Only::Decode if m.$single.decode.is_some() => m.$single.decode.as_ref(),
                            _ => m.$single.any.as_ref(),
                        }
                    })
                }
            )*

            $($(
                #[allow(unused)]
                pub(crate) fn $multiple(&self, mode: &Mode<'_>) -> &[(Span, $multiple_ty)] {
                    self.by_mode(mode, |m| {
                        match mode.only {
                            Only::Encode if !m.$multiple.encode.is_empty() => Some(&m.$multiple.encode[..]),
                            Only::Decode if !m.$multiple.decode.is_empty() => Some(&m.$multiple.decode[..]),
                            _ if !m.$multiple.any.is_empty() => Some(&m.$multiple.any[..]),
                            _ => None,
                        }
                    }).unwrap_or_default()
                }
            )*)*
        }

        #[derive(Default)]
        struct $new {
            $(
                $(#[doc = $single_doc])*
                $single: Vec<(Span, $single_ty)>,
            )*
            $($($(#[$($multiple_meta)*])*
            $multiple: Vec<(Span, $multiple_ty)>,)*)*
        }

        #[derive(Default)]
        struct $layer {
            $(
                $(#[doc = $single_doc])*
                $single: OneOf<Option<(Span, $single_ty)>>,
            )*
            $($($(#[$($multiple_meta)*])* $multiple: OneOf<Vec<(Span, $multiple_ty)>>,)*)*
        }

        impl $layer {
            /// Merge attributes.
            fn merge_with(&mut self, cx: &Ctxt, new: $new, only: Option<Only>) {
                $(
                    for $single in new.$single {
                        let out = match only {
                            None => &mut self.$single.any,
                            Some(Only::Decode) => &mut self.$single.decode,
                            Some(Only::Encode) => &mut self.$single.encode,
                        };

                        if out.is_some() {
                            cx.error_span(
                                $single.0,
                                format_args!(
                                    "#[{}] multiple {} attributes specified",
                                    ATTR,
                                    first!($($single_example)?, stringify!($single))
                                ),
                            );
                        } else {
                            *out = Some($single);
                        }
                    }
                )*

                $($(
                    let list = match only {
                        None => {
                            &mut self.$multiple.any
                        }
                        Some(Only::Encode) => {
                            &mut self.$multiple.encode
                        }
                        Some(Only::Decode) => {
                            &mut self.$multiple.decode
                        }
                    };

                    list.extend(new.$multiple);
                )*)*
            }
        }
    }
}

layer! {
    TypeAttr, TypeLayerNew, TypeLayer {
        /// `#[musli(crate = <path>)]`.
        krate: syn::Path,
        /// `#[musli(name_all = "..")]`.
        name_all: NameAll,
        /// `#[musli(name(type = <type>))]`.
        #[example = "name(type = <type>)"]
        name_type: syn::Type,
        /// `#[musli(name(method = "sized" | "unsized" | "unsized_bytes"))]`.
        #[example = "name(method = \"sized\" | \"unsized\" | \"unsized_bytes\")"]
        name_method: NameMethod,
        /// `#[musli(name(format_with = ..))]`.
        #[example = "name(format_with = ..)"]
        name_format_with: syn::Path,
        /// If `#[musli(tag = <expr>)]` is specified.
        #[example = "tag = <expr>"]
        tag_value: syn::Expr,
        /// If `#[musli(tag(type = <type>))]` is specified.
        #[example = "tag(type = <type>)"]
        tag_type: syn::Type,
        /// If `#[musli(tag(method = <type>))]` is specified.
        #[example = "tag(method = \"method\")"]
        tag_method: NameMethod,
        /// If `#[musli(tag(format_with = ..))]` is specified.
        #[example = "tag(format_with = .)"]
        tag_format_with: syn::Path,
        /// If `#[musli(content = <expr>)]` is specified.
        #[example = "content = <expr>"]
        content_value: syn::Expr,
        /// If `#[musli(content(type = <expr>))]` is specified.
        #[example = "content(type = <expr>)"]
        content_type: syn::Type,
        /// If `#[musli(content(method = "sized" | "unsized" | "unsized_bytes"))]` is specified.
        #[example = "content(method = \"sized\" | \"unsized\" | \"unsized_bytes\")"]
        content_method: NameMethod,
        /// If `#[musli(content(format_with = ..))]` is specified.
        #[example = "content(format_with = ..)"]
        content_format_with: syn::Path,
        /// `#[musli(packed)]` or `#[musli(transparent)]`.
        packing: Packing,
        @multiple
        /// Bounds in a where predicate.
        bounds: syn::WherePredicate,
        /// Bounds to require for a `Decode` implementation.
        decode_bounds: syn::WherePredicate,
        /// Lifetimes specified.
        decode_bounds_lifetimes: syn::Lifetime,
        /// Types specified.
        decode_bounds_types: syn::Ident,
    }
}

impl TypeAttr {
    pub(crate) fn is_name_type_ambiguous(&self, mode: &Mode<'_>) -> bool {
        self.name_type(mode).is_none()
            && self.name_all(mode).is_none()
            && self.name_method(mode).is_none()
    }

    pub(crate) fn enum_tagging_span(&self, mode: &Mode<'_>) -> Option<Span> {
        let tag = self.tag_value(mode);
        let content = self.content_value(mode);
        Some(tag.or(content)?.0)
    }

    /// Indicates the state of enum tagging.
    pub(crate) fn enum_tagging(&self, mode: &Mode<'_>) -> Option<EnumTagging<'_>> {
        let (_, tag_value) = self.tag_value(mode)?;

        let default_tag_type = build::determine_type(tag_value);
        let (_, ty, method) = build::split_name(
            mode.kind,
            self.tag_type(mode).or(default_tag_type.as_ref()),
            None,
            self.tag_method(mode),
        );

        let tag_type = NameType {
            ty,
            method,
            format_with: self.tag_format_with(mode),
        };

        Some(match self.content_value(mode) {
            Some((_, content_value)) => {
                let default_content_type = build::determine_type(content_value);
                let (_, ty, method) = build::split_name(
                    mode.kind,
                    self.content_type(mode).or(default_content_type.as_ref()),
                    None,
                    self.content_method(mode),
                );

                EnumTagging::Adjacent {
                    tag_value,
                    tag_type,
                    content_value,
                    content_type: NameType {
                        ty,
                        method,
                        format_with: self.content_format_with(mode),
                    },
                }
            }
            _ => EnumTagging::Internal {
                tag_value,
                tag_type,
            },
        })
    }

    /// Get the configured crate, or fallback to default.
    pub(crate) fn crate_or_default(&self, default: &str) -> syn::Path {
        if let Some((_, krate)) = self.root.krate.any.as_ref() {
            return krate.clone();
        }

        let mut path = syn::Path::from(syn::Ident::new(default, Span::call_site()));
        path.leading_colon = Some(<Token![::]>::default());
        path
    }
}

pub(crate) fn type_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> TypeAttr {
    let mut attr = TypeAttr::default();

    for a in attrs {
        if !a.path().is_ident(ATTR) {
            continue;
        }

        let mut new = TypeLayerNew::default();
        let mut mode = None;
        let mut only = None;

        let result = a.parse_nested_meta(|meta| {
            // #[musli(mode = <path>)]
            if meta.path.is_ident("mode") {
                meta.input.parse::<Token![=]>()?;
                mode = Some(parse_mode(&meta)?);
                return Ok(());
            }

            if meta.path.is_ident("encode_only") {
                only = Some(Only::Encode);
                return Ok(());
            }

            if meta.path.is_ident("decode_only") {
                only = Some(Only::Decode);
                return Ok(());
            }

            // #[musli(tag = <expr>)]
            if meta.path.is_ident("tag") {
                let ty = TypeConfig::parse("name", meta.input, true, true)?;
                new.tag_value.extend(ty.value);
                new.tag_type.extend(ty.ty);
                new.tag_method.extend(ty.method);
                new.tag_format_with.extend(ty.format_with);
                return Ok(());
            }

            // #[musli(content = <expr>)]
            if meta.path.is_ident("content") {
                let ty = TypeConfig::parse("content", meta.input, true, true)?;
                new.content_value.extend(ty.value);
                new.content_type.extend(ty.ty);
                new.content_method.extend(ty.method);
                new.content_format_with.extend(ty.format_with);
                return Ok(());
            }

            // #[musli(crate = <path>)]
            if meta.path.is_ident("crate") {
                let path = if meta.input.parse::<Option<Token![=]>>()?.is_some() {
                    meta.input.parse()?
                } else {
                    syn::parse_quote!(crate)
                };

                new.krate.push((meta.path.span(), path));
                return Ok(());
            }

            // #[musli(name)]
            if meta.path.is_ident("name") {
                let ty = TypeConfig::parse("name", meta.input, false, true)?;
                new.name_type.extend(ty.ty);
                new.name_method.extend(ty.method);
                new.name_format_with.extend(ty.format_with);
                return Ok(());
            }

            // #[musli(bound = {..})]
            if meta.path.is_ident("bound") {
                meta.input.parse::<Token![=]>()?;
                parse_bounds(&meta, &mut new.bounds)?;
                return Ok(());
            }

            // #[musli(decode_bound = {..})]
            if meta.path.is_ident("decode_bound") {
                if meta.input.parse::<Option<Token![<]>>()?.is_some() {
                    parse_bound_types(
                        &meta,
                        &mut new.decode_bounds_lifetimes,
                        &mut new.decode_bounds_types,
                    )?;
                }

                meta.input.parse::<Token![=]>()?;
                parse_bounds(&meta, &mut new.decode_bounds)?;
                return Ok(());
            }

            // #[musli(packed)]
            if meta.path.is_ident("packed") {
                new.packing.push((meta.path.span(), Packing::Packed));
                return Ok(());
            }

            // #[musli(transparent)]
            if meta.path.is_ident("transparent") {
                new.packing.push((meta.path.span(), Packing::Transparent));
                return Ok(());
            }

            // #[musli(name_all = "..")]
            if meta.path.is_ident("name_all") {
                new.name_all
                    .push((meta.path.span(), parse_name_all(&meta)?));
                return Ok(());
            }

            Err(syn::Error::new_spanned(
                meta.path,
                format_args!("#[{ATTR}] Unsupported type attribute"),
            ))
        });

        if let Err(error) = result {
            cx.syn_error(error);
        }

        let attr = match mode {
            Some(mode) => {
                let modes = attr.modes.entry(mode.kind.clone()).or_default();
                cx.register_mode(mode);
                modes
            }
            None => &mut attr.root,
        };

        attr.merge_with(cx, new, only);
    }

    attr
}

fn parse_name_all(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<NameAll, syn::Error> {
    meta.input.parse::<Token![=]>()?;

    let string: syn::LitStr = meta.input.parse()?;
    let s = string.value();

    let Some(name_all) = NameAll::parse(s.as_str()) else {
        let mut options = Vec::new();

        for option in NameAll::ALL {
            options.push(format!(r#""{option}""#));
        }

        let options = options.join(", ");

        return Err(syn::Error::new_spanned(
            string,
            format_args!("#[{ATTR}(name_all = {s:?})]: Bad value, expected one of {options}"),
        ));
    };

    Ok(name_all)
}

fn parse_bound_types(
    meta: &syn::meta::ParseNestedMeta,
    lifetimes: &mut Vec<(Span, syn::Lifetime)>,
    types: &mut Vec<(Span, syn::Ident)>,
) -> syn::Result<()> {
    let mut first = true;
    let mut last = false;

    while !meta.input.is_empty() && !meta.input.peek(Token![>]) {
        if !first {
            last |= meta.input.parse::<Option<Token![,]>>()?.is_none();
        }

        'out: {
            if let Some(lt) = meta.input.parse::<Option<syn::Lifetime>>()? {
                lifetimes.push((lt.span(), lt));
                break 'out;
            }

            let ident = meta.input.parse::<syn::Ident>()?;
            types.push((ident.span(), ident));
        }

        first = false;

        if last {
            break;
        }
    }

    meta.input.parse::<Token![>]>()?;
    Ok(())
}

fn parse_bounds(
    meta: &syn::meta::ParseNestedMeta,
    out: &mut Vec<(Span, syn::WherePredicate)>,
) -> syn::Result<()> {
    let content;
    syn::braced!(content in meta.input);
    let where_clauses = content.parse_terminated(syn::WherePredicate::parse, Token![,])?;

    for where_clause in where_clauses {
        out.push((meta.path.span(), where_clause));
    }

    Ok(())
}

layer! {
    VariantAttr, VariantLayerNew, VariantLayer {
        /// `#[musli(name = ..)]`.
        #[example = "name = .."]
        name_expr: syn::Expr,
        /// `#[musli(name(type = <type>))]`.
        #[example = "name(type = <type>)"]
        name_type: syn::Type,
        /// `#[musli(name(method = "sized" | "unsized" | "unsized_bytes"))]`.
        #[example = "name(method = \"sized\" | \"unsized\" | \"unsized_bytes\")"]
        name_method: NameMethod,
        /// `#[musli(name(format_with = <path>))]`.
        #[example = "name(format_with = <path>)"]
        name_format_with: syn::Path,
        /// Pattern used to match the given field when decoding.
        pattern: syn::Pat,
        /// `#[musli(name_all = "..")]`.
        name_all: NameAll,
        /// `#[musli(packed)]` or `#[musli(transparent)]`.
        packing: Packing,
        /// `#[musli(default)]`.
        default_variant: (),
    }
}

impl VariantAttr {
    pub(crate) fn is_name_type_ambiguous(&self, mode: &Mode<'_>) -> bool {
        self.name_type(mode).is_none()
            && self.name_all(mode).is_none()
            && self.name_method(mode).is_none()
    }
}

/// Parse variant attributes.
pub(crate) fn variant_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> VariantAttr {
    let mut attr = VariantAttr::default();

    for a in attrs {
        if !a.path().is_ident(ATTR) {
            continue;
        }

        let mut new = VariantLayerNew::default();
        let mut mode = None;
        let mut only = None;

        let result = a.parse_nested_meta(|meta| {
            // #[musli(mode = <path>)]
            if meta.path.is_ident("mode") {
                meta.input.parse::<Token![=]>()?;
                mode = Some(parse_mode(&meta)?);
                return Ok(());
            }

            if meta.path.is_ident("encode_only") {
                only = Some(Only::Encode);
                return Ok(());
            }

            if meta.path.is_ident("decode_only") {
                only = Some(Only::Decode);
                return Ok(());
            }

            if meta.path.is_ident("rename") {
                return Err(syn::Error::new_spanned(
                    meta.path,
                    "#[musli(rename = ..)] has been changed to #[musli(name = ..)]",
                ));
            }

            // #[musli(pattern = <expr>)]
            if meta.path.is_ident("pattern") {
                meta.input.parse::<Token![=]>()?;
                new.pattern
                    .push((meta.path.span(), meta.input.call(syn::Pat::parse_single)?));
                return Ok(());
            }

            // #[musli(default)]
            if meta.path.is_ident("default") {
                new.default_variant.push((meta.path.span(), ()));
                return Ok(());
            }

            // #[musli(packed)]
            if meta.path.is_ident("packed") {
                new.packing.push((meta.path.span(), Packing::Packed));
                return Ok(());
            }

            // #[musli(transparent)]
            if meta.path.is_ident("transparent") {
                new.packing.push((meta.path.span(), Packing::Transparent));
                return Ok(());
            }

            // #[musli(name_all = "..")]
            if meta.path.is_ident("name_all") {
                new.name_all
                    .push((meta.path.span(), parse_name_all(&meta)?));
                return Ok(());
            }

            // #[musli(name)]
            if meta.path.is_ident("name") {
                let ty = TypeConfig::parse("name", meta.input, true, true)?;
                new.name_expr.extend(ty.value);
                new.name_type.extend(ty.ty);
                new.name_method.extend(ty.method);
                new.name_format_with.extend(ty.format_with);
                return Ok(());
            }

            Err(syn::Error::new_spanned(
                meta.path,
                format_args!("#[{ATTR}] Unsupported type attribute"),
            ))
        });

        if let Err(error) = result {
            cx.syn_error(error);
        }

        let attr = match mode {
            Some(mode) => {
                let out = attr.modes.entry(mode.kind.clone()).or_default();
                cx.register_mode(mode);
                out
            }
            None => &mut attr.root,
        };

        attr.merge_with(cx, new, only);
    }

    attr
}

#[derive(Default, Clone, Copy)]
pub(crate) enum FieldEncoding {
    Packed,
    Bytes,
    Trace,
    #[default]
    Default,
}

layer! {
    Field, FieldNew, FieldLayer {
        /// Module to use when decoding.
        encode_path: syn::Path,
        /// Path to use when decoding.
        decode_path: syn::Path,
        /// Method to check if we want to skip encoding.
        skip_encoding_if: syn::Path,
        /// Rename a field to the given literal.
        name: syn::Expr,
        /// Pattern used to match the given field when decoding.
        pattern: syn::Pat,
        /// Use a default value for the field if it's not available.
        is_default: Option<syn::Path>,
        /// Use a default value for the field if it's not available.
        skip: (),
        /// Field encoding to use.
        encoding: FieldEncoding,
    }
}

impl Field {
    /// Expand encode of the given field.
    pub(crate) fn encode_path_expanded<'a>(
        &self,
        mode: &Mode<'a>,
        span: Span,
    ) -> (Span, DefaultOrCustom<'a>) {
        if let Some((span, encode_path)) = self.encode_path(mode) {
            (*span, DefaultOrCustom::Custom(encode_path.clone()))
        } else {
            let field_encoding = self.encoding(mode).map(|&(_, e)| e).unwrap_or_default();
            let encode_path = mode.encode_t_encode(field_encoding);
            (span, DefaultOrCustom::Default(encode_path))
        }
    }

    /// Expand decode of the given field.
    pub(crate) fn decode_path_expanded<'a>(
        &self,
        mode: &Mode<'a>,
        span: Span,
        allocator_ident: &syn::Ident,
    ) -> (Span, DefaultOrCustom<'a>) {
        if let Some((span, decode_path)) = self.decode_path(mode) {
            (*span, DefaultOrCustom::Custom(decode_path.clone()))
        } else {
            let field_encoding = self.encoding(mode).map(|&(_, e)| e).unwrap_or_default();
            let decode_path = mode.decode_t_decode(field_encoding, allocator_ident);
            (span, DefaultOrCustom::Default(decode_path))
        }
    }
}

/// Parse field attributes.
pub(crate) fn field_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> Field {
    let mut attr = Field::default();

    for a in attrs {
        if !a.path().is_ident(ATTR) {
            continue;
        }

        let mut new = FieldNew::default();
        let mut mode = None;
        let mut only = None;

        let result = a.parse_nested_meta(|meta| {
            // #[musli(mode = <path>)]
            if meta.path.is_ident("mode") {
                meta.input.parse::<Token![=]>()?;
                mode = Some(parse_mode(&meta)?);
                return Ok(());
            }

            if meta.path.is_ident("encode_only") {
                only = Some(Only::Encode);
                return Ok(());
            }

            if meta.path.is_ident("decode_only") {
                only = Some(Only::Decode);
                return Ok(());
            }

            // parse #[musli(with = <path>)]
            if meta.path.is_ident("with") {
                meta.input.parse::<Token![=]>()?;
                let mut path = meta.input.parse::<syn::Path>()?;

                let (span, arguments) = match path.segments.last_mut() {
                    Some(s) => (
                        s.span(),
                        mem::replace(&mut s.arguments, syn::PathArguments::None),
                    ),
                    None => (path.span(), syn::PathArguments::None),
                };

                let mut encode_path = path.clone();

                encode_path.segments.push({
                    let mut segment = syn::PathSegment::from(syn::Ident::new("encode", span));
                    segment.arguments = arguments.clone();
                    segment
                });

                let mut decode_path = path.clone();

                decode_path.segments.push({
                    let mut segment = syn::PathSegment::from(syn::Ident::new("decode", span));
                    segment.arguments = arguments;
                    segment
                });

                new.encode_path.push((path.span(), encode_path));
                new.decode_path.push((path.span(), decode_path));
                return Ok(());
            }

            // #[musli(skip_encoding_if = <path>)]
            if meta.path.is_ident("skip_encoding_if") {
                meta.input.parse::<Token![=]>()?;
                new.skip_encoding_if
                    .push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            if meta.path.is_ident("rename") {
                return Err(syn::Error::new_spanned(
                    meta.path,
                    "#[musli(rename = ..)] has been changed to #[musli(name = ..)]",
                ));
            }

            // #[musli(name = <expr>)]
            if meta.path.is_ident("name") {
                meta.input.parse::<Token![=]>()?;
                new.name.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // #[musli(pattern = <expr>)]
            if meta.path.is_ident("pattern") {
                meta.input.parse::<Token![=]>()?;
                new.pattern
                    .push((meta.path.span(), meta.input.call(syn::Pat::parse_single)?));
                return Ok(());
            }

            // #[musli(default)]
            if meta.path.is_ident("default") {
                if meta.input.parse::<Option<Token![=]>>()?.is_some() {
                    new.is_default
                        .push((meta.path.span(), Some(meta.input.parse()?)));
                } else {
                    new.is_default.push((meta.path.span(), None));
                }

                return Ok(());
            }

            // #[musli(skip)]
            if meta.path.is_ident("skip") {
                new.skip.push((meta.path.span(), ()));
                return Ok(());
            }

            // #[musli(trace)]
            if meta.path.is_ident("trace") {
                new.encoding.push((meta.path.span(), FieldEncoding::Trace));
                return Ok(());
            }

            // #[musli(bytes)]
            if meta.path.is_ident("bytes") {
                new.encoding.push((meta.path.span(), FieldEncoding::Bytes));
                return Ok(());
            }

            // #[musli(packed)]
            if meta.path.is_ident("packed") {
                new.encoding.push((meta.path.span(), FieldEncoding::Packed));
                return Ok(());
            }

            Err(syn::Error::new_spanned(
                meta.path,
                format_args!("#[{ATTR}] Unsupported field attribute"),
            ))
        });

        if let Err(error) = result {
            cx.syn_error(error);
        }

        let attr = match mode {
            Some(mode) => {
                let out = attr.modes.entry(mode.kind.clone()).or_default();
                cx.register_mode(mode);
                out
            }
            None => &mut attr.root,
        };

        attr.merge_with(cx, new, only);
    }

    attr
}

fn parse_mode(meta: &ParseNestedMeta<'_>) -> syn::Result<ModeIdent> {
    let ident: syn::Ident = meta.input.parse()?;
    let s = ident.to_string();

    let kind = match s.as_str() {
        "Binary" => ModeKind::Binary,
        "Text" => ModeKind::Text,
        other => ModeKind::Custom(other.into()),
    };

    Ok(ModeIdent { ident, kind })
}

#[derive(Clone, Default)]
struct TypeConfig {
    /// Either the inherent expression parameter, or the associated `expr = ..` value.
    value: Option<(Span, syn::Expr)>,
    /// The `type = ..` parameter.
    ty: Option<(Span, syn::Type)>,
    /// The `method = ".."` parameter`.
    method: Option<(Span, NameMethod)>,
    /// The `format_with = ..` parameter`.
    format_with: Option<(Span, syn::Path)>,
}

impl TypeConfig {
    fn parse(
        name: &str,
        input: ParseStream,
        as_expr: bool,
        format_with: bool,
    ) -> syn::Result<Self> {
        let mut this = Self::default();

        if as_expr && input.parse::<Option<Token![=]>>()?.is_some() {
            if let Some((s, _)) = this.value.replace((input.span(), input.parse()?)) {
                return Err(syn::Error::new(
                    s,
                    format_args!("#[musli({name} = ..)]: Duplicate value for attribute"),
                ));
            }

            return Ok(this);
        }

        let content;
        _ = syn::parenthesized!(content in input);

        while !content.is_empty() {
            'ok: {
                if let Some(ty) = content.parse::<Option<Token![type]>>()? {
                    content.parse::<Token![=]>()?;

                    if let Some((s, _)) = this.ty.replace((ty.span(), content.parse()?)) {
                        return Err(syn::Error::new(
                            s,
                            format_args!("#[musli({name}(type = ..))]: Duplicate attribute"),
                        ));
                    }

                    break 'ok;
                }

                let id = content.parse::<syn::Ident>()?;

                if as_expr && id == "value" {
                    content.parse::<Token![=]>()?;

                    if let Some((s, _)) = this.value.replace((id.span(), content.parse()?)) {
                        return Err(syn::Error::new(
                            s,
                            format_args!(
                                "#[musli({name}(value = ..))]: Duplicate value for attribute"
                            ),
                        ));
                    }

                    break 'ok;
                }

                if id == "method" {
                    content.parse::<Token![=]>()?;

                    if let Some((s, _)) = this.method.replace((id.span(), content.parse()?)) {
                        return Err(syn::Error::new(
                            s,
                            format_args!("#[musli({name}(method = ..))]: Duplicate attribute"),
                        ));
                    }

                    break 'ok;
                }

                if format_with && id == "format_with" {
                    content.parse::<Token![=]>()?;

                    if let Some((s, _)) = this.format_with.replace((id.span(), content.parse()?)) {
                        return Err(syn::Error::new(
                            s,
                            format_args!("#[musli({name}(format_with = ..))]: Duplicate attribute"),
                        ));
                    }

                    break 'ok;
                }

                return Err(syn::Error::new_spanned(
                    &id,
                    format_args!("#[musli({name}({id} = ..))]: Unsupported attribute"),
                ));
            };

            if content.parse::<Option<Token![,]>>()?.is_none() {
                break;
            }
        }

        Ok(this)
    }
}

/// A default or custom path to use.
pub(crate) enum DefaultOrCustom<'a> {
    /// A default method call from [`Tokens`][super::Tokens].
    Default(ImportedMethod<'a>),
    /// A custom specified path.
    Custom(syn::Path),
}

impl DefaultOrCustom<'_> {
    #[inline]
    pub(crate) fn is_default(&self) -> bool {
        matches!(self, DefaultOrCustom::Default(..))
    }
}

impl ToTokens for DefaultOrCustom<'_> {
    #[inline]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DefaultOrCustom::Default(method) => method.to_tokens(tokens),
            DefaultOrCustom::Custom(path) => path.to_tokens(tokens),
        }
    }
}
