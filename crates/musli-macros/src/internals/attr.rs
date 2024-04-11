use std::collections::HashMap;
use std::fmt;
use std::mem;

use proc_macro2::Span;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::{self, TagMethod};
use crate::internals::ATTR;
use crate::internals::{Ctxt, Mode};

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
    /// An enumerator is adjacently tagged.
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
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packing {
    #[default]
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
            modes: HashMap<syn::Path, $layer>,
        }

        impl $attr {
            fn by_mode<A, O>(&self, mode: Mode<'_>, access: A) -> Option<&O>
            where
                A: Copy + Fn(&$layer) -> Option<&O>,
                O: ?Sized,
            {
                if let Some(value) = mode.ident.and_then(|m| self.modes.get(m).and_then(access)) {
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
        /// `#[musli(name_format_with)]`.
        name_format_with: syn::Path,
        /// `#[musli(default_variant = "..")]`.
        default_variant: DefaultTag,
        /// `#[musli(default_field = "..")]`.
        default_field: DefaultTag,
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
    pub(crate) fn enum_tagging_span(&self, mode: Mode<'_>) -> Option<Span> {
        let tag = self.tag(mode);
        let content = self.content(mode);
        Some(tag.or(content)?.0)
    }

    /// Indicates the state of enum tagging.
    pub(crate) fn enum_tagging(&self, mode: Mode<'_>) -> Option<EnumTagging<'_>> {
        let (_, tag) = self.tag(mode)?;

        let tag_method = expander::determine_tag_method(tag);
        let tag = EnumTag {
            value: tag,
            method: tag_method,
        };

        match self.content(mode) {
            Some((_, content)) => Some(EnumTagging::Adjacent { tag, content }),
            _ => Some(EnumTagging::Internal { tag }),
        }
    }

    /// Get the configured crate, or fallback to default.
    pub(crate) fn crate_or_default(&self) -> syn::Path {
        if let Some((_, krate)) = self.root.krate.any.as_ref() {
            krate.clone()
        } else {
            let mut path = syn::Path::from(syn::Ident::new(ATTR, Span::call_site()));
            path.leading_colon = Some(<Token![::]>::default());
            path
        }
    }
}

pub(crate) fn type_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> TypeAttr {
    let mut attr = TypeAttr::default();

    for a in attrs {
        if !a.path().is_ident(ATTR) {
            continue;
        }

        let mut new = TypeLayerNew::default();
        let mut mode = None::<syn::Path>;
        let mut only = None;

        let result = a.parse_nested_meta(|meta| {
            // parse #[musli(mode = <path>)]
            if meta.path.is_ident("mode") {
                meta.input.parse::<Token![=]>()?;
                mode = Some(meta.input.parse()?);
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

            // parse #[musli(tag = <expr>)]
            if meta.path.is_ident("tag") {
                meta.input.parse::<Token![=]>()?;
                new.tag.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(content = <expr>)]
            if meta.path.is_ident("content") {
                meta.input.parse::<Token![=]>()?;
                new.content.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(crate = <path>)]
            if meta.path.is_ident("crate") {
                let path = if meta.input.parse::<Option<Token![=]>>()?.is_some() {
                    meta.input.parse()?
                } else {
                    syn::parse_quote!(crate)
                };

                new.krate.push((meta.path.span(), path));
                return Ok(());
            }

            // parse #[musli(name_type = <type>)]
            if meta.path.is_ident("name_type") {
                meta.input.parse::<Token![=]>()?;
                new.name_type.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(name_format_with = <path>)]
            if meta.path.is_ident("name_format_with") {
                meta.input.parse::<Token![=]>()?;
                new.name_format_with
                    .push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(default_variant = "..")]
            if meta.path.is_ident("default_variant") {
                meta.input.parse::<Token![=]>()?;
                let string = meta.input.parse::<syn::LitStr>()?;

                new.default_variant.push(match string.value().as_str() {
                    "index" => (meta.path.span(), DefaultTag::Index),
                    "name" => (meta.path.span(), DefaultTag::Name),
                    value => {
                        return Err(syn::Error::new_spanned(
                            string,
                            format_args!("#[{ATTR}(default_variant = {value:?})] Bad value, expected one of \"index\" or \"name\""),
                        ));
                    }
                });

                return Ok(());
            }

            // parse #[musli(default_field = "..")]
            if meta.path.is_ident("default_field") {
                meta.input.parse::<Token![=]>()?;
                let string = meta.input.parse::<syn::LitStr>()?;

                new.default_field.push(match string.value().as_str() {
                    "index" => (meta.path.span(), DefaultTag::Index),
                    "name" => (meta.path.span(), DefaultTag::Name),
                    value => {
                        return Err(syn::Error::new_spanned(
                            string,
                            format_args!("#[{ATTR}(default_field = {value:?})]: Bad value, expected one of \"index\" or \"name\""),
                        ));
                    }
                });

                return Ok(());
            }

            // parse #[musli(bound = {..})]
            if meta.path.is_ident("bound") {
                meta.input.parse::<Token![=]>()?;
                parse_bounds(&meta, &mut new.bounds)?;
                return Ok(());
            }

            // parse #[musli(decode_bound = {..})]
            if meta.path.is_ident("decode_bound") {
                meta.input.parse::<Token![=]>()?;
                parse_bounds(&meta, &mut new.decode_bounds)?;
                return Ok(());
            }

            // parse #[musli(packed)]
            if meta.path.is_ident("packed") {
                new.packing.push((meta.path.span(), Packing::Packed));
                return Ok(());
            }

            // parse #[musli(transparent)]
            if meta.path.is_ident("transparent") {
                new.packing.push((meta.path.span(), Packing::Transparent));
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
                cx.register_mode(mode.clone());
                attr.modes.entry(mode).or_default()
            }
            None => &mut attr.root,
        };

        attr.merge_with(cx, new, only);
    }

    attr
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
        /// `#[musli(packed)]` or `#[musli(transparent)]`.
        packing: Packing,
        /// `#[musli(default)]`.
        default_variant: (),
        /// `#[musli(default_field = "..")]`.
        default_field: DefaultTag,
        @multiple
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
        let mut mode = None::<syn::Path>;
        let mut only = None;

        let result = a.parse_nested_meta(|meta| {
            // parse #[musli(mode = <path>)]
            if meta.path.is_ident("mode") {
                meta.input.parse::<Token![=]>()?;
                mode = Some(meta.input.parse()?);
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

            // parse #[musli(name_type = <type>)]
            if meta.path.is_ident("name_type") {
                meta.input.parse::<Token![=]>()?;
                new.name_type.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(name_format_with = <path>)]
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

            // parse #[musli(name = <expr>)]
            if meta.path.is_ident("name") {
                meta.input.parse::<Token![=]>()?;
                new.name.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(default)]
            if meta.path.is_ident("default") {
                new.default_variant.push((meta.path.span(), ()));
                return Ok(());
            }

            // parse #[musli(default_field = "..")]
            if meta.path.is_ident("default_field") {
                meta.input.parse::<Token![=]>()?;
                let string = meta.input.parse::<syn::LitStr>()?;

                new.default_field.push(match string.value().as_str() {
                    "index" => (meta.path.span(), DefaultTag::Index),
                    "name" => (meta.path.span(), DefaultTag::Name),
                    value => {
                        return Err(syn::Error::new_spanned(
                            string,
                            format_args!("#[{ATTR}(default_field = {value:?})]: Bad value, expected one of \"index\" or \"name\"."),
                        ));
                    }
                });

                return Ok(());
            }

            // parse #[musli(packed)]
            if meta.path.is_ident("packed") {
                new.packing.push((meta.path.span(), Packing::Packed));
                return Ok(());
            }

            // parse #[musli(transparent)]
            if meta.path.is_ident("transparent") {
                new.packing.push((meta.path.span(), Packing::Transparent));
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
                cx.register_mode(mode.clone());
                attr.modes.entry(mode).or_default()
            }
            None => &mut attr.root,
        };

        attr.merge_with(cx, new, only);
    }

    attr
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
        /// Use a default value for the field if it's not available.
        is_default: Option<syn::Path>,
        /// Use a default value for the field if it's not available.
        skip: (),
        /// Use the alternate TraceDecode for the field.
        trace: (),
        /// Use the alternate EncodeBytes for the field.
        bytes: (),
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
            let trace = self.trace(mode).is_some();
            let bytes = self.bytes(mode).is_some();
            let encode_path = mode.encode_t_encode(trace, bytes);
            (span, encode_path)
        }
    }

    /// Expand decode of the given field.
    pub(crate) fn decode_path_expanded(&self, mode: Mode<'_>, span: Span) -> (Span, syn::Path) {
        let decode_path = self.decode_path(mode);

        if let Some((span, decode_path)) = decode_path {
            (*span, decode_path.clone())
        } else {
            let trace = self.trace(mode).is_some();
            let bytes = self.bytes(mode).is_some();
            let decode_path = mode.decode_t_decode(trace, bytes);
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
        let mut mode = None::<syn::Path>;
        let mut only = None;

        let result = a.parse_nested_meta(|meta| {
            // parse #[musli(mode = <path>)]
            if meta.path.is_ident("mode") {
                meta.input.parse::<Token![=]>()?;
                mode = Some(meta.input.parse()?);
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

            // parse parse #[musli(with = <path>)]
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

            // parse #[musli(skip_encoding_if = <path>)]
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

            // parse #[musli(name = <expr>)]
            if meta.path.is_ident("name") {
                meta.input.parse::<Token![=]>()?;
                new.name.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(default)]
            if meta.path.is_ident("default") {
                if meta.input.parse::<Option<Token![=]>>()?.is_some() {
                    new.is_default
                        .push((meta.path.span(), Some(meta.input.parse()?)));
                } else {
                    new.is_default.push((meta.path.span(), None));
                }

                return Ok(());
            }

            // parse #[musli(skip)]
            if meta.path.is_ident("skip") {
                new.skip.push((meta.path.span(), ()));
                return Ok(());
            }

            // parse #[musli(trace)]
            if meta.path.is_ident("trace") {
                new.trace.push((meta.path.span(), ()));
                return Ok(());
            }

            // parse #[musli(bytes)]
            if meta.path.is_ident("bytes") {
                new.bytes.push((meta.path.span(), ()));
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
                cx.register_mode(mode.clone());
                attr.modes.entry(mode).or_default()
            }
            None => &mut attr.root,
        };

        attr.merge_with(cx, new, only);
    }

    attr
}
