use std::rc::Rc;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::de::{build_call, build_reference};
use crate::expander::{
    self, Data, EnumData, Expander, FieldData, NameMethod, StructData, UnsizedMethod, VariantData,
};

use super::attr::{EnumTagging, Packing};
use super::name::NameAll;
use super::tokens::Tokens;
use super::ATTR;
use super::{Ctxt, Expansion, Mode, Only, Result};

pub(crate) struct Build<'a> {
    pub(crate) input: &'a syn::DeriveInput,
    pub(crate) cx: &'a Ctxt,
    pub(crate) tokens: &'a Tokens,
    pub(crate) bounds: &'a [(Span, syn::WherePredicate)],
    pub(crate) decode_bounds: &'a [(Span, syn::WherePredicate)],
    pub(crate) expansion: Expansion<'a>,
    pub(crate) data: BuildData<'a>,
    pub(crate) decode_t_decode: syn::Path,
    #[allow(unused)]
    pub(crate) decode_t_visit: syn::Path,
    pub(crate) encode_t_encode: syn::Path,
    pub(crate) enum_tagging_span: Option<Span>,
}

impl Build<'_> {
    /// Emit diagnostics for when we try to implement `Decode` for an enum which
    /// is marked as `#[musli(transparent)]`.
    pub(crate) fn encode_transparent_enum_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format_args!("#[{ATTR}(transparent)] cannot be used to encode enums",),
        );
    }

    /// Emit diagnostics indicating that we tried to implement decode for a
    /// packed enum.
    pub(crate) fn decode_packed_enum_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format_args!("#[{ATTR}(packed)] cannot be used to decode enums"),
        );
    }

    /// Emit diagnostics indicating that we tried to use a `#[musli(default)]`
    /// annotation on a packed container.
    pub(crate) fn packed_default_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format_args!("#[{ATTR}(default)] fields cannot be used in an packed container",),
        );
    }

    /// Emit diagnostics for a transparent encode / decode that failed because
    /// the wrong number of fields existed.
    pub(crate) fn transparent_diagnostics(&self, span: Span, fields: &[Rc<Field>]) {
        if fields.is_empty() {
            self.cx.error_span(
                span,
                format_args!("#[{ATTR}(transparent)] types must have a single unskipped field"),
            );
        } else {
            self.cx.error_span(
                span,
                format_args!(
                    "#[{ATTR}(transparent)] can only be used on types which have a single field",
                ),
            );
        }
    }

    /// Validate encode attributes.
    pub(crate) fn validate_encode(&self) -> Result<()> {
        self.validate()
    }

    /// Validate set of legal attributes.
    pub(crate) fn validate_decode(&self) -> Result<()> {
        self.validate()
    }

    fn validate(&self) -> Result<()> {
        match &self.data {
            BuildData::Struct(..) => {
                if let Some(span) = self.enum_tagging_span {
                    self.cx.error_span(
                        span,
                        format_args!(
                            "#[{ATTR}(tag)] and #[{ATTR}(content)] are only supported on enums"
                        ),
                    );

                    return Err(());
                }
            }
            BuildData::Enum(..) => (),
        }

        Ok(())
    }
}

/// Build model for enums and structs.
pub(crate) enum BuildData<'a> {
    Struct(Body<'a>),
    Enum(Enum<'a>),
}

pub(crate) struct Body<'a> {
    pub(crate) span: Span,
    pub(crate) name: &'a syn::LitStr,
    pub(crate) unskipped_fields: Vec<Rc<Field<'a>>>,
    pub(crate) all_fields: Vec<Rc<Field<'a>>>,
    pub(crate) name_type: syn::Type,
    pub(crate) name_method: NameMethod,
    pub(crate) name_format_with: Option<&'a (Span, syn::Path)>,
    pub(crate) packing: Packing,
    pub(crate) path: syn::Path,
}

impl Body<'_> {
    pub(crate) fn name_format(&self, value: &syn::Expr) -> syn::Expr {
        match self.name_format_with {
            Some((_, path)) => build_call(path, [build_reference(value.clone())]),
            None => build_reference(value.clone()),
        }
    }

    pub(crate) fn name_local_type(&self) -> syn::Type {
        match self.name_method {
            NameMethod::Unsized(..) => syn::Type::Reference(syn::TypeReference {
                and_token: <Token![&]>::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(self.name_type.clone()),
            }),
            NameMethod::Value => self.name_type.clone(),
        }
    }
}

pub(crate) struct Enum<'a> {
    pub(crate) span: Span,
    pub(crate) name: &'a syn::LitStr,
    pub(crate) enum_tagging: Option<EnumTagging<'a>>,
    pub(crate) enum_packing: Packing,
    pub(crate) variants: Vec<Variant<'a>>,
    pub(crate) fallback: Option<&'a syn::Ident>,
    pub(crate) name_type: syn::Type,
    pub(crate) name_method: NameMethod,
    pub(crate) name_format_with: Option<&'a (Span, syn::Path)>,
    pub(crate) packing_span: Option<&'a (Span, Packing)>,
}

impl Enum<'_> {
    pub(crate) fn name_format(&self, value: &syn::Expr) -> syn::Expr {
        match self.name_format_with {
            Some((_, path)) => build_call(path, [build_reference(value.clone())]),
            None => build_reference(value.clone()),
        }
    }

    pub(crate) fn name_local_type(&self) -> syn::Type {
        match self.name_method {
            NameMethod::Unsized(..) => syn::Type::Reference(syn::TypeReference {
                and_token: <Token![&]>::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(self.name_type.clone()),
            }),
            NameMethod::Value => self.name_type.clone(),
        }
    }
}

pub(crate) struct Variant<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) name: syn::Expr,
    pub(crate) pattern: Option<&'a syn::Pat>,
    pub(crate) st: Body<'a>,
    pub(crate) patterns: Punctuated<syn::FieldPat, Token![,]>,
}

pub(crate) struct Field<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) encode_path: (Span, syn::Path),
    pub(crate) decode_path: (Span, syn::Path),
    pub(crate) name: syn::Expr,
    pub(crate) pattern: Option<&'a syn::Pat>,
    /// Skip field entirely and always initialize with the specified expresion,
    /// or default value through `default_attr`.
    pub(crate) skip: Option<Span>,
    pub(crate) skip_encoding_if: Option<&'a (Span, syn::Path)>,
    /// Fill with default value, if missing.
    pub(crate) default_attr: Option<(Span, Option<&'a syn::Path>)>,
    pub(crate) self_access: syn::Expr,
    pub(crate) member: syn::Member,
    pub(crate) packing: Packing,
    pub(crate) var: syn::Ident,
    pub(crate) ty: &'a syn::Type,
}

/// Setup a build.
///
/// Handles mode decoding, and construction of parameters which might give rise to errors.
pub(crate) fn setup<'a>(
    e: &'a Expander,
    expansion: Expansion<'a>,
    only: Only,
) -> Result<Build<'a>> {
    let mode = expansion.as_mode(&e.tokens, only);

    let data = match &e.data {
        Data::Struct(data) => BuildData::Struct(setup_struct(e, mode, data)?),
        Data::Enum(data) => BuildData::Enum(setup_enum(e, mode, data)?),
        Data::Union => {
            e.cx.error_span(e.input.ident.span(), "musli: not supported for unions");
            return Err(());
        }
    };

    Ok(Build {
        input: e.input,
        cx: &e.cx,
        tokens: &e.tokens,
        bounds: e.type_attr.bounds(mode),
        decode_bounds: e.type_attr.decode_bounds(mode),
        expansion,
        data,
        decode_t_decode: mode.decode_t_decode(false, false),
        decode_t_visit: mode.decode_t_visit(),
        encode_t_encode: mode.encode_t_encode(false, false),
        enum_tagging_span: e.type_attr.enum_tagging_span(mode),
    })
}

fn setup_struct<'a>(e: &'a Expander, mode: Mode<'_>, data: &'a StructData<'a>) -> Result<Body<'a>> {
    let mut unskipped_fields = Vec::with_capacity(data.fields.len());
    let mut all_fields = Vec::with_capacity(data.fields.len());

    let packing = e
        .type_attr
        .packing(mode)
        .map(|&(_, p)| p)
        .unwrap_or_default();

    let (name_all, name_type, name_method) = split_name(
        e.type_attr.name_type(mode),
        e.type_attr.name_all(mode),
        e.type_attr.name_method(mode),
    );

    let path = syn::Path::from(syn::Ident::new("Self", e.input.ident.span()));

    for f in &data.fields {
        let field = Rc::new(setup_field(e, mode, f, name_all, packing, None)?);

        if field.skip.is_none() {
            unskipped_fields.push(field.clone());
        }

        all_fields.push(field);
    }

    Ok(Body {
        span: data.span,
        name: &data.name,
        unskipped_fields,
        all_fields,
        name_type,
        name_method,
        name_format_with: e.type_attr.name_format_with(mode),
        packing,
        path,
    })
}

fn setup_enum<'a>(e: &'a Expander, mode: Mode<'_>, data: &'a EnumData<'a>) -> Result<Enum<'a>> {
    let mut variants = Vec::with_capacity(data.variants.len());
    let mut fallback = None;

    let enum_tagging = e.type_attr.enum_tagging(mode);

    let packing_span = e.type_attr.packing(mode);

    let enum_packing = e
        .type_attr
        .packing(mode)
        .map(|&(_, p)| p)
        .unwrap_or_default();

    let (_, name_type, name_method) = split_name(
        e.type_attr.name_type(mode),
        e.type_attr.name_all(mode),
        e.type_attr.name_method(mode),
    );

    if enum_tagging.is_some() {
        match packing_span {
            Some((_, Packing::Tagged)) => (),
            Some(&(span, packing)) => {
                e.cx.error_span(span, format_args!("#[{ATTR}({packing})] cannot be combined with #[{ATTR}(tag)] or #[{ATTR}(content)]"));
                return Err(());
            }
            _ => (),
        }
    }

    for v in &data.variants {
        variants.push(setup_variant(e, mode, v, &mut fallback)?);
    }

    Ok(Enum {
        span: data.span,
        name: &data.name,
        enum_tagging,
        enum_packing,
        variants,
        fallback,
        name_type,
        name_method,
        name_format_with: e.type_attr.name_format_with(mode),
        packing_span,
    })
}

fn setup_variant<'a>(
    e: &'a Expander<'_>,
    mode: Mode<'_>,
    data: &'a VariantData<'a>,
    fallback: &mut Option<&'a syn::Ident>,
) -> Result<Variant<'a>> {
    let mut unskipped_fields = Vec::with_capacity(data.fields.len());
    let mut all_fields = Vec::with_capacity(data.fields.len());

    let variant_packing = data
        .attr
        .packing(mode)
        .or_else(|| e.type_attr.packing(mode))
        .map(|&(_, v)| v)
        .unwrap_or_default();

    let (name_all, name_type, name_method) = split_name(
        data.attr.name_type(mode),
        data.attr.name_all(mode),
        data.attr.name_method(mode),
    );

    let name = expander::expand_name(
        data,
        mode,
        e.type_attr
            .name_all(mode)
            .map(|&(_, v)| v)
            .unwrap_or_default(),
        Some(data.ident),
    )?;

    let pattern = data.attr.pattern(mode).map(|(_, p)| p);

    let mut path = syn::Path::from(syn::Ident::new("Self", data.span));
    path.segments.push(data.ident.clone().into());

    if let Some((span, _)) = data.attr.default_variant(mode) {
        if !data.fields.is_empty() {
            e.cx.error_span(
                *span,
                format_args!("#[{ATTR}(default)] variant must be empty"),
            );
        } else if fallback.is_some() {
            e.cx.error_span(
                *span,
                format_args!("#[{ATTR}(default)] only one fallback variant is supported",),
            );
        } else {
            *fallback = Some(data.ident);
        }
    }

    let mut patterns = Punctuated::default();

    for f in &data.fields {
        let field = Rc::new(setup_field(
            e,
            mode,
            f,
            name_all,
            variant_packing,
            Some(&mut patterns),
        )?);

        if field.skip.is_none() {
            unskipped_fields.push(field.clone());
        }

        all_fields.push(field);
    }

    Ok(Variant {
        span: data.span,
        index: data.index,
        name,
        pattern,
        patterns,
        st: Body {
            span: data.span,
            name: &data.name,
            unskipped_fields,
            all_fields,
            packing: variant_packing,
            name_type,
            name_method,
            name_format_with: data.attr.name_format_with(mode),
            path,
        },
    })
}

fn setup_field<'a>(
    e: &'a Expander,
    mode: Mode<'_>,
    data: &'a FieldData<'a>,
    name_all: NameAll,
    packing: Packing,
    patterns: Option<&mut Punctuated<syn::FieldPat, Token![,]>>,
) -> Result<Field<'a>> {
    let encode_path = data.attr.encode_path_expanded(mode, data.span);
    let decode_path = data.attr.decode_path_expanded(mode, data.span);

    let name = expander::expand_name(data, mode, name_all, data.ident)?;
    let pattern = data.attr.pattern(mode).map(|(_, p)| p);

    let skip = data.attr.skip(mode).map(|&(s, ())| s);
    let skip_encoding_if = data.attr.skip_encoding_if(mode);
    let default_attr = data
        .attr
        .is_default(mode)
        .map(|(s, path)| (*s, path.as_ref()));

    let member = match data.ident {
        Some(ident) => syn::Member::Named(ident.clone()),
        None => syn::Member::Unnamed(syn::Index {
            index: data.index as u32,
            span: data.span,
        }),
    };

    let self_access = if let Some(patterns) = patterns {
        match data.ident {
            Some(ident) => {
                patterns.push(syn::FieldPat {
                    attrs: Vec::new(),
                    member: syn::Member::Named(ident.clone()),
                    colon_token: None,
                    pat: Box::new(syn::Pat::Path(syn::PatPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(ident.clone()),
                    })),
                });

                syn::Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: ident.clone().into(),
                })
            }
            None => {
                let var = quote::format_ident!("v{}", data.index);

                patterns.push(syn::FieldPat {
                    attrs: Vec::new(),
                    member: syn::Member::Unnamed(syn::Index::from(data.index)),
                    colon_token: Some(<Token![:]>::default()),
                    pat: Box::new(syn::Pat::Path(syn::PatPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(var.clone()),
                    })),
                });

                syn::Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: var.into(),
                })
            }
        }
    } else {
        let expr = syn::Expr::Field(syn::ExprField {
            attrs: Vec::new(),
            base: Box::new(syn::Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: <Token![self]>::default().into(),
            })),
            dot_token: <Token![.]>::default(),
            member: member.clone(),
        });

        syn::Expr::Reference(syn::ExprReference {
            attrs: Vec::new(),
            and_token: <Token![&]>::default(),
            mutability: None,
            expr: Box::new(expr),
        })
    };

    let var = match &member {
        syn::Member::Named(ident) => e.cx.ident_with_span(&ident.to_string(), ident.span(), "_f"),
        syn::Member::Unnamed(index) => {
            e.cx.ident_with_span(&index.index.to_string(), index.span, "_f")
        }
    };

    Ok(Field {
        span: data.span,
        index: data.index,
        encode_path,
        decode_path,
        name,
        pattern,
        skip,
        skip_encoding_if,
        default_attr,
        self_access,
        member,
        packing,
        var,
        ty: data.ty,
    })
}

fn split_name(
    name_type: Option<&(Span, syn::Type)>,
    name_all: Option<&(Span, NameAll)>,
    name_method: Option<&(Span, NameMethod)>,
) -> (NameAll, syn::Type, NameMethod) {
    let name_all = name_all.map(|&(_, v)| v);
    let name_method = name_method.map(|&(_, v)| v);

    let Some((_, name_type)) = name_type else {
        let name_all = name_all.unwrap_or_default();
        let name_method = name_method.unwrap_or_else(|| name_all.name_method());
        return (name_all, name_all.ty(), name_method);
    };

    let (name_method, default_name_all) = match name_method {
        Some(name_method) => (name_method, name_method.name_all()),
        None => determine_name_method(name_type),
    };

    let name_all = name_all.or(default_name_all).unwrap_or_default();
    (name_all, name_type.clone(), name_method)
}

fn determine_name_method(ty: &syn::Type) -> (NameMethod, Option<NameAll>) {
    match ty {
        syn::Type::Path(syn::TypePath { qself: None, path }) if path.is_ident("str") => {
            return (
                NameMethod::Unsized(UnsizedMethod::Default),
                Some(NameAll::Name),
            );
        }
        syn::Type::Slice(syn::TypeSlice { elem, .. }) => match &**elem {
            syn::Type::Path(syn::TypePath { qself: None, path }) if path.is_ident("u8") => {
                return (
                    NameMethod::Unsized(UnsizedMethod::Bytes),
                    Some(NameAll::Name),
                );
            }
            _ => {}
        },
        _ => {}
    }

    (NameMethod::Value, None)
}
