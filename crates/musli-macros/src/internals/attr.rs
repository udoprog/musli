use core::fmt;

use crate::internals::symbol::*;
use crate::internals::Ctxt;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::parse;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Ident;
use syn::Meta::*;
use syn::NestedMeta::*;

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

/// A rename into either a tag or a name.
pub(crate) enum Rename {
    Tag(syn::LitInt),
    Name(syn::LitStr),
}

impl Rename {
    pub(crate) fn as_lit(&self) -> syn::Lit {
        match self {
            Rename::Tag(tag) => syn::Lit::Int(tag.clone()),
            Rename::Name(name) => syn::Lit::Str(name.clone()),
        }
    }
}

#[derive(Default)]
pub(crate) struct TypeAttr {
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
    pub(crate) encode_path: Option<(Span, syn::ExprPath)>,
    /// Path to use when decoding.
    pub(crate) decode_path: Option<(Span, syn::ExprPath)>,
    /// Method to check if we want to skip encoding.
    pub(crate) skip_encoding_if: Option<(Span, syn::ExprPath)>,
    /// Rename a field to the given literal.
    pub(crate) rename: Option<(Span, Rename)>,
    /// Use a default value for the field if it's not available.
    pub(crate) default: Option<Span>,
}

#[derive(Default)]
pub(crate) struct VariantAttr {
    /// Rename a field to the given literal.
    pub(crate) rename: Option<(Span, Rename)>,
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
    pub(crate) fn skip_encoding_if(&self) -> Option<(Span, &syn::ExprPath)> {
        let (span, path) = self.skip_encoding_if.as_ref()?;
        Some((*span, path))
    }

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

pub(crate) fn type_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> TypeAttr {
    let mut attr = TypeAttr::default();

    for a in attrs {
        for m in get_jobs_attrs(cx, a).into_iter().flatten() {
            match &m {
                // parse #[musli(variant_tag)]
                Meta(NameValue(m)) if m.path == VARIANT => {
                    if let Ok(tag) = get_lit_str(cx, VARIANT, &m.lit) {
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
                // parse #[musli(field_tag)]
                Meta(NameValue(m)) if m.path == FIELD => {
                    if let Ok(tag) = get_lit_str(cx, FIELD, &m.lit) {
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
                Meta(Path(path)) if path == PACKED => {
                    attr.set_packing(cx, m.span(), Packing::Packed);
                }
                // parse #[musli(flatten)]
                Meta(Path(path)) if path == TRANSPARENT => {
                    attr.set_packing(cx, m.span(), Packing::Transparent);
                }
                meta => {
                    cx.error_spanned_by(meta, format!("unsupported #[{}] attribute", ATTR));
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
        for m in get_jobs_attrs(cx, a).into_iter().flatten() {
            match &m {
                // parse #[musli(with)]
                Meta(NameValue(m)) if m.path == WITH => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, WITH, &m.lit) {
                        let mut encode_path = path.clone();

                        encode_path
                            .path
                            .segments
                            .push(Ident::new("encode", Span::call_site()).into());

                        attr.set_encode_path(cx, m.lit.span(), encode_path);

                        let mut decode_path = path.clone();

                        decode_path
                            .path
                            .segments
                            .push(Ident::new("decode", Span::call_site()).into());

                        attr.set_decode_path(cx, m.lit.span(), decode_path);
                    }
                }
                // parse #[musli(skip_encoding_if)]
                Meta(NameValue(m)) if m.path == SKIP_ENCODING_IF => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, SKIP_ENCODING_IF, &m.lit) {
                        attr.set_skip_encoding_if(cx, m.lit.span(), path);
                    }
                }
                // parse #[musli(tag)]
                Meta(NameValue(m)) if m.path == TAG => {
                    let span = m.span();

                    if let Some((span, _)) = &attr.rename {
                        cx.error_span(*span, "conflicting attribute");
                    } else if let Ok(string) = get_lit_int(cx, TAG, &m.lit) {
                        attr.rename = Some((span, Rename::Tag(string.clone())));
                    }
                }
                // parse #[musli(name)]
                Meta(NameValue(m)) if m.path == NAME => {
                    let span = m.span();

                    if let Some((span, _)) = &attr.rename {
                        cx.error_span(*span, "conflicting attribute");
                    } else if let Ok(string) = get_lit_str(cx, NAME, &m.lit) {
                        attr.rename = Some((span, Rename::Name(string.clone())));
                    }
                }
                // parse #[musli(default)]
                Meta(Path(path)) if path == DEFAULT => {
                    if let Some(span) = attr.default {
                        cx.error_span(
                            span,
                            format!("#[{}({})] was previously defined here", ATTR, DEFAULT),
                        );
                    } else {
                        attr.default = Some(m.span());
                    }
                }
                meta => {
                    cx.error_spanned_by(meta, format!("unsupported #[{}] attribute", ATTR));
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
        for m in get_jobs_attrs(cx, a).into_iter().flatten() {
            match &m {
                // parse #[musli(tag)]
                Meta(NameValue(m)) if m.path == TAG => {
                    let span = m.span();

                    if let Some((span, _)) = &attr.rename {
                        cx.error_span(*span, "conflicting attribute");
                    } else if let Ok(string) = get_lit_int(cx, TAG, &m.lit) {
                        attr.rename = Some((span, Rename::Tag(string.clone())));
                    }
                }
                // parse #[musli(name)]
                Meta(NameValue(m)) if m.path == NAME => {
                    let span = m.span();

                    if let Some((span, _)) = &attr.rename {
                        cx.error_span(*span, "conflicting attribute");
                    } else if let Ok(string) = get_lit_str(cx, NAME, &m.lit) {
                        attr.rename = Some((span, Rename::Name(string.clone())));
                    }
                }
                // parse #[musli(flatten)]
                Meta(Path(path)) if path == TRANSPARENT => {
                    attr.set_packing(cx, m.span(), Packing::Transparent);
                }
                // parse #[musli(packed)]
                Meta(Path(path)) if path == PACKED => {
                    attr.set_packing(cx, m.span(), Packing::Packed);
                }
                meta => {
                    cx.error_spanned_by(meta, format!("unsupported #[{}] attribute", ATTR));
                }
            }
        }
    }

    attr
}

fn get_lit_int<'a>(cx: &Ctxt, attr_name: Symbol, lit: &'a syn::Lit) -> Result<&'a syn::LitInt, ()> {
    get_lit_int2(cx, attr_name, attr_name, lit)
}

fn get_lit_int2<'a>(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitInt, ()> {
    if let syn::Lit::Int(lit) = lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected {} {} attribute to be an integer: `{} = \"...\"`",
                ATTR, attr_name, meta_item_name
            ),
        );
        Err(())
    }
}

fn get_lit_str<'a>(cx: &Ctxt, attr_name: Symbol, lit: &'a syn::Lit) -> Result<&'a syn::LitStr, ()> {
    get_lit_str2(cx, attr_name, attr_name, lit)
}

fn get_lit_str2<'a>(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected musli {} attribute to be a string: `{} = \"...\"`",
                attr_name, meta_item_name
            ),
        );
        Err(())
    }
}

fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_name: Symbol,
    lit: &syn::Lit,
) -> Result<syn::ExprPath, ()> {
    let string = get_lit_str(cx, attr_name, lit)?;
    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(lit, format!("failed to parse path: {:?}", string.value()));
    })
}

fn parse_lit_str<T>(s: &syn::LitStr) -> parse::Result<T>
where
    T: Parse,
{
    let tokens = spanned_tokens(s)?;
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> parse::Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(crate::internals::respan::respan(stream, s.span()))
}

/// Parse musli attributes.
pub(crate) fn get_jobs_attrs(
    cx: &Ctxt,
    attr: &syn::Attribute,
) -> Option<impl Iterator<Item = syn::NestedMeta>> {
    if attr.path != ATTR {
        return None;
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Some(meta.nested.into_iter()),
        Ok(other) => {
            cx.error_spanned_by(other, "expected #[musli(...)]");
            None
        }
        Err(err) => {
            cx.syn_error(err);
            None
        }
    }
}
