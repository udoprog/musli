use core::fmt;
use std::collections::HashMap;

use crate::internals::symbol::*;
use crate::internals::Ctxt;
use crate::internals::Mode;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::parse;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Ident;

#[derive(Debug, Clone)]
pub enum Tagging {
    /// Externally tagged.
    External,
    /// The type is internally tagged by the field given by the expression.
    Internal(syn::Expr),
}

impl Default for Tagging {
    fn default() -> Self {
        Tagging::External
    }
}

/// The kind of tag to use.
#[derive(Debug, Clone, Copy)]
pub enum DefaultTag {
    Index,
    Name,
}

impl Default for DefaultTag {
    fn default() -> Self {
        Self::Index
    }
}

/// If the type is tagged or not.
#[derive(Debug, Clone)]
pub enum Packing {
    Tagged(Tagging),
    Packed,
    Transparent,
}

impl fmt::Display for Packing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Packing::Tagged(Tagging::External) => write!(f, "tagged"),
            Packing::Tagged(Tagging::Internal(..)) => write!(f, "internally tagged"),
            Packing::Packed => write!(f, "packed"),
            Packing::Transparent => write!(f, "transparent"),
        }
    }
}

impl Default for Packing {
    fn default() -> Self {
        Self::Tagged(Tagging::External)
    }
}

#[derive(Default)]
struct InnerTypeAttr {
    /// `#[musli(crate = <path>)]`.
    krate: Option<(Span, syn::ExprPath)>,
    /// `#[musli(tag_type)]`.
    tag_type: Option<(Span, syn::Type)>,
    /// `#[musli(default_variant_tag = "..")]`.
    default_variant_tag: DefaultTag,
    /// `#[musli(default_field_tag = "..")]`.
    default_field_tag: DefaultTag,
    /// `#[musli(packed)]` or `#[musli(transparent)]`.
    packing: Option<(Span, Packing)>,
}

impl InnerTypeAttr {
    /// Update packing of type.
    fn set_packing(&mut self, cx: &Ctxt, span: Span, packing: Packing) {
        if let Some((_, existing)) = &self.packing {
            cx.error_span(
                span,
                format!(
                    "#[{}({})] cannot be combined with #[{}({})]",
                    ATTR, packing, ATTR, existing
                ),
            );
        }

        self.packing = Some((span, packing));
    }

    fn set_crate(&mut self, cx: &Ctxt, span: Span, path: syn::ExprPath) {
        if let Some((span, _)) = self.krate {
            cx.error_span(
                span,
                format!("#[{}({})] cannot be used multiple times", ATTR, CRATE),
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
    pub(crate) fn packing_span(&self, mode: Mode<'_>) -> Option<&(Span, Packing)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.packing.as_ref())
            .or(self.root.packing.as_ref())
    }

    /// Indicates the packing state of the type.
    pub(crate) fn packing(&self, mode: Mode<'_>) -> Option<&Packing> {
        Some(&self.packing_span(mode)?.1)
    }

    /// Default field tag.
    pub(crate) fn default_field_tag(&self, mode: Mode<'_>) -> DefaultTag {
        mode.ident
            .and_then(|m| Some(self.modes.get(m)?.default_field_tag))
            .unwrap_or(self.root.default_field_tag)
    }

    pub(crate) fn default_variant_tag(&self, mode: Mode<'_>) -> DefaultTag {
        mode.ident
            .and_then(|m| Some(self.modes.get(m)?.default_variant_tag))
            .unwrap_or(self.root.default_variant_tag)
    }

    /// Get the tag type of the type.
    pub(crate) fn tag_type(&self, mode: Mode<'_>) -> Option<&(Span, syn::Type)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.tag_type.as_ref())
            .or_else(|| self.root.tag_type.as_ref())
    }

    /// Get the configured crate, or fallback to default.
    pub(crate) fn crate_or_default(&self) -> syn::ExprPath {
        if let Some((_, krate)) = &self.root.krate {
            krate.clone()
        } else {
            let path = syn::Path::from(syn::Ident::new(&*ATTR, Span::call_site()));

            syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: path.into(),
            }
        }
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
                format!("#[{}] multiple encode methods specified", ATTR),
            );
        } else {
            self.encode_path = Some((span, encode_path));
        }
    }

    fn set_decode_path(&mut self, cx: &Ctxt, span: Span, decode_path: syn::ExprPath) {
        if self.decode_path.is_some() {
            cx.error_spanned_by(
                decode_path,
                format!("#[{}] multiple decode methods specified", ATTR),
            );
        } else {
            self.decode_path = Some((span, decode_path));
        }
    }

    fn set_skip_encoding_if(&mut self, cx: &Ctxt, span: Span, skip_encoding_if: syn::ExprPath) {
        if self.skip_encoding_if.is_some() {
            cx.error_spanned_by(
                skip_encoding_if,
                format!("#[{}] multiple skip_encoding_if methods specified", ATTR),
            );
        } else {
            self.skip_encoding_if = Some((span, skip_encoding_if));
        }
    }
}

#[derive(Default)]
struct InternalVariantAttr {
    /// `#[musli(tag_type)]`.
    tag_type: Option<(Span, syn::Type)>,
    /// Rename a field to the given expression.
    rename: Option<(Span, syn::Expr)>,
    /// `#[musli(packed)]` or `#[musli(transparent)]`.
    packing: Option<(Span, Packing)>,
    /// `#[musli(default)]`.
    default: Option<Span>,
    /// `#[musli(default_field_tag = "..")]`.
    default_field_tag: Option<DefaultTag>,
}

impl InternalVariantAttr {
    /// Update packing of type.
    fn set_packing(&mut self, cx: &Ctxt, span: Span, packing: Packing) {
        if let Some((_, existing)) = &self.packing {
            cx.error_span(
                span,
                format!(
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
            .or_else(|| self.root.default)
    }

    /// Test if the `#[musli(rename)]` tag is specified.
    pub(crate) fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.rename.as_ref())
            .or_else(|| self.root.rename.as_ref())
    }

    /// Indicates if the tagged state of the variant is set.
    pub(crate) fn packing(&self, mode: Mode<'_>) -> Option<&Packing> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.packing.as_ref())
            .or_else(|| self.root.packing.as_ref())
            .map(|p| &p.1)
    }

    /// Default field tag.
    pub(crate) fn default_field_tag(&self, mode: Mode<'_>) -> Option<DefaultTag> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.default_field_tag)
            .or_else(|| self.root.default_field_tag)
    }

    /// Get the tag type of the type.
    pub(crate) fn tag_type(&self, mode: Mode<'_>) -> Option<&(Span, syn::Type)> {
        mode.ident
            .and_then(|m| self.modes.get(m))
            .map(|a| a.tag_type.as_ref())
            .unwrap_or_else(|| self.root.tag_type.as_ref())
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
            .or_else(|| self.root.default)
    }

    /// Test if the `#[musli(rename)]` tag is specified.
    pub(crate) fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.rename.as_ref())
            .or_else(|| self.root.rename.as_ref())
    }

    /// Expand encode of the given field.
    pub(crate) fn encode_path(&self, mode: Mode<'_>, span: Span) -> (Span, TokenStream) {
        let encode_path = mode
            .ident
            .and_then(|m| self.modes.get(m)?.encode_path.as_ref())
            .or_else(|| self.root.encode_path.as_ref());

        if let Some((span, encode_path)) = encode_path {
            let mode_ident = mode.mode_ident();
            (
                *span,
                quote_spanned!(*span => #encode_path::<#mode_ident, _>),
            )
        } else {
            let encode_path = mode.encode_t_encode();
            (span, encode_path)
        }
    }

    /// Expand decode of the given field.
    pub(crate) fn decode_path(&self, mode: Mode<'_>, span: Span) -> (Span, TokenStream) {
        let decode_path = mode
            .ident
            .and_then(|m| self.modes.get(m)?.decode_path.as_ref())
            .or_else(|| self.root.decode_path.as_ref());

        if let Some((span, decode_path)) = decode_path {
            let mode_ident = mode.mode_ident();
            (
                *span,
                quote_spanned!(*span => #decode_path::<#mode_ident, _>),
            )
        } else {
            let decode_path = mode.decode_t_decode();
            (span, decode_path)
        }
    }

    /// Get skip encoding if.
    pub(crate) fn skip_encoding_if(&self, mode: Mode<'_>) -> Option<(Span, &syn::ExprPath)> {
        let (span, path) = mode
            .ident
            .and_then(|m| self.modes.get(m)?.skip_encoding_if.as_ref())
            .or_else(|| self.root.skip_encoding_if.as_ref())?;

        Some((*span, path))
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
                            attr.set_packing(
                                cx,
                                path.span(),
                                Packing::Tagged(Tagging::Internal(expr)),
                            );
                        }
                    }
                    // parse #[musli(crate = <path>)]
                    Attribute::KeyValue(path, value) if path == CRATE => {
                        if let Some(path) = value_as_path(cx, CRATE, value) {
                            attr.set_crate(cx, path.span(), path);
                        }
                    }
                    // parse #[musli(tag_type = <type>)]
                    Attribute::KeyValue(path, value) if path == TAG_TYPE => {
                        if let Some(ty) = value_as_type(cx, TAG_TYPE, value) {
                            attr.tag_type = Some((path.span(), ty));
                        }
                    }
                    // parse #[musli(default_variant_tag = "..")]
                    Attribute::KeyValue(path, expr) if path == DEFAULT_VARIANT_TAG => {
                        if let Some(tag) = parse_value_string(cx, DEFAULT_VARIANT_TAG, expr) {
                            attr.default_variant_tag = match tag.value().as_str() {
                                "index" => DefaultTag::Index,
                                "name" => DefaultTag::Name,
                                _ => {
                                    cx.error_spanned_by(
                                        tag,
                                        format!("illegal #[{}({})] value", ATTR, TAG),
                                    );
                                    continue;
                                }
                            };
                        }
                    }
                    // parse #[musli(default_field_tag = "..")]
                    Attribute::KeyValue(path, expr) if path == DEFAULT_FIELD_TAG => {
                        if let Some(tag) = parse_value_string(cx, DEFAULT_FIELD_TAG, expr) {
                            attr.default_field_tag = match tag.value().as_str() {
                                "index" => DefaultTag::Index,
                                "name" => DefaultTag::Name,
                                _ => {
                                    cx.error_spanned_by(
                                        tag,
                                        format!("illegal #[{}({})] value", ATTR, TAG),
                                    );
                                    continue;
                                }
                            };
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
                        cx.error_span(meta.span(), format!("unsupported #[{}] attribute", ATTR));
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
                        if let Some(path) = value_as_path(cx, WITH, value) {
                            let mut encode_path = path.clone();

                            encode_path
                                .path
                                .segments
                                .push(Ident::new("encode", Span::call_site()).into());

                            attr.set_encode_path(cx, path.span(), encode_path);

                            let mut decode_path = path.clone();

                            decode_path
                                .path
                                .segments
                                .push(Ident::new("decode", Span::call_site()).into());

                            attr.set_decode_path(cx, path.span(), decode_path);
                        }
                    }
                    // parse #[musli(skip_encoding_if)]
                    Attribute::KeyValue(path, expr) if path == SKIP_ENCODING_IF => {
                        if let Some(path) = value_as_path(cx, SKIP_ENCODING_IF, expr) {
                            attr.set_skip_encoding_if(cx, path.span(), path.clone());
                        }
                    }
                    // parse #[musli(tag = <expr>)]
                    Attribute::KeyValue(path, value) if path == TAG => {
                        let span = path.span();

                        if let Some(expr) = value_as_expr(cx, TAG, value) {
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
                                format!("#[{}({})] was previously defined here", ATTR, DEFAULT),
                            );
                        } else {
                            attr.default = Some(path.span());
                        }
                    }
                    meta => {
                        cx.error_span(meta.span(), format!("unsupported #[{}] attribute", ATTR));
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
                    // parse #[musli(tag_type = <type>)]
                    Attribute::KeyValue(path, value) if path == TAG_TYPE => {
                        if let Some(ty) = value_as_type(cx, TAG_TYPE, value) {
                            attr.tag_type = Some((path.span(), ty));
                        }
                    }
                    // parse #[musli(tag = <expr>)]
                    Attribute::KeyValue(path, value) if path == TAG => {
                        let span = path.span();

                        if let Some(expr) = value_as_expr(cx, TAG, value) {
                            if let Some((span, _)) = &attr.rename {
                                cx.error_span(*span, "conflicting attribute");
                            } else {
                                attr.rename = Some((span, expr));
                            }
                        }
                    }
                    // parse #[musli(default_field_tag = "..")]
                    Attribute::KeyValue(path, expr) if path == DEFAULT_FIELD_TAG => {
                        if let Some(tag) = parse_value_string(cx, DEFAULT_FIELD_TAG, expr) {
                            attr.default_field_tag = Some(match tag.value().as_str() {
                                "index" => DefaultTag::Index,
                                "name" => DefaultTag::Name,
                                _ => {
                                    cx.error_spanned_by(
                                        tag,
                                        format!("illegal #[{}({})] value", ATTR, TAG),
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
                        cx.error_span(meta.span(), format!("unsupported #[{}] attribute", ATTR));
                    }
                }
            }
        }
    }

    attr
}

/// Get expression path.
fn value_as_path<'a>(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::ExprPath> {
    match value {
        AttributeValue::Path(path) => Some(path),
        _ => {
            cx.error_span(
                value.span(),
                format!("#[{} = ...] should be a simple path", attr),
            );
            None
        }
    }
}

/// Get expression from value.
fn value_as_expr<'a>(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::Expr> {
    match value {
        AttributeValue::Expr(expr) => Some(expr),
        _ => {
            cx.error_span(
                value.span(),
                format!("#[{} = ...] should be a simple path", attr),
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
            cx.error_span(expr.span(), format!("#[{} = ...] should be a string", attr));
            None
        }
    }
}

/// Get expression as a type.
fn value_as_type(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::Type> {
    match value {
        AttributeValue::Type(ty) => Some(ty),
        expr => {
            cx.error_span(expr.span(), format!("#[{} = ...] should be a type", attr));
            None
        }
    }
}

/// Parse musli attributes.
fn parse_musli_attrs<T>(cx: &Ctxt, attr: &syn::Attribute) -> Option<T>
where
    T: Parse,
{
    if attr.path != ATTR {
        return None;
    }

    let attributes: T = match syn::parse2(attr.tokens.clone()) {
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
}

impl Spanned for AttributeValue {
    fn span(&self) -> Span {
        match self {
            AttributeValue::Path(value) => value.span(),
            AttributeValue::Type(value) => value.span(),
            AttributeValue::Expr(value) => value.span(),
            AttributeValue::Lit(value) => value.span(),
        }
    }
}

enum Attribute {
    Path(syn::Path),
    KeyValue(syn::Path, AttributeValue),
}

impl Spanned for Attribute {
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

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<syn::Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(mode = <ident>)]
                    path if path == MODE => {
                        mode = Some(content.parse()?);

                        if content.parse::<Option<syn::Token![,]>>()?.is_none() {
                            break;
                        }

                        continue;
                    }
                    // parse #[musli(tag = <expr>)]
                    path if path == TAG => AttributeValue::Expr(content.parse()?),
                    // parse #[musli(crate = <path>)]
                    path if path == CRATE => AttributeValue::Path(content.parse()?),
                    // parse #[musli(tag_type = <type>)]
                    path if path == TAG_TYPE => AttributeValue::Type(content.parse()?),
                    // parse #[musli(default_variant_tag = "..")]
                    path if path == DEFAULT_VARIANT_TAG => AttributeValue::Lit(content.parse()?),
                    // parse #[musli(default_field_tag = "..")]
                    path if path == DEFAULT_FIELD_TAG => AttributeValue::Lit(content.parse()?),
                    path => {
                        return Err(syn::Error::new(path.span(), "unsupported attribute"));
                    }
                };

                attributes.push(Attribute::KeyValue(path, value));
            } else {
                attributes.push(Attribute::Path(path));
            }

            if content.parse::<Option<syn::Token![,]>>()?.is_none() {
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

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<syn::Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(mode = <ident>)]
                    path if path == MODE => {
                        mode = Some(content.parse()?);

                        if content.parse::<Option<syn::Token![,]>>()?.is_none() {
                            break;
                        }

                        continue;
                    }
                    // parse #[musli(tag_type = <type>)]
                    path if path == TAG_TYPE => AttributeValue::Type(content.parse()?),
                    // parse #[musli(tag = <expr>)]
                    path if path == TAG => AttributeValue::Expr(content.parse()?),
                    // parse #[musli(default_field_tag = <expr>)]
                    path if path == DEFAULT_FIELD_TAG => AttributeValue::Lit(content.parse()?),
                    path => {
                        return Err(syn::Error::new(path.span(), "unsupported attribute"));
                    }
                };

                attributes.push(Attribute::KeyValue(path, value));
            } else {
                attributes.push(Attribute::Path(path));
            }

            if content.parse::<Option<syn::Token![,]>>()?.is_none() {
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

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<syn::Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(mode = <ident>)]
                    path if path == MODE => {
                        mode = Some(content.parse()?);

                        if content.parse::<Option<syn::Token![,]>>()?.is_none() {
                            break;
                        }

                        continue;
                    }
                    // parse #[musli(with)]
                    path if path == WITH => AttributeValue::Path(content.parse()?),
                    // parse #[musli(skip_encoding_if)]
                    path if path == SKIP_ENCODING_IF => AttributeValue::Path(content.parse()?),
                    // parse #[musli(tag = <expr>)]
                    path if path == TAG => AttributeValue::Expr(content.parse()?),
                    path => {
                        return Err(syn::Error::new(path.span(), "unsupported attribute"));
                    }
                };

                attributes.push(Attribute::KeyValue(path, value));
            } else {
                attributes.push(Attribute::Path(path));
            }

            if content.parse::<Option<syn::Token![,]>>()?.is_none() {
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
