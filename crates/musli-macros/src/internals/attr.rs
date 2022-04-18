use core::fmt;

use crate::internals::symbol::*;
use crate::internals::Ctxt;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::parse;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Ident;

/// The kind of tag to use.
#[derive(Debug, Clone, Copy)]
pub enum Tag {
    Index,
    Name,
}

impl Default for Tag {
    fn default() -> Self {
        Self::Index
    }
}

/// If the type is tagged or not.
#[derive(Debug, Clone, Copy)]
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
pub(crate) struct TypeAttr {
    /// `#[musli(tag_type)]`.
    pub(crate) tag_type: Option<(Span, syn::Type)>,
    /// `#[musli(variant_tag)]`.
    pub(crate) variant_tag: Tag,
    /// `#[musli(field_tag)]`.
    pub(crate) field_tag: Tag,
    /// `#[musli(packed)]` or `#[musli(transparent)]`.
    pub(crate) packing: Option<(Span, Packing)>,
}

impl TypeAttr {
    /// Indicates the tag state of the type.
    pub(crate) fn packing(&self) -> Packing {
        match self.packing {
            Some((_, packing)) => packing,
            None => Packing::Tagged,
        }
    }

    /// Update packing of type.
    fn set_packing(&mut self, cx: &Ctxt, span: Span, packing: Packing) {
        if let Some((_, existing)) = self.packing {
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
pub(crate) struct FieldAttr {
    /// Module to use when decoding.
    pub(crate) encode_path: Option<(Span, syn::Path)>,
    /// Path to use when decoding.
    pub(crate) decode_path: Option<(Span, syn::Path)>,
    /// Method to check if we want to skip encoding.
    pub(crate) skip_encoding_if: Option<(Span, syn::Path)>,
    /// Rename a field to the given literal.
    pub(crate) rename: Option<(Span, syn::Expr)>,
    /// Use a default value for the field if it's not available.
    pub(crate) default: Option<Span>,
}

#[derive(Default)]
pub(crate) struct VariantAttr {
    /// `#[musli(tag_type)]`.
    pub(crate) tag_type: Option<(Span, syn::Type)>,
    /// Rename a field to the given literal.
    pub(crate) rename: Option<(Span, syn::Expr)>,
    /// `#[musli(packed)]` or `#[musli(transparent)]`.
    pub(crate) packing: Option<(Span, Packing)>,
}

impl VariantAttr {
    /// Indicates if the tagged state of the variant is set.
    pub(crate) fn packing(&self) -> Option<Packing> {
        Some(self.packing?.1)
    }

    /// Update packing of type.
    fn set_packing(&mut self, cx: &Ctxt, span: Span, packing: Packing) {
        if let Some((_, existing)) = self.packing {
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

impl FieldAttr {
    /// Expand encode of the given field.
    pub(crate) fn encode_path(
        &self,
        encode_trait: &TokenStream,
        span: Span,
    ) -> (Span, TokenStream) {
        if let Some((span, encode_path)) = &self.encode_path {
            (*span, quote_spanned!(*span => #encode_path))
        } else {
            (span, quote_spanned!(span => #encode_trait::encode))
        }
    }

    /// Expand decode of the given field.
    pub(crate) fn decode_path(
        &self,
        decode_trait: &TokenStream,
        span: Span,
    ) -> (Span, TokenStream) {
        if let Some((span, decode_path)) = &self.decode_path {
            (*span, quote_spanned!(*span => #decode_path))
        } else {
            (span, quote!(#decode_trait::decode))
        }
    }

    /// Get skip encoding if.
    pub(crate) fn skip_encoding_if(&self) -> Option<(Span, &syn::Path)> {
        let (span, path) = self.skip_encoding_if.as_ref()?;
        Some((*span, path))
    }

    fn set_encode_path(&mut self, cx: &Ctxt, span: Span, encode_path: syn::Path) {
        if self.encode_path.is_some() {
            cx.error_spanned_by(
                encode_path,
                format!("#[{}] multiple encode methods specified", ATTR),
            );
        } else {
            self.encode_path = Some((span, encode_path));
        }
    }

    fn set_decode_path(&mut self, cx: &Ctxt, span: Span, decode_path: syn::Path) {
        if self.decode_path.is_some() {
            cx.error_spanned_by(
                decode_path,
                format!("#[{}] multiple decode methods specified", ATTR),
            );
        } else {
            self.decode_path = Some((span, decode_path));
        }
    }

    fn set_skip_encoding_if(&mut self, cx: &Ctxt, span: Span, skip_encoding_if: syn::Path) {
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

pub(crate) fn type_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> TypeAttr {
    let mut attr = TypeAttr::default();

    for a in attrs {
        for attribute in parse_musli_attrs::<TypeAttributes>(cx, a)
            .map(|a| a.attributes)
            .into_iter()
            .flatten()
        {
            match attribute {
                // parse #[musli(tag_type = <type>)]
                Attribute::KeyValue(path, value) if path == TAG_TYPE => {
                    if let Some(ty) = value_as_type(cx, TAG_TYPE, value) {
                        attr.tag_type = Some((path.span(), ty));
                    }
                }
                // parse #[musli(variant_tag = "..")]
                Attribute::KeyValue(path, expr) if path == VARIANT => {
                    if let Some(tag) = parse_value_string(cx, VARIANT, expr) {
                        attr.variant_tag = match tag.value().as_str() {
                            "index" => Tag::Index,
                            "name" => Tag::Name,
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
                // parse #[musli(field_tag = "..")]
                Attribute::KeyValue(path, expr) if path == FIELD => {
                    if let Some(tag) = parse_value_string(cx, FIELD, expr) {
                        attr.field_tag = match tag.value().as_str() {
                            "index" => Tag::Index,
                            "name" => Tag::Name,
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

    attr
}

/// Parse field attributes.
pub(crate) fn field_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> FieldAttr {
    let mut attr = FieldAttr::default();

    for a in attrs {
        for attribute in parse_musli_attrs::<FieldAttributes>(cx, a)
            .map(|a| a.attributes)
            .into_iter()
            .flatten()
        {
            match attribute {
                // parse #[musli(with = <path>)]
                Attribute::KeyValue(path, value) if path == WITH => {
                    if let Some(path) = value_as_path(cx, WITH, value) {
                        let mut encode_path = path.clone();

                        encode_path
                            .segments
                            .push(Ident::new("encode", Span::call_site()).into());

                        attr.set_encode_path(cx, path.span(), encode_path);

                        let mut decode_path = path.clone();

                        decode_path
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

    attr
}

/// Parse variant attributes.
pub(crate) fn variant_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> VariantAttr {
    let mut attr = VariantAttr::default();

    for a in attrs {
        for m in parse_musli_attrs::<VariantAttributes>(cx, a)
            .map(|a| a.attributes)
            .into_iter()
            .flatten()
        {
            match m {
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
                // parse #[musli(transparent)]
                Attribute::Path(path) if path == TRANSPARENT => {
                    attr.set_packing(cx, path.span(), Packing::Transparent);
                }
                // parse #[musli(packed)]
                Attribute::Path(path) if path == PACKED => {
                    attr.set_packing(cx, path.span(), Packing::Packed);
                }
                meta => {
                    cx.error_span(meta.span(), format!("unsupported #[{}] attribute", ATTR));
                }
            }
        }
    }

    attr
}

/// Get expression path.
fn value_as_path<'a>(cx: &Ctxt, attr: Symbol, value: AttributeValue) -> Option<syn::Path> {
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
    Path(syn::Path),
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
    attributes: Vec<Attribute>,
}

impl Parse for TypeAttributes {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut attributes = Vec::new();

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<syn::Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(tag_type = <type>)]
                    path if path == TAG_TYPE => AttributeValue::Type(content.parse()?),
                    // parse #[musli(variant = "..")]
                    path if path == VARIANT => AttributeValue::Lit(content.parse()?),
                    // parse #[musli(field_tag = "..")]
                    path if path == FIELD => AttributeValue::Lit(content.parse()?),
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
            attributes,
        })
    }
}

struct VariantAttributes {
    _parens: syn::token::Paren,
    attributes: Vec<Attribute>,
}

impl Parse for VariantAttributes {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut attributes = Vec::new();

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<syn::Token![=]>>()? {
                let value = match &path {
                    // parse #[musli(tag_type = <type>)]
                    path if path == TAG_TYPE => AttributeValue::Type(content.parse()?),
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
            attributes,
        })
    }
}

struct FieldAttributes {
    _parens: syn::token::Paren,
    attributes: Vec<Attribute>,
}

impl Parse for FieldAttributes {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut attributes = Vec::new();

        let content;
        let parens = syn::parenthesized!(content in input);

        while !content.is_empty() {
            let path: syn::Path = content.parse()?;

            if let Some(..) = content.parse::<Option<syn::Token![=]>>()? {
                let value = match &path {
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
            attributes,
        })
    }
}
