use std::collections::HashMap;
use std::fmt;
use std::mem;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::determine_tag_method;
use crate::expander::TagMethod;
use crate::internals::symbol::*;
use crate::internals::{Ctxt, Mode, ModePath};

/// The value and method to encode / decode a tag.
#[derive(Clone, Copy)]
pub(crate) struct EnumTag<'a> {
    pub(crate) value: &'a syn::Expr,
    pub(crate) method: Option<TagMethod>,
}

#[derive(Clone, Copy)]
pub(crate) enum EnumTagging<'a> {
    /// The type is internally tagged by the field given by the expression.
    Internal { tag: EnumTag<'a> },
    Adjacent {
        tag: EnumTag<'a>,
        content: &'a syn::Expr,
    },
}

/// The kind of tag to use.
#[derive(Debug, Clone, Copy)]
pub enum DefaultTag {
    Index,
    Name,
}

/// If the type is tagged or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packing {
    Tagged,
    Packed,
    Transparent,
}

impl fmt::Display for Packing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Packing::Tagged => write!(f, "tagged"),
            Packing::Packed => write!(f, "packed"),
            Packing::Transparent => write!(f, "transparent"),
        }
    }
}

impl Default for Packing {
    fn default() -> Self {
        Self::Tagged
    }
}

#[derive(Default)]
struct InnerTypeAttr {
    /// `#[musli(crate = <path>)]`.
    krate: Option<(Span, syn::ExprPath)>,
    /// `#[musli(name_type)]`.
    tag_type: Option<(Span, syn::Type)>,
    /// `#[musli(default_variant_name = "..")]`.
    default_variant_name: Option<DefaultTag>,
    /// `#[musli(default_field_name = "..")]`.
    default_field_name: Option<DefaultTag>,
    /// If `#[musli(tag = <expr>)]` is specified.
    tag: Option<(Span, syn::Expr)>,
    /// If `#[musli(content = <expr>)]` is specified.
    content: Option<(Span, syn::Expr)>,
    /// `#[musli(packed)]` or `#[musli(transparent)]`.
    packing: Option<(Span, Packing)>,
    /// Bounds in a where predicate.
    bounds: Vec<syn::WherePredicate>,
    /// Bounds to require for a `Decode` implementation.
    decode_bounds: Vec<syn::WherePredicate>,
}

impl InnerTypeAttr {
    /// Update how an enum is tagged.
    fn set_tag(&mut self, cx: &Ctxt, span: Span, expr: syn::Expr) {
        if self.tag.is_some() {
            cx.error_span(
                span,
                format_args!("#[{ATTR}({TAG})] may only be specified once",),
            );

            return;
        }

        self.tag = Some((span, expr));
    }

    /// Update how an enum is tagged.
    fn set_content(&mut self, cx: &Ctxt, span: Span, expr: syn::Expr) {
        if self.content.is_some() {
            cx.error_span(
                span,
                format_args!("#[{ATTR}({TAG})] may only be specified once",),
            );

            return;
        }

        self.content = Some((span, expr));
    }

    /// Update packing of type.
    fn set_packing(&mut self, cx: &Ctxt, span: Span, packing: Packing) -> bool {
        if let Some((_, existing)) = &self.packing {
            if *existing != packing {
                cx.error_span(
                    span,
                    format_args!(
                        "#[{}({})] cannot be combined with #[{}({})]",
                        ATTR, packing, ATTR, existing
                    ),
                );

                return false;
            }
        }

        self.packing = Some((span, packing));
        true
    }

    fn set_crate(&mut self, cx: &Ctxt, span: Span, path: syn::ExprPath) {
        if let Some((span, _)) = self.krate {
            cx.error_span(
                span,
                format_args!("#[{}({})] cannot be used multiple times", ATTR, CRATE),
            );
        }

        self.krate = Some((span, path));
    }
}

#[derive(Default)]
pub(crate) struct TypeAttr {
    root: InnerTypeAttr,
    /// Nested configuartions for modes.
    modes: HashMap<syn::ExprPath, InnerTypeAttr>,
}

impl TypeAttr {
    /// Indicates the packing state of the type.
    pub(crate) fn packing_span(&self, mode: Mode<'_>) -> Option<(Span, Packing)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.packing)
            .or(self.root.packing)
    }

    fn tag(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.tag.as_ref())
            .or(self.root.tag.as_ref())
    }

    fn content(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.content.as_ref())
            .or(self.root.content.as_ref())
    }

    pub(crate) fn enum_tagging_span(&self, mode: Mode<'_>) -> Option<Span> {
        let tag = self.tag(mode);
        let content = self.content(mode);
        Some(tag.or(content)?.0)
    }

    /// Indicates the state of enum tagging.
    pub(crate) fn enum_tagging(&self, mode: Mode<'_>) -> Option<EnumTagging<'_>> {
        let tag = self.tag(mode);
        let (_, tag) = tag?;

        let tag_method = determine_tag_method(tag);
        let tag = EnumTag {
            value: tag,
            method: tag_method,
        };

        match self.content(mode) {
            Some((_, content)) => Some(EnumTagging::Adjacent { tag, content }),
            _ => Some(EnumTagging::Internal { tag }),
        }
    }

    /// Indicates the packing state of the type.
    pub(crate) fn packing(&self, mode: Mode<'_>) -> Option<Packing> {
        Some(self.packing_span(mode)?.1)
    }

    /// Default field tag.
    pub(crate) fn default_field_name(&self, mode: Mode<'_>) -> Option<DefaultTag> {
        mode.ident
            .and_then(|m| Some(self.modes.get(m)?.default_field_name))
            .unwrap_or(self.root.default_field_name)
    }

    pub(crate) fn default_variant_name(&self, mode: Mode<'_>) -> Option<DefaultTag> {
        mode.ident
            .and_then(|m| Some(self.modes.get(m)?.default_variant_name))
            .unwrap_or(self.root.default_variant_name)
    }

    /// Get the tag type of the type.
    pub(crate) fn tag_type(&self, mode: Mode<'_>) -> Option<&(Span, syn::Type)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.tag_type.as_ref())
            .or(self.root.tag_type.as_ref())
    }

    /// Get the configured crate, or fallback to default.
    pub(crate) fn crate_or_default(&self) -> syn::ExprPath {
        if let Some((_, krate)) = &self.root.krate {
            krate.clone()
        } else {
            let path = syn::Path::from(syn::Ident::new(&ATTR, Span::call_site()));

            syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            }
        }
    }

    /// Get the where clause that is associated with the type.
    pub(crate) fn bounds(&self, mode: Mode<'_>) -> &[syn::WherePredicate] {
        mode.ident
            .and_then(|m| Some(&self.modes.get(m)?.bounds))
            .unwrap_or(&self.root.bounds)
    }

    /// Get bounds to require for a `Decode` implementation.
    pub(crate) fn decode_bounds(&self, mode: Mode<'_>) -> &[syn::WherePredicate] {
        mode.ident
            .and_then(|m| Some(&self.modes.get(m)?.decode_bounds))
            .unwrap_or(&self.root.decode_bounds)
    }
}

#[derive(Default)]
struct InnerFieldAttr {
    /// Module to use when decoding.
    encode_path: Option<(Span, syn::ExprPath)>,
    /// Path to use when decoding.
    decode_path: Option<(Span, syn::ExprPath)>,
    /// Method to check if we want to skip encoding.
    skip_encoding_if: Option<(Span, syn::ExprPath)>,
    /// Rename a field to the given literal.
    rename: Option<(Span, syn::Expr)>,
    /// Use a default value for the field if it's not available.
    default: Option<Span>,
}

impl InnerFieldAttr {
    fn set_encode_path(&mut self, cx: &Ctxt, span: Span, encode_path: syn::ExprPath) {
        if self.encode_path.is_some() {
            cx.error_spanned_by(
                encode_path,
                format_args!("#[{}] multiple encode methods specified", ATTR),
            );
        } else {
            self.encode_path = Some((span, encode_path));
        }
    }

    fn set_decode_path(&mut self, cx: &Ctxt, span: Span, decode_path: syn::ExprPath) {
        if self.decode_path.is_some() {
            cx.error_spanned_by(
                decode_path,
                format_args!("#[{}] multiple decode methods specified", ATTR),
            );
        } else {
            self.decode_path = Some((span, decode_path));
        }
    }

    fn set_skip_encoding_if(&mut self, cx: &Ctxt, span: Span, skip_encoding_if: syn::ExprPath) {
        if self.skip_encoding_if.is_some() {
            cx.error_spanned_by(
                skip_encoding_if,
                format_args!("#[{}] multiple skip_encoding_if methods specified", ATTR),
            );
        } else {
            self.skip_encoding_if = Some((span, skip_encoding_if));
        }
    }
}

#[derive(Default)]
struct InternalVariantAttr {
    /// `#[musli(name_type)]`.
    tag_type: Option<(Span, syn::Type)>,
    /// Rename a field to the given expression.
    rename: Option<(Span, syn::Expr)>,
    /// `#[musli(packed)]` or `#[musli(transparent)]`.
    packing: Option<(Span, Packing)>,
    /// `#[musli(default)]`.
    default: Option<Span>,
    /// `#[musli(default_field_name = "..")]`.
    default_field_name: Option<DefaultTag>,
}

impl InternalVariantAttr {
    /// Update packing of type.
    fn set_packing(&mut self, cx: &Ctxt, span: Span, packing: Packing) {
        if let Some((_, existing)) = &self.packing {
            cx.error_span(
                span,
                format_args!(
                    "#[{}({})] cannot be combined with #[{}({})]",
                    ATTR, packing, ATTR, existing
                ),
            );
        }

        self.packing = Some((span, packing));
    }
}

#[derive(Default)]
pub(crate) struct VariantAttr {
    root: InternalVariantAttr,
    modes: HashMap<syn::ExprPath, InternalVariantAttr>,
}

impl VariantAttr {
    /// Test if the `#[musli(default)]` tag is specified.
    pub(crate) fn default_attr(&self, mode: Mode<'_>) -> Option<Span> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.default)
            .or(self.root.default)
    }

    /// Test if the `#[musli(rename)]` tag is specified.
    pub(crate) fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.rename.as_ref())
            .or(self.root.rename.as_ref())
    }

    /// Indicates if the tagged state of the variant is set.
    pub(crate) fn packing(&self, mode: Mode<'_>) -> Option<Packing> {
        let packing = mode
            .ident
            .and_then(|m| self.modes.get(m)?.packing.as_ref())
            .or(self.root.packing.as_ref())?;

        Some(packing.1)
    }

    /// Default field tag.
    pub(crate) fn default_field_name(&self, mode: Mode<'_>) -> Option<DefaultTag> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.default_field_name)
            .or(self.root.default_field_name)
    }

    /// Get the tag type of the type.
    pub(crate) fn tag_type(&self, mode: Mode<'_>) -> Option<&(Span, syn::Type)> {
        mode.ident
            .and_then(|m| self.modes.get(m))
            .map(|a| a.tag_type.as_ref())
            .unwrap_or(self.root.tag_type.as_ref())
    }
}

#[derive(Default)]
pub(crate) struct FieldAttr {
    root: InnerFieldAttr,
    modes: HashMap<syn::ExprPath, InnerFieldAttr>,
}

impl FieldAttr {
    /// Test if the `#[musli(default)]` tag is specified.
    pub(crate) fn default_attr(&self, mode: Mode<'_>) -> Option<Span> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.default)
            .or(self.root.default)
    }

    /// Test if the `#[musli(rename)]` tag is specified.
    pub(crate) fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.rename.as_ref())
            .or(self.root.rename.as_ref())
    }

    /// Expand encode of the given field.
    pub(crate) fn encode_path(&self, mode: Mode<'_>, span: Span) -> (Span, TokenStream) {
        let encode_path = mode
            .ident
            .and_then(|m| self.modes.get(m)?.encode_path.as_ref())
            .or(self.root.encode_path.as_ref());

        if let Some((span, encode_path)) = encode_path {
            let mut encode_path = encode_path.clone();
            let mode_ident = mode.mode_ident();

            if let Some(last) = encode_path.path.segments.last_mut() {
                adjust_mode_path(last, mode_ident);
            }

            (*span, encode_path.to_token_stream())
        } else {
            let encode_path = mode.encode_t_encode(span);
            (span, encode_path)
        }
    }

    /// Expand decode of the given field.
    pub(crate) fn decode_path(&self, mode: Mode<'_>, span: Span) -> (Span, TokenStream) {
        let decode_path = mode
            .ident
            .and_then(|m| self.modes.get(m)?.decode_path.as_ref())
            .or(self.root.decode_path.as_ref());

        if let Some((span, decode_path)) = decode_path {
            let mut decode_path = decode_path.clone();
            let mode_ident = mode.mode_ident();

            if let Some(last) = decode_path.path.segments.last_mut() {
                adjust_mode_path(last, mode_ident);
            }

            (*span, decode_path.to_token_stream())
        } else {
            let decode_path = mode.decode_t_decode(span);
            (span, decode_path)
        }
    }

    /// Get skip encoding if.
    pub(crate) fn skip_encoding_if(&self, mode: Mode<'_>) -> Option<(Span, &syn::ExprPath)> {
        let (span, path) = mode
            .ident
            .and_then(|m| self.modes.get(m)?.skip_encoding_if.as_ref())
            .or(self.root.skip_encoding_if.as_ref())?;

        Some((*span, path))
    }
}

/// Adjust a mode path.
fn adjust_mode_path(last: &mut syn::PathSegment, mode_ident: ModePath) {
    let insert_args = |args: &mut Punctuated<_, _>| {
        args.insert(
            0,
            syn::GenericArgument::Type(syn::Type::Verbatim(mode_ident.to_token_stream())),
        );

        args.insert(
            1,
            syn::GenericArgument::Type(syn::Type::Infer(syn::TypeInfer {
                underscore_token: <Token![_]>::default(),
            })),
        );
    };

    match &mut last.arguments {
        syn::PathArguments::None => {
            let mut args = syn::AngleBracketedGenericArguments {
                colon2_token: Some(<Token![::]>::default()),
                lt_token: <Token![<]>::default(),
                args: Punctuated::default(),
                gt_token: <Token![>]>::default(),
            };

            insert_args(&mut args.args);
            last.arguments = syn::PathArguments::AngleBracketed(args);
        }
        syn::PathArguments::AngleBracketed(args) => {
            insert_args(&mut args.args);
        }
        syn::PathArguments::Parenthesized(args) => {
            args.inputs
                .insert(0, syn::Type::Verbatim(mode_ident.to_token_stream()));

            args.inputs.insert(
                1,
                syn::Type::Infer(syn::TypeInfer {
                    underscore_token: <Token![_]>::default(),
                }),
            );
        }
    }
}

pub(crate) fn type_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> TypeAttr {
    let mut attr = TypeAttr::default();

    for a in attrs {
        if let Some(attributes) = parse_musli_attrs::<TypeAttributes>(cx, a) {
            let mut attr = match attributes.mode {
                Some(mode) => {
                    cx.register_mode(mode.clone());
                    attr.modes.entry(mode).or_default()
                }
                None => &mut attr.root,
            };

            for attribute in attributes.attributes {
                match attribute {
                    // parse #[musli(tag = <expr>)]
                    Attribute::KeyValue(path, value) if path == TAG => {
                        if let Some(expr) = value_as_expr(cx, TAG, value) {
                            attr.set_tag(cx, path.span(), expr);
                        }
                    }
                    // parse #[musli(content = <expr>)]
                    Attribute::KeyValue(path, value) if path == CONTENT => {
                        if let Some(expr) = value_as_expr(cx, CONTENT, value) {
                            attr.set_content(cx, path.span(), expr);
                        }
                    }
                    // parse #[musli(crate = <path>)]
                    Attribute::KeyValue(path, value) if path == CRATE => {
                        if let Some(path) = value_as_path(cx, CRATE, value) {
                            attr.set_crate(cx, path.span(), path);
                        }
                    }
                    // parse #[musli(name_type = <type>)]
                    Attribute::KeyValue(path, value) if path == NAME_TYPE => {
                        if let Some(ty) = value_as_type(cx, NAME_TYPE, value) {
                            attr.tag_type = Some((path.span(), ty));
                        }
                    }
                    // parse #[musli(default_variant_name = "..")]
                    Attribute::KeyValue(path, expr) if path == DEFAULT_VARIANT_NAME => {
                        if let Some(tag) = parse_value_string(cx, DEFAULT_VARIANT_NAME, expr) {
                            attr.default_variant_name = match tag.value().as_str() {
                                "index" => Some(DefaultTag::Index),
                                "name" => Some(DefaultTag::Name),
                                _ => {
                                    cx.error_spanned_by(
                                        tag,
                                        format_args!(
                                            "illegal #[{ATTR}({DEFAULT_VARIANT_NAME})] value"
                                        ),
                                    );
                                    continue;
                                }
                            };
                        }
                    }
                    // parse #[musli(default_field_name = "..")]
                    Attribute::KeyValue(path, expr) if path == DEFAULT_FIELD_NAME => {
                        if let Some(tag) = parse_value_string(cx, DEFAULT_FIELD_NAME, expr) {
                            attr.default_field_name = match tag.value().as_str() {
                                "index" => Some(DefaultTag::Index),
                                "name" => Some(DefaultTag::Name),
                                _ => {
                                    cx.error_spanned_by(
                                        tag,
                                        format_args!(
                                            "illegal #[{ATTR}({DEFAULT_FIELD_NAME})] value"
                                        ),
                                    );
                                    continue;
                                }
                            };
                        }
                    }
                    // parse #[musli(bound = ..)]
                    Attribute::KeyValue(path, expr) if path == BOUND => {
                        if let Some(bound) = parse_bound(cx, BOUND, expr) {
                            attr.bounds.push(bound);
                        }
                    }
                    // parse #[musli(decode_bound = ..)]
                    Attribute::KeyValue(path, expr) if path == DECODE_BOUND => {
                        if let Some(bound) = parse_bound(cx, DECODE_BOUND, expr) {
                            attr.decode_bounds.push(bound);
                        }
                    }
                    // parse #[musli(packed)]
                    Attribute::Path(path) if path == PACKED => {
                        attr.set_packing(cx, path.span(), Packing::Packed);
                    }
                    // parse #[musli(flatten)]
                    Attribute::Path(path) if path == TRANSPARENT => {
                        attr.set_packing(cx, path.span(), Packing::Transparent);
                    }
                    meta => {
                        let path = format_path(meta.path());
                        cx.error_span(
                            meta.span(),
                            format_args!("#[{ATTR}({path})] unsupported type attribute"),
                        );
                    }
                }
            }
        }
    }

    attr
}

/// Parse field attributes.
pub(crate) fn field_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> FieldAttr {
    let mut attr = FieldAttr::default();

    for a in attrs {
        if let Some(attributes) = parse_musli_attrs::<FieldAttributes>(cx, a) {
            let mut attr = match attributes.mode {
                Some(mode) => {
                    cx.register_mode(mode.clone());
                    attr.modes.entry(mode).or_default()
                }
                None => &mut attr.root,
            };

            for attribute in attributes.attributes {
                match attribute {
                    // parse #[musli(with = <path>)]
                    Attribute::KeyValue(path, value) if path == WITH => {
                        if let Some(mut path) = value_as_path(cx, WITH, value) {
                            let arguments = match path.path.segments.last_mut() {
                                Some(path) => {
                                    mem::replace(&mut path.arguments, syn::PathArguments::None)
                                }
                                None => syn::PathArguments::None,
                            };

                            let mut encode_path = path.clone();

                            encode_path.path.segments.push({
                                let mut segment: syn::PathSegment =
                                    syn::Ident::new("encode", Span::call_site()).into();
                                segment.arguments = arguments.clone();
                                segment
                            });

                            let mut decode_path = path.clone();

                            decode_path.path.segments.push({
                                let mut segment: syn::PathSegment =
                                    syn::Ident::new("decode", Span::call_site()).into();
                                segment.arguments = arguments.clone();
                                segment
                            });

                            attr.set_encode_path(cx, path.span(), encode_path);
                            attr.set_decode_path(cx, path.span(), decode_path);
                        }
                    }
                    // parse #[musli(skip_encoding_if)]
                    Attribute::KeyValue(path, expr) if path == SKIP_ENCODING_IF => {
                        if let Some(path) = value_as_path(cx, SKIP_ENCODING_IF, expr) {
                            attr.set_skip_encoding_if(cx, path.span(), path.clone());
                        }
                    }
                    // parse #[musli(rename = <expr>)]
                    Attribute::KeyValue(path, value) if path == RENAME => {
                        let span = path.span();

                        if let Some(expr) = value_as_expr(cx, RENAME, value) {
                            if let Some((span, _)) = &attr.rename {
                                cx.error_span(*span, "conflicting attribute");
                            } else {
                                attr.rename = Some((span, expr));
                            }
                        }
                    }
                    // parse #[musli(default)]
                    Attribute::Path(path) if path == DEFAULT => {
                        if let Some(span) = attr.default {
                            cx.error_span(
                                span,
                                format_args!("#[{ATTR}({DEFAULT})] was previously defined here"),
                            );
                        } else {
                            attr.default = Some(path.span());
                        }
                    }
                    meta => {
                        let path = format_path(meta.path());
                        cx.error_span(
                            meta.span(),
                            format_args!("#[{ATTR}({path})] unsupported field attribute"),
                        );
                    }
                }
            }
        }
    }

    attr
}

/// Parse variant attributes.
pub(crate) fn variant_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> VariantAttr {
    let mut attr = VariantAttr::default();

    for a in attrs {
        if let Some(attributes) = parse_musli_attrs::<VariantAttributes>(cx, a) {
            let mut attr = match attributes.mode {
                Some(mode) => {
                    cx.register_mode(mode.clone());
                    attr.modes.entry(mode).or_default()
                }
                None => &mut attr.root,
            };

            for attribute in attributes.attributes {
                match attribute {
                    // parse #[musli(name_type = <type>)]
                    Attribute::KeyValue(path, value) if path == NAME_TYPE => {
                        if let Some(ty) = value_as_type(cx, NAME_TYPE, value) {
                            attr.tag_type = Some((path.span(), ty));
                        }
                    }
                    // parse #[musli(rename = <expr>)]
                    Attribute::KeyValue(path, value) if path == RENAME => {
                        let span = path.span();

                        if let Some(expr) = value_as_expr(cx, RENAME, value) {
                            if let Some((span, _)) = &attr.rename {
                                cx.error_span(*span, "conflicting attribute");
                            } else {
                                attr.rename = Some((span, expr));
                            }
                        }
                    }
                    // parse #[musli(default_field_name = "..")]
                    Attribute::KeyValue(path, expr) if path == DEFAULT_FIELD_NAME => {
                        if let Some(tag) = parse_value_string(cx, DEFAULT_FIELD_NAME, expr) {
                            attr.default_field_name = Some(match tag.value().as_str() {
                                "index" => DefaultTag::Index,
                                "name" => DefaultTag::Name,
                                _ => {
                                    cx.error_spanned_by(
                                        tag,
                                        format_args!(
                                            "illegal #[{ATTR}({DEFAULT_FIELD_NAME})] value"
                                        ),
                                    );
                                    continue;
                                }
                            });
                        }
                    }
                    // parse #[musli(transparent)]
                    Attribute::Path(path) if path == TRANSPARENT => {
                        attr.set_packing(cx, path.span(), Packing::Transparent);
                    }
                    // parse #[musli(packed)]
                    Attribute::Path(path) if path == PACKED => {
                        attr.set_packing(cx, path.span(), Packing::Packed);
                    }
                    // parse #[musli(default)]
                    Attribute::Path(path) if path == DEFAULT => {
                        attr.default = Some(path.span());
                    }
                    meta => {
                        let path = format_path(meta.path());
                        cx.error_span(
                            meta.span(),
                            format_args!("#[{ATTR}({path})] unsupported variant attribute"),
                        );
                    }
                }
            }
        }
    }

    attr
}

/// Get expression path.
fn value_as_path(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::ExprPath> {
    match value {
        AttributeValue::Path(path) => Some(path),
        _ => {
            cx.error_span(
                value.span(),
                format_args!("#[{ATTR}({attr} = ..)] should be a simple path"),
            );
            None
        }
    }
}

/// Get expression from value.
fn value_as_expr(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::Expr> {
    match value {
        AttributeValue::Expr(expr) => Some(expr),
        _ => {
            cx.error_span(
                value.span(),
                format_args!("#[{ATTR}({attr} = ..)] should be a simple path"),
            );
            None
        }
    }
}

/// Get expression as string.
fn parse_value_string(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::LitStr> {
    match value {
        AttributeValue::Lit(syn::Lit::Str(string)) => Some(string),
        expr => {
            cx.error_span(
                expr.span(),
                format_args!("#[{} = ...] should be a string", attr),
            );
            None
        }
    }
}

/// Get aan attribute value as a bound.
fn parse_bound(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::WherePredicate> {
    match value {
        AttributeValue::Bound(clause) => Some(clause),
        expr => {
            cx.error_span(
                expr.span(),
                format_args!(
                    "#[{} = ...] should be a where predicate like `T: Encode`",
                    attr
                ),
            );
            None
        }
    }
}

/// Get expression as a type.
fn value_as_type(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::Type> {
    match value {
        AttributeValue::Type(ty) => Some(ty),
        expr => {
            cx.error_span(
                expr.span(),
                format_args!("#[{} = ...] should be a type", attr),
            );
            None
        }
    }
}

/// Parse musli attributes.
fn parse_musli_attrs<T>(cx: &Ctxt, attr: &syn::Attribute) -> Option<T>
where
    T: Parse,
{
    if attr.path() != ATTR {
        return None;
    }

    let attributes: T = match syn::parse2(attr.meta.to_token_stream()) {
        Ok(attributes) => attributes,
        Err(e) => {
            cx.syn_error(e);
            return None;
        }
    };

    Some(attributes)
}

/// The flexible value of an attribute.
pub enum AttributeValue {
    /// A path.
    Path(syn::ExprPath),
    /// A type.
    Type(syn::Type),
    /// A literal value.
    Expr(syn::Expr),
    /// A literal value.
    Lit(syn::Lit),
    /// A collection of bounds.
    Bound(syn::WherePredicate),
}

impl AttributeValue {
    fn span(&self) -> Span {
        match self {
            AttributeValue::Path(value) => value.span(),
            AttributeValue::Type(value) => value.span(),
            AttributeValue::Expr(value) => value.span(),
            AttributeValue::Lit(value) => value.span(),
            AttributeValue::Bound(value) => value.span(),
        }
    }
}

enum Attribute {
    Path(syn::Path),
    KeyValue(syn::Path, AttributeValue),
}

impl Attribute {
    /// Extract the path of the attribute.
    fn path(&self) -> &syn::Path {
        match self {
            Attribute::Path(path) => path,
            Attribute::KeyValue(path, _) => path,
        }
    }
}

impl Attribute {
    fn span(&self) -> Span {
        match self {
            Attribute::Path(path) => path.span(),
            Attribute::KeyValue(path, _) => path.span(),
        }
    }
}

struct TypeAttributes {
    _parens: syn::token::Paren,
    mode: Option<syn::ExprPath>,
    attributes: Vec<Attribute>,
}

impl Parse for TypeAttributes {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut mode = None;
        let mut attributes = Vec::new();

        // Skip over `musli` attribute.
        let _ = input.parse::<syn::Path>();

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(mode = <ident>)]
                    path if path == MODE => {
                        mode = Some(content.parse()?);

                        if content.parse::<Option<Token![,]>>()?.is_none() {
                            break;
                        }

                        continue;
                    }
                    // parse #[musli(tag = <expr>)]
                    path if path == TAG => AttributeValue::Expr(content.parse()?),
                    // parse #[musli(content = <expr>)]
                    path if path == CONTENT => AttributeValue::Expr(content.parse()?),
                    // parse #[musli(crate = <path>)]
                    path if path == CRATE => AttributeValue::Path(content.parse()?),
                    // parse #[musli(name_type = <type>)]
                    path if path == NAME_TYPE => AttributeValue::Type(content.parse()?),
                    // parse #[musli(default_variant_name = "..")]
                    path if path == DEFAULT_VARIANT_NAME => AttributeValue::Lit(content.parse()?),
                    // parse #[musli(default_field_name = "..")]
                    path if path == DEFAULT_FIELD_NAME => AttributeValue::Lit(content.parse()?),
                    // parse #[musli(bounds = "..")]
                    path if path == BOUND => AttributeValue::Bound(content.parse()?),
                    // parse #[musli(decode_bounds = "..")]
                    path if path == DECODE_BOUND => AttributeValue::Bound(content.parse()?),
                    path => {
                        let p = format_path(path);
                        return Err(syn::Error::new(
                            path.span(),
                            format_args!("#[{ATTR}({p})] unsupported type attribute"),
                        ));
                    }
                };

                attributes.push(Attribute::KeyValue(path, value));
            } else {
                attributes.push(Attribute::Path(path));
            }

            if content.parse::<Option<Token![,]>>()?.is_none() {
                break;
            }
        }

        Ok(Self {
            _parens: parens,
            mode,
            attributes,
        })
    }
}

struct VariantAttributes {
    _parens: syn::token::Paren,
    mode: Option<syn::ExprPath>,
    attributes: Vec<Attribute>,
}

impl Parse for VariantAttributes {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut mode = None;
        let mut attributes = Vec::new();

        // Skip over `musli` attribute.
        let _ = input.parse::<syn::Path>();

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(mode = <ident>)]
                    path if path == MODE => {
                        mode = Some(content.parse()?);

                        if content.parse::<Option<Token![,]>>()?.is_none() {
                            break;
                        }

                        continue;
                    }
                    // parse #[musli(name_type = <type>)]
                    path if path == NAME_TYPE => AttributeValue::Type(content.parse()?),
                    // parse #[musli(rename = <expr>)]
                    path if path == RENAME => AttributeValue::Expr(content.parse()?),
                    // parse #[musli(default_field_name = <expr>)]
                    path if path == DEFAULT_FIELD_NAME => AttributeValue::Lit(content.parse()?),
                    path => {
                        let p = format_path(path);
                        return Err(syn::Error::new(
                            path.span(),
                            format_args!("#[{ATTR}({p})] unsupported variant attribute"),
                        ));
                    }
                };

                attributes.push(Attribute::KeyValue(path, value));
            } else {
                attributes.push(Attribute::Path(path));
            }

            if content.parse::<Option<Token![,]>>()?.is_none() {
                break;
            }
        }

        Ok(Self {
            _parens: parens,
            mode,
            attributes,
        })
    }
}

struct FieldAttributes {
    _parens: syn::token::Paren,
    mode: Option<syn::ExprPath>,
    attributes: Vec<Attribute>,
}

impl Parse for FieldAttributes {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut mode = None;
        let mut attributes = Vec::new();

        // Skip over `musli` attribute.
        let _ = input.parse::<syn::Path>();

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(mode = <ident>)]
                    path if path == MODE => {
                        mode = Some(content.parse()?);

                        if content.parse::<Option<Token![,]>>()?.is_none() {
                            break;
                        }

                        continue;
                    }
                    // parse #[musli(with)]
                    path if path == WITH => AttributeValue::Path(content.parse()?),
                    // parse #[musli(skip_encoding_if)]
                    path if path == SKIP_ENCODING_IF => AttributeValue::Path(content.parse()?),
                    // parse #[musli(rename = <expr>)]
                    path if path == RENAME => AttributeValue::Expr(content.parse()?),
                    path => {
                        let p = format_path(path);
                        return Err(syn::Error::new(
                            path.span(),
                            format_args!("#[{ATTR}({p})] unsupported field attribute"),
                        ));
                    }
                };

                attributes.push(Attribute::KeyValue(path, value));
            } else {
                attributes.push(Attribute::Path(path));
            }

            if content.parse::<Option<Token![,]>>()?.is_none() {
                break;
            }
        }

        Ok(Self {
            _parens: parens,
            mode,
            attributes,
        })
    }
}

/// Format implementation of a path which ignores anything except the
/// identifier components.
fn format_path(path: &syn::Path) -> impl fmt::Display + '_ {
    FormatPath { path }
}

struct FormatPath<'a> {
    path: &'a syn::Path,
}

impl<'a> fmt::Display for FormatPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = self.path.segments.iter();
        let last = it.next_back();

        for part in it {
            write!(f, "{}::", part.ident)?;
        }

        if let Some(part) = last {
            write!(f, "{}", part.ident)?;
        }

        Ok(())
    }
}
