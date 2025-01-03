use std::rc::Rc;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::{
    self, Data, EnumData, Expander, FieldData, NameMethod, NameType, StructData, StructKind,
    UnsizedMethod, VariantData,
};

use super::attr::{DefaultOrCustom, EnumTagging, FieldEncoding, ModeKind, Packing};
use super::mode::ImportedMethod;
use super::name::NameAll;
use super::ATTR;
use super::{Ctxt, Expansion, Mode, Result, Tokens};

pub(crate) struct Parameters {
    pub(crate) lt: syn::Lifetime,
    pub(crate) lt_exists: bool,
    pub(crate) allocator_ident: syn::Ident,
    pub(crate) allocator_exists: bool,
}

pub(crate) struct Build<'tok, 'a> {
    pub(crate) mode: Mode<'a>,
    pub(crate) input: &'a syn::DeriveInput,
    pub(crate) cx: &'a Ctxt,
    pub(crate) bounds: &'a [(Span, syn::WherePredicate)],
    pub(crate) decode_bounds: &'a [(Span, syn::WherePredicate)],
    pub(crate) expansion: Expansion<'a>,
    pub(crate) data: BuildData<'a>,
    pub(crate) decode_t_decode: ImportedMethod<'a>,
    pub(crate) encode_t_encode: ImportedMethod<'a>,
    pub(crate) enum_tagging_span: Option<Span>,
    pub(crate) tokens: &'tok Tokens<'a>,
    pub(crate) p: Parameters,
}

impl Build<'_, '_> {
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
            format_args!(
                "#[{ATTR}(default)] fields can only be used in the end of packed containers"
            ),
        );
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
    pub(crate) name_type: NameType<'a>,
    pub(crate) packing: Packing,
    pub(crate) kind: StructKind,
    pub(crate) path: syn::Path,
}

impl Body<'_> {
    pub(crate) fn validate(&self, cx: &Ctxt) {
        if self.packing == Packing::Transparent && !matches!(&self.unskipped_fields[..], [_]) {
            cx.transparent_diagnostics(self.span, &self.unskipped_fields);
        }
    }
}

pub(crate) struct Enum<'a> {
    pub(crate) span: Span,
    pub(crate) name: &'a syn::LitStr,
    pub(crate) enum_tagging: EnumTagging<'a>,
    pub(crate) enum_packing: Packing,
    pub(crate) variants: Vec<Variant<'a>>,
    pub(crate) fallback: Option<&'a syn::Ident>,
    pub(crate) name_type: NameType<'a>,
    pub(crate) packing_span: Option<&'a (Span, Packing)>,
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
    pub(crate) encode_path: (Span, DefaultOrCustom<'a>),
    pub(crate) decode_path: (Span, DefaultOrCustom<'a>),
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
pub(crate) fn setup<'tok, 'a>(
    e: &'a Expander,
    expansion: Expansion<'a>,
    mode: Mode<'a>,
    tokens: &'tok Tokens<'a>,
    p: Parameters,
) -> Result<Build<'tok, 'a>> {
    let data = match &e.data {
        Data::Struct(data) => BuildData::Struct(setup_struct(e, &mode, data, &p.allocator_ident)),
        Data::Enum(data) => BuildData::Enum(setup_enum(e, &mode, data, &p.allocator_ident)),
        Data::Union => {
            e.cx.error_span(e.input.ident.span(), "musli: not supported for unions");
            return Err(());
        }
    };

    if e.cx.has_errors() {
        return Err(());
    }

    let decode_t_decode = mode.decode_t_decode(FieldEncoding::Default, &p.allocator_ident);
    let encode_t_encode = mode.encode_t_encode(FieldEncoding::Default);

    let bounds = e.type_attr.bounds(&mode);
    let decode_bounds = e.type_attr.decode_bounds(&mode);
    let enum_tagging_span = e.type_attr.enum_tagging_span(&mode);

    Ok(Build {
        mode,
        input: e.input,
        cx: &e.cx,
        bounds,
        decode_bounds,
        expansion,
        data,
        decode_t_decode,
        encode_t_encode,
        enum_tagging_span,
        tokens,
        p,
    })
}

fn setup_struct<'a>(
    e: &'a Expander,
    mode: &Mode<'a>,
    data: &'a StructData<'a>,
    allocator_ident: &syn::Ident,
) -> Body<'a> {
    let mut unskipped_fields = Vec::with_capacity(data.fields.len());
    let mut all_fields = Vec::with_capacity(data.fields.len());

    let packing = e
        .type_attr
        .packing(mode)
        .map(|&(_, p)| p)
        .unwrap_or_default();

    let (name_all, name_type, name_method) = match data.kind {
        StructKind::Indexed(..) if e.type_attr.is_name_type_ambiguous(mode) => {
            let name_all = NameAll::Index;
            (name_all, name_all.ty(), NameMethod::Sized)
        }
        _ => split_name(
            mode.kind,
            e.type_attr.name_type(mode),
            e.type_attr.name_all(mode),
            e.type_attr.name_method(mode),
        ),
    };

    let path = syn::Path::from(syn::Ident::new("Self", e.input.ident.span()));

    for f in &data.fields {
        let field = Rc::new(setup_field(
            e,
            mode,
            f,
            name_all,
            packing,
            None,
            allocator_ident,
        ));

        if field.skip.is_none() {
            unskipped_fields.push(field.clone());
        }

        all_fields.push(field);
    }

    let body = Body {
        span: data.span,
        name: &data.name,
        unskipped_fields,
        all_fields,
        name_type: NameType {
            ty: name_type,
            method: name_method,
            format_with: e.type_attr.name_format_with(mode),
        },
        packing,
        kind: data.kind,
        path,
    };

    body.validate(&e.cx);
    body
}

fn setup_enum<'a>(
    e: &'a Expander,
    mode: &Mode<'a>,
    data: &'a EnumData<'a>,
    allocator_ident: &syn::Ident,
) -> Enum<'a> {
    let mut variants = Vec::with_capacity(data.variants.len());
    let mut fallback = None;

    let packing_span = e.type_attr.packing(mode);

    let enum_tagging = match e.type_attr.enum_tagging(mode) {
        Some(enum_tagging) => enum_tagging,
        None => {
            if data
                .variants
                .iter()
                .all(|v| matches!(v.kind, StructKind::Indexed(0) | StructKind::Empty))
            {
                EnumTagging::Empty
            } else {
                EnumTagging::Default
            }
        }
    };

    if !matches!(enum_tagging, EnumTagging::Default | EnumTagging::Empty) {
        match packing_span {
            Some((_, Packing::Tagged)) => (),
            Some(&(span, Packing::Packed)) => {
                e.cx.error_span(span, format_args!("#[{ATTR}(packed)] cannot be combined with #[{ATTR}(tag)] or #[{ATTR}(content)]"));
            }
            Some(&(span, Packing::Transparent)) => {
                e.cx.error_span(span, format_args!("#[{ATTR}(transparent)] cannot be combined with #[{ATTR}(tag)] or #[{ATTR}(content)]"));
            }
            _ => (),
        }
    }

    let enum_packing = e
        .type_attr
        .packing(mode)
        .map(|&(_, p)| p)
        .unwrap_or_default();

    let (_, name_type, name_method) = split_name(
        mode.kind,
        e.type_attr.name_type(mode),
        e.type_attr.name_all(mode),
        e.type_attr.name_method(mode),
    );

    for v in &data.variants {
        variants.push(setup_variant(e, mode, v, &mut fallback, allocator_ident));
    }

    Enum {
        span: data.span,
        name: &data.name,
        enum_tagging,
        enum_packing,
        variants,
        fallback,
        name_type: NameType {
            ty: name_type,
            method: name_method,
            format_with: e.type_attr.name_format_with(mode),
        },
        packing_span,
    }
}

fn setup_variant<'a>(
    e: &'a Expander<'_>,
    mode: &Mode<'a>,
    data: &'a VariantData<'a>,
    fallback: &mut Option<&'a syn::Ident>,
    allocator_ident: &syn::Ident,
) -> Variant<'a> {
    let mut unskipped_fields = Vec::with_capacity(data.fields.len());
    let mut all_fields = Vec::with_capacity(data.fields.len());

    let variant_packing = data
        .attr
        .packing(mode)
        .or_else(|| e.type_attr.packing(mode))
        .map(|&(_, v)| v)
        .unwrap_or_default();

    let (name_all, name_type, name_method) = match data.kind {
        StructKind::Indexed(..) if data.attr.is_name_type_ambiguous(mode) => {
            let name_all = NameAll::Index;
            (name_all, name_all.ty(), NameMethod::Sized)
        }
        _ => split_name(
            mode.kind,
            data.attr.name_type(mode),
            data.attr.name_all(mode),
            data.attr.name_method(mode),
        ),
    };

    let (type_name_all, _, _) = split_name(
        mode.kind,
        e.type_attr.name_type(mode),
        e.type_attr.name_all(mode),
        e.type_attr.name_method(mode),
    );

    let name = expander::expand_name(data, mode, type_name_all, Some(data.ident));

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
            allocator_ident,
        ));

        if field.skip.is_none() {
            unskipped_fields.push(field.clone());
        }

        all_fields.push(field);
    }

    let st = Body {
        span: data.span,
        name: &data.name,
        unskipped_fields,
        all_fields,
        packing: variant_packing,
        kind: data.kind,
        name_type: NameType {
            ty: name_type,
            method: name_method,
            format_with: data.attr.name_format_with(mode),
        },
        path,
    };

    st.validate(&e.cx);

    Variant {
        span: data.span,
        index: data.index,
        name,
        pattern,
        patterns,
        st,
    }
}

fn setup_field<'a>(
    e: &'a Expander,
    mode: &Mode<'a>,
    data: &'a FieldData<'a>,
    name_all: NameAll,
    packing: Packing,
    patterns: Option<&mut Punctuated<syn::FieldPat, Token![,]>>,
    allocator_ident: &syn::Ident,
) -> Field<'a> {
    let encode_path = data.attr.encode_path_expanded(mode, data.span);
    let decode_path = data
        .attr
        .decode_path_expanded(mode, data.span, allocator_ident);

    let name = expander::expand_name(data, mode, name_all, data.ident);
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
                let colon_token;
                let pat;

                let expr = if skip.is_none() {
                    colon_token = None;

                    pat = syn::Pat::Path(syn::PatPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(ident.clone()),
                    });

                    syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: ident.clone().into(),
                    })
                } else {
                    colon_token = Some(<Token![:]>::default());

                    pat = syn::Pat::Wild(syn::PatWild {
                        attrs: Vec::new(),
                        underscore_token: <Token![_]>::default(),
                    });

                    syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(quote::format_ident!("skipped_{}", ident)),
                    })
                };

                patterns.push(syn::FieldPat {
                    attrs: Vec::new(),
                    member: syn::Member::Named(ident.clone()),
                    colon_token,
                    pat: Box::new(pat),
                });

                expr
            }
            None => {
                let pat;
                let expr;

                if skip.is_none() {
                    let var = quote::format_ident!("v{}", data.index);

                    pat = syn::Pat::Path(syn::PatPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(var.clone()),
                    });

                    expr = syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: var.into(),
                    });
                } else {
                    pat = syn::Pat::Wild(syn::PatWild {
                        attrs: Vec::new(),
                        underscore_token: <Token![_]>::default(),
                    });

                    expr = syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(quote::format_ident!("skipped_{}", data.index)),
                    });
                };

                patterns.push(syn::FieldPat {
                    attrs: Vec::new(),
                    member: syn::Member::Unnamed(syn::Index::from(data.index)),
                    colon_token: Some(<Token![:]>::default()),
                    pat: Box::new(pat),
                });

                expr
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

    Field {
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
    }
}

pub(crate) fn split_name(
    kind: Option<&ModeKind>,
    ty: Option<&(Span, syn::Type)>,
    all: Option<&(Span, NameAll)>,
    method: Option<&(Span, NameMethod)>,
) -> (NameAll, syn::Type, NameMethod) {
    let kind_name_all = kind.and_then(ModeKind::default_name_all);

    let all = all.map(|&(_, v)| v);
    let method = method.map(|&(_, v)| v);

    let Some((_, ty)) = ty else {
        let all = all.or(kind_name_all).unwrap_or_default();
        let method = method.unwrap_or_else(|| all.name_method());
        return (all, all.ty(), method);
    };

    let (method, default_all) = match method {
        Some(method) => (method, method.name_all()),
        None => determine_name_method(ty),
    };

    let all = all.or(default_all).unwrap_or_default();
    (all, ty.clone(), method)
}

pub(crate) fn determine_type(expr: &syn::Expr) -> Option<(Span, syn::Type)> {
    Some((expr.span(), determine_type_inner(expr, false)?))
}

fn determine_type_inner(expr: &syn::Expr, neg: bool) -> Option<syn::Type> {
    match expr {
        syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
            syn::Lit::Str(..) => {
                return Some(syn::parse_quote!(str));
            }
            syn::Lit::ByteStr(..) => {
                return Some(syn::parse_quote!([u8]));
            }
            syn::Lit::Int(..) => {
                if neg {
                    return Some(syn::parse_quote!(isize));
                } else {
                    return Some(syn::parse_quote!(usize));
                }
            }
            _ => {}
        },
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(..),
            expr,
            ..
        }) => {
            return determine_type_inner(expr, !neg);
        }
        _ => {}
    }

    None
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

    (NameMethod::Sized, None)
}
