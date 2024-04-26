use std::collections::HashMap;
use std::mem;

use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::NameMethod;
use crate::expander::UnsizedMethod;
use crate::internals::name::NameAll;
use crate::internals::ATTR;
use crate::internals::{Ctxt, Mode};

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

#[derive(Clone, Copy)]
pub(crate) enum EnumTagging<'a> {
    /// Use the default tagging method, as provided by the encoder-specific
    /// method.
    Default,
    /// Only the tag is encoded.
    Empty,
    /// The type is internally tagged by the field given by the expression.
    Internal { tag: &'a syn::Expr },
    /// An enumerator is adjacently tagged.
    Adjacent {
        tag: &'a syn::Expr,
        content: &'a syn::Expr,
    },
}

/// If the type is tagged or not.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packing {
    #[default]
    Tagged,
    Packed,
    Transparent,
}

macro_rules! merge {
    ($self:expr, $cx:expr, $new:expr, $field:ident, $only:expr) => {{
        for $field in $new.$field {
            let out = match $only {
                None => &mut $self.$field.any,
                Some(Only::Decode) => &mut $self.$field.decode,
                Some(Only::Encode) => &mut $self.$field.encode,
            };

            if out.is_some() {
                $cx.error_span(
                    $field.0,
                    format_args!(
                        "#[{}] multiple {} attributes specified",
                        ATTR,
                        stringify!($field)
                    ),
                );
            } else {
                *out = Some($field);
            }
        }
    }};
}

macro_rules! layer {
    ($attr:ident, $new:ident, $layer:ident {
        $($(#[$($single_meta:meta)*])* $single:ident: $single_ty:ty,)* $(,)?
        @multiple
        $($(#[$($multiple_meta:meta)*])* $multiple:ident: $multiple_ty:ty,)* $(,)?
    }) => {
        #[derive(Default)]
        pub(crate) struct $attr {
            root: $layer,
            modes: HashMap<ModeKind, $layer>,
        }

        impl $attr {
            fn by_mode<A, O>(&self, mode: Mode<'_>, access: A) -> Option<&O>
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
                pub(crate) fn $single(&self, mode: Mode<'_>) -> Option<&(Span, $single_ty)> {
                    self.by_mode(mode, |m| {
                        match mode.only {
                            Only::Encode if m.$single.encode.is_some() => m.$single.encode.as_ref(),
                            Only::Decode if m.$single.decode.is_some() => m.$single.decode.as_ref(),
                            _ => m.$single.any.as_ref(),
                        }
                    })
                }
            )*

            $(
                #[allow(unused)]
                pub(crate) fn $multiple(&self, mode: Mode<'_>) -> &[(Span, $multiple_ty)] {
                    self.by_mode(mode, |m| {
                        match mode.only {
                            Only::Encode if !m.$multiple.encode.is_empty() => Some(&m.$multiple.encode[..]),
                            Only::Decode if !m.$multiple.decode.is_empty() => Some(&m.$multiple.decode[..]),
                            _ if !m.$multiple.any.is_empty() => Some(&m.$multiple.any[..]),
                            _ => None,
                        }
                    }).unwrap_or_default()
                }
            )*
        }

        #[derive(Default)]
        struct $new {
            $($(#[$($single_meta)*])* $single: Vec<(Span, $single_ty)>,)*
            $($(#[$($multiple_meta)*])* $multiple: Vec<(Span, $multiple_ty)>,)*
        }

        #[derive(Default)]
        struct $layer {
            $($(#[$($single_meta)*])* $single: OneOf<Option<(Span, $single_ty)>>,)*
            $($(#[$($multiple_meta)*])* $multiple: OneOf<Vec<(Span, $multiple_ty)>>,)*
        }

        impl $layer {
            /// Merge attributes.
            fn merge_with(&mut self, cx: &Ctxt, new: $new, only: Option<Only>) {
                $(
                    merge!(self, cx, new, $single, only);
                )*

                $(
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
                )*
            }
        }
    }
}

layer! {
    TypeAttr, TypeLayerNew, TypeLayer {
        /// `#[musli(crate = <path>)]`.
        krate: syn::Path,
        /// `#[musli(name_type)]`.
        name_type: syn::Type,
        /// `#[musli(name_all = "..")]`.
        name_all: NameAll,
        /// `#[musli(name_method = "..")]`.
        name_method: NameMethod,
        /// `#[musli(name_format_with)]`.
        name_format_with: syn::Path,
        /// If `#[musli(tag = <expr>)]` is specified.
        tag: syn::Expr,
        /// If `#[musli(content = <expr>)]` is specified.
        content: syn::Expr,
        /// `#[musli(packed)]` or `#[musli(transparent)]`.
        packing: Packing,
        @multiple
        /// Bounds in a where predicate.
        bounds: syn::WherePredicate,
        /// Bounds to require for a `Decode` implementation.
        decode_bounds: syn::WherePredicate,
    }
}

impl TypeAttr {
    pub(crate) fn is_name_type_ambiguous(&self, mode: Mode<'_>) -> bool {
        self.name_type(mode).is_none()
            && self.name_all(mode).is_none()
            && self.name_method(mode).is_none()
    }

    pub(crate) fn enum_tagging_span(&self, mode: Mode<'_>) -> Option<Span> {
        let tag = self.tag(mode);
        let content = self.content(mode);
        Some(tag.or(content)?.0)
    }

    /// Indicates the state of enum tagging.
    pub(crate) fn enum_tagging(&self, mode: Mode<'_>) -> Option<EnumTagging<'_>> {
        let (_, tag) = self.tag(mode)?;

        Some(match self.content(mode) {
            Some((_, content)) => EnumTagging::Adjacent { tag, content },
            _ => EnumTagging::Internal { tag },
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
                meta.input.parse::<Token![=]>()?;
                new.tag.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // #[musli(content = <expr>)]
            if meta.path.is_ident("content") {
                meta.input.parse::<Token![=]>()?;
                new.content.push((meta.path.span(), meta.input.parse()?));
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

            // #[musli(name_type = <type>)]
            if meta.path.is_ident("name_type") {
                meta.input.parse::<Token![=]>()?;
                new.name_type.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // #[musli(name_format_with = <path>)]
            if meta.path.is_ident("name_format_with") {
                meta.input.parse::<Token![=]>()?;
                new.name_format_with
                    .push((meta.path.span(), meta.input.parse()?));
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

            // #[musli(name_method = "..")]
            if meta.path.is_ident("name_method") {
                new.name_method
                    .push((meta.path.span(), parse_name_method(&meta)?));
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

fn parse_name_method(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<NameMethod, syn::Error> {
    meta.input.parse::<Token![=]>()?;

    let string: syn::LitStr = meta.input.parse()?;
    let s = string.value();

    match s.as_str() {
        "value" => Ok(NameMethod::Value),
        "unsized" => Ok(NameMethod::Unsized(UnsizedMethod::Default)),
        "unsized_bytes" => Ok(NameMethod::Unsized(UnsizedMethod::Bytes)),
        _ => Err(syn::Error::new_spanned(
            string,
            "#[musli(name_method = ..)]: Bad value, expected one of \"value\", \"unsized\", \"unsized_bytes\"",
        )),
    }
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
        /// `#[musli(name_type)]`.
        name_type: syn::Type,
        /// `#[musli(name_format_with)]`.
        name_format_with: syn::Path,
        /// Name a variant with the given expression.
        name: syn::Expr,
        /// Pattern used to match the given field when decoding.
        pattern: syn::Pat,
        /// `#[musli(name_all = "..")]`.
        name_all: NameAll,
        /// `#[musli(name_method = "..")]`.
        name_method: NameMethod,
        /// `#[musli(packed)]` or `#[musli(transparent)]`.
        packing: Packing,
        /// `#[musli(default)]`.
        default_variant: (),
        @multiple
    }
}

impl VariantAttr {
    pub(crate) fn is_name_type_ambiguous(&self, mode: Mode<'_>) -> bool {
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

            // #[musli(name_type = <type>)]
            if meta.path.is_ident("name_type") {
                meta.input.parse::<Token![=]>()?;
                new.name_type.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // #[musli(name_format_with = <path>)]
            if meta.path.is_ident("name_format_with") {
                meta.input.parse::<Token![=]>()?;
                new.name_format_with
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

            // #[musli(name_method = "..")]
            if meta.path.is_ident("name_method") {
                new.name_method
                    .push((meta.path.span(), parse_name_method(&meta)?));
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
        @multiple
    }
}

impl Field {
    /// Expand encode of the given field.
    pub(crate) fn encode_path_expanded(&self, mode: Mode<'_>, span: Span) -> (Span, syn::Path) {
        let encode_path = self.encode_path(mode);

        if let Some((span, encode_path)) = encode_path {
            (*span, encode_path.clone())
        } else {
            let field_encoding = self.encoding(mode).map(|&(_, e)| e).unwrap_or_default();
            let encode_path = mode.encode_t_encode(field_encoding);
            (span, encode_path)
        }
    }

    /// Expand decode of the given field.
    pub(crate) fn decode_path_expanded(&self, mode: Mode<'_>, span: Span) -> (Span, syn::Path) {
        let decode_path = self.decode_path(mode);

        if let Some((span, decode_path)) = decode_path {
            (*span, decode_path.clone())
        } else {
            let field_encoding = self.encoding(mode).map(|&(_, e)| e).unwrap_or_default();
            let decode_path = mode.decode_t_decode(field_encoding);
            (span, decode_path)
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
