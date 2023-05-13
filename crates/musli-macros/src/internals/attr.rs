use std::collections::HashMap;
use std::fmt;
use std::mem;

use proc_macro2::Span;
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
    ($self:expr, $cx:expr, $new:expr, $field:ident) => {{
        for $field in $new.$field {
            if $self.$field.is_some() {
                $cx.error_span(
                    $field.0,
                    format_args!(
                        "#[{}] multiple {} attributes specified",
                        ATTR,
                        stringify!($field)
                    ),
                );
            } else {
                $self.$field = Some($field);
            }
        }
    }};
}

macro_rules! layer {
    ($new:ident, $layer:ident {
        $($(#[$($single_meta:meta)*])* $single:ident: $single_ty:ty,)* $(,)?
        @multiple
        $($(#[$($multiple_meta:meta)*])* $multiple:ident: $multiple_ty:ty,)* $(,)?
    }) => {
        #[derive(Default)]
        struct $new {
            $($(#[$($single_meta)*])* $single: Vec<(Span, $single_ty)>,)*
            $($(#[$($multiple_meta)*])* $multiple: Vec<(Span, $multiple_ty)>,)*
        }

        #[derive(Default)]
        struct $layer {
            $($(#[$($single_meta)*])* $single: Option<(Span, $single_ty)>,)*
            $($(#[$($multiple_meta)*])* $multiple: Vec<(Span, $multiple_ty)>,)*
        }

        impl $layer {
            /// Merge attributes.
            fn merge_with(&mut self, cx: &Ctxt, new: $new) {
                $(merge!(self, cx, new, $single);)*
                $(self.$multiple.extend(new.$multiple);)*
            }
        }
    }
}

layer! {
    TypeLayerNew, TypeLayer {
        /// `#[musli(crate = <path>)]`.
        krate: syn::Path,
        /// `#[musli(name_type)]`.
        name_type: syn::Type,
        /// `#[musli(default_variant_name = "..")]`.
        default_variant_name: DefaultTag,
        /// `#[musli(default_field_name = "..")]`.
        default_field_name: DefaultTag,
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

#[derive(Default)]
pub(crate) struct TypeAttr {
    root: TypeLayer,
    /// Nested configuartions for modes.
    modes: HashMap<syn::Path, TypeLayer>,
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
            .map(|(_, v)| v)
    }

    pub(crate) fn default_variant_name(&self, mode: Mode<'_>) -> Option<DefaultTag> {
        mode.ident
            .and_then(|m| Some(self.modes.get(m)?.default_variant_name))
            .unwrap_or(self.root.default_variant_name)
            .map(|(_, v)| v)
    }

    /// Get the tag type of the type.
    pub(crate) fn name_type(&self, mode: Mode<'_>) -> Option<&(Span, syn::Type)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.name_type.as_ref())
            .or(self.root.name_type.as_ref())
    }

    /// Get the configured crate, or fallback to default.
    pub(crate) fn crate_or_default(&self) -> syn::Path {
        if let Some((_, krate)) = &self.root.krate {
            krate.clone()
        } else {
            syn::Path::from(syn::Ident::new(&ATTR, Span::call_site()))
        }
    }

    /// Get the where clause that is associated with the type.
    pub(crate) fn bounds(&self, mode: Mode<'_>) -> &[(Span, syn::WherePredicate)] {
        mode.ident
            .and_then(|m| Some(&self.modes.get(m)?.bounds))
            .unwrap_or(&self.root.bounds)
    }

    /// Get bounds to require for a `Decode` implementation.
    pub(crate) fn decode_bounds(&self, mode: Mode<'_>) -> &[(Span, syn::WherePredicate)] {
        mode.ident
            .and_then(|m| Some(&self.modes.get(m)?.decode_bounds))
            .unwrap_or(&self.root.decode_bounds)
    }
}

pub(crate) fn type_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> TypeAttr {
    let mut attr = TypeAttr::default();

    for a in attrs {
        if a.path() != ATTR {
            continue;
        }

        let mut new = TypeLayerNew::default();
        let mut mode = None::<syn::Path>;

        let result = a.parse_nested_meta(|meta| {
            // parse #[musli(mode = <path>)]
            if meta.path == MODE {
                meta.input.parse::<Token![=]>()?;
                mode = Some(meta.input.parse()?);
                return Ok(());
            }

            // parse #[musli(tag = <expr>)]
            if meta.path == TAG {
                meta.input.parse::<Token![=]>()?;
                new.tag.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(content = <expr>)]
            if meta.path == CONTENT {
                meta.input.parse::<Token![=]>()?;
                new.content.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(crate = <path>)]
            if meta.path == CRATE {
                meta.input.parse::<Token![=]>()?;
                new.krate.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(name_type = <type>)]
            if meta.path == NAME_TYPE {
                meta.input.parse::<Token![=]>()?;
                new.name_type.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(default_variant_name = "..")]
            if meta.path == DEFAULT_VARIANT_NAME {
                meta.input.parse::<Token![=]>()?;
                let string = meta.input.parse::<syn::LitStr>()?;

                new.default_variant_name
                    .push(match string.value().as_str() {
                        "index" => (meta.path.span(), DefaultTag::Index),
                        "name" => (meta.path.span(), DefaultTag::Name),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                string,
                                format_args!("#[{ATTR}({DEFAULT_VARIANT_NAME})] Bad value"),
                            ));
                        }
                    });

                return Ok(());
            }

            // parse #[musli(default_field_name = "..")]
            if meta.path == DEFAULT_FIELD_NAME {
                meta.input.parse::<Token![=]>()?;
                let string = meta.input.parse::<syn::LitStr>()?;

                new.default_field_name.push(match string.value().as_str() {
                    "index" => (meta.path.span(), DefaultTag::Index),
                    "name" => (meta.path.span(), DefaultTag::Name),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            string,
                            format_args!("#[{ATTR}({DEFAULT_FIELD_NAME})]: Bad value."),
                        ));
                    }
                });

                return Ok(());
            }

            // parse #[musli(bound = {..})]
            if meta.path == BOUND {
                meta.input.parse::<Token![=]>()?;
                parse_bounds(&meta, &mut new.bounds)?;
                return Ok(());
            }

            // parse #[musli(decode_bound = {..})]
            if meta.path == DECODE_BOUND {
                meta.input.parse::<Token![=]>()?;
                parse_bounds(&meta, &mut new.decode_bounds)?;
                return Ok(());
            }

            // parse #[musli(packed)]
            if meta.path == PACKED {
                new.packing.push((meta.path.span(), Packing::Packed));
                return Ok(());
            }

            // parse #[musli(transparent)]
            if meta.path == TRANSPARENT {
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

        attr.merge_with(cx, new);
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
    VariantLayerNew, VariantLayer {
        /// `#[musli(name_type)]`.
        name_type: syn::Type,
        /// Rename a field to the given expression.
        rename: syn::Expr,
        /// `#[musli(packed)]` or `#[musli(transparent)]`.
        packing: Packing,
        /// `#[musli(default)]`.
        default: (),
        /// `#[musli(default_field_name = "..")]`.
        default_field_name: DefaultTag,
        @multiple
    }
}

#[derive(Default)]
pub(crate) struct VariantAttr {
    root: VariantLayer,
    modes: HashMap<syn::Path, VariantLayer>,
}

impl VariantAttr {
    /// Test if the `#[musli(default)]` tag is specified.
    pub(crate) fn default_attr(&self, mode: Mode<'_>) -> Option<Span> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.default)
            .or(self.root.default)
            .map(|(s, ())| s)
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
            .map(|(_, v)| v)
    }

    /// Get the tag type of the type.
    pub(crate) fn name_type(&self, mode: Mode<'_>) -> Option<&(Span, syn::Type)> {
        mode.ident
            .and_then(|m| self.modes.get(m))
            .map(|a| a.name_type.as_ref())
            .unwrap_or(self.root.name_type.as_ref())
    }
}

/// Parse variant attributes.
pub(crate) fn variant_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> VariantAttr {
    let mut attr = VariantAttr::default();

    for a in attrs {
        if a.path() != ATTR {
            continue;
        }

        let mut new = VariantLayerNew::default();
        let mut mode = None::<syn::Path>;

        let result = a.parse_nested_meta(|meta| {
            // parse #[musli(mode = <path>)]
            if meta.path == MODE {
                meta.input.parse::<Token![=]>()?;
                mode = Some(meta.input.parse()?);
                return Ok(());
            }

            // parse #[musli(name_type = <type>)]
            if meta.path == NAME_TYPE {
                meta.input.parse::<Token![=]>()?;
                new.name_type.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(rename = <expr>)]
            if meta.path == RENAME {
                meta.input.parse::<Token![=]>()?;
                new.rename.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(default)]
            if meta.path == DEFAULT {
                new.default.push((meta.path.span(), ()));
                return Ok(());
            }

            // parse #[musli(default_field_name = "..")]
            if meta.path == DEFAULT_FIELD_NAME {
                meta.input.parse::<Token![=]>()?;
                let string = meta.input.parse::<syn::LitStr>()?;

                new.default_field_name.push(match string.value().as_str() {
                    "index" => (meta.path.span(), DefaultTag::Index),
                    "name" => (meta.path.span(), DefaultTag::Name),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            string,
                            format_args!("#[{ATTR}({DEFAULT_FIELD_NAME})]: Bad value."),
                        ));
                    }
                });

                return Ok(());
            }

            // parse #[musli(packed)]
            if meta.path == PACKED {
                new.packing.push((meta.path.span(), Packing::Packed));
                return Ok(());
            }

            // parse #[musli(transparent)]
            if meta.path == TRANSPARENT {
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

        attr.merge_with(cx, new);
    }

    attr
}

layer! {
    FieldNew, FieldLayer {
        /// Module to use when decoding.
        encode_path: syn::Path,
        /// Path to use when decoding.
        decode_path: syn::Path,
        /// Method to check if we want to skip encoding.
        skip_encoding_if: syn::Path,
        /// Rename a field to the given literal.
        rename: syn::Expr,
        /// Use a default value for the field if it's not available.
        default: (),
        @multiple
    }
}

#[derive(Default)]
pub(crate) struct Field {
    root: FieldLayer,
    modes: HashMap<syn::Path, FieldLayer>,
}

impl Field {
    /// Test if the `#[musli(default)]` tag is specified.
    pub(crate) fn default_attr(&self, mode: Mode<'_>) -> Option<Span> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.default)
            .or(self.root.default)
            .map(|(span, ())| span)
    }

    /// Test if the `#[musli(rename)]` tag is specified.
    pub(crate) fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        mode.ident
            .and_then(|m| self.modes.get(m)?.rename.as_ref())
            .or(self.root.rename.as_ref())
    }

    /// Expand encode of the given field.
    pub(crate) fn encode_path(&self, mode: Mode<'_>, span: Span) -> (Span, syn::Path) {
        let encode_path = mode
            .ident
            .and_then(|m| self.modes.get(m)?.encode_path.as_ref())
            .or(self.root.encode_path.as_ref());

        if let Some((span, encode_path)) = encode_path {
            let mut encode_path = encode_path.clone();
            let mode_ident = mode.mode_ident();

            if let Some(last) = encode_path.segments.last_mut() {
                adjust_mode_path(last, mode_ident);
            }

            (*span, encode_path)
        } else {
            let encode_path = mode.encode_t_encode(span);
            (span, encode_path)
        }
    }

    /// Expand decode of the given field.
    pub(crate) fn decode_path(&self, mode: Mode<'_>, span: Span) -> (Span, syn::Path) {
        let decode_path = mode
            .ident
            .and_then(|m| self.modes.get(m)?.decode_path.as_ref())
            .or(self.root.decode_path.as_ref());

        if let Some((span, decode_path)) = decode_path {
            let mut decode_path = decode_path.clone();
            let mode_ident = mode.mode_ident();

            if let Some(last) = decode_path.segments.last_mut() {
                adjust_mode_path(last, mode_ident);
            }

            (*span, decode_path)
        } else {
            let decode_path = mode.decode_t_decode(span);
            (span, decode_path)
        }
    }

    /// Get skip encoding if.
    pub(crate) fn skip_encoding_if(&self, mode: Mode<'_>) -> Option<(Span, &syn::Path)> {
        let (span, path) = mode
            .ident
            .and_then(|m| self.modes.get(m)?.skip_encoding_if.as_ref())
            .or(self.root.skip_encoding_if.as_ref())?;

        Some((*span, path))
    }
}

/// Parse field attributes.
pub(crate) fn field_attrs(cx: &Ctxt, attrs: &[syn::Attribute]) -> Field {
    let mut attr = Field::default();

    for a in attrs {
        if a.path() != ATTR {
            continue;
        }

        let mut new = FieldNew::default();
        let mut mode = None::<syn::Path>;

        let result = a.parse_nested_meta(|meta| {
            // parse #[musli(mode = <path>)]
            if meta.path == MODE {
                meta.input.parse::<Token![=]>()?;
                mode = Some(meta.input.parse()?);
                return Ok(());
            }

            // parse parse #[musli(with = <path>)]
            if meta.path == WITH {
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
            if meta.path == SKIP_ENCODING_IF {
                meta.input.parse::<Token![=]>()?;
                new.skip_encoding_if
                    .push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(rename = <expr>)]
            if meta.path == RENAME {
                meta.input.parse::<Token![=]>()?;
                new.rename.push((meta.path.span(), meta.input.parse()?));
                return Ok(());
            }

            // parse #[musli(default)]
            if meta.path == DEFAULT {
                new.default.push((meta.path.span(), ()));
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

        attr.merge_with(cx, new);
    }

    attr
}

/// Adjust a mode path.
fn adjust_mode_path(last: &mut syn::PathSegment, mode_ident: ModePath) {
    let insert_args = |args: &mut Punctuated<_, _>| {
        args.insert(
            0,
            syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                qself: None,
                path: mode_ident.as_path(),
            })),
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
            args.inputs.insert(
                0,
                syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: mode_ident.as_path(),
                }),
            );

            args.inputs.insert(
                1,
                syn::Type::Infer(syn::TypeInfer {
                    underscore_token: <Token![_]>::default(),
                }),
            );
        }
    }
}
