use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::Token;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use crate::expander::{
    self, Data, EnumData, Expander, FieldData, Name, NameMethod, StructData, StructKind,
    UnsizedMethod, VariantData,
};
use crate::internals::mode::AllocatorParam;

use super::attr::{DefaultOrCustom, EnumTagging, FieldEncoding, ModeKind, MusliBound, Packing};
use super::mode::ImportedMethod;
use super::name::NameAll;
use super::{ATTR, Ctxt, Expansion, Mode, Only, Result, Tokens};

pub(crate) struct Parameters {
    pub(crate) lt: syn::Lifetime,
    pub(crate) lt_exists: bool,
    pub(crate) allocator_ident: syn::Ident,
    pub(crate) allocator_exists: bool,
}

pub(crate) struct Build<'a> {
    pub(crate) mode: Mode<'a>,
    pub(crate) input: &'a syn::DeriveInput,
    pub(crate) cx: &'a Ctxt,
    pub(crate) bounds: &'a [(Span, MusliBound)],
    pub(crate) decode_bounds: &'a [(Span, MusliBound)],
    pub(crate) expansion: Expansion<'a>,
    pub(crate) data: BuildData<'a>,
    pub(crate) decode_t_decode: ImportedMethod<'a>,
    pub(crate) encode_t_encode: ImportedMethod<'a>,
    pub(crate) enum_tag: Option<Span>,
    pub(crate) enum_content: Option<Span>,
    pub(crate) tokens: &'a Tokens<'a>,
    pub(crate) p: Parameters,
}

impl Build<'_> {
    /// Validate encode attributes.
    pub(crate) fn validate_encode(&self) -> Result<()> {
        self.validate(Only::Encode)
    }

    /// Validate decode attribute.
    pub(crate) fn validate_decode(&self) -> Result<()> {
        self.validate(Only::Decode)
    }

    fn validate(&self, only: Only) -> Result<()> {
        self.data
            .validate(only, self.cx, &self.mode, self.enum_tag, self.enum_content);
        Ok(())
    }
}

/// Build model for enums and structs.
pub(crate) enum BuildData<'a> {
    Struct(Box<Body<'a>>),
    Enum(Box<Enum<'a>>),
}

impl BuildData<'_> {
    fn validate(
        &self,
        _: Only,
        cx: &Ctxt,
        mode: &Mode<'_>,
        enum_tag: Option<Span>,
        enum_content: Option<Span>,
    ) {
        match self {
            BuildData::Struct(body) => {
                if let Some(span) = enum_tag {
                    cx.error_span(
                        span,
                        format_args!(
                            "In {mode} the #[{ATTR}(tag)] attribute is only supported on enums"
                        ),
                    );
                }

                if let Some(span) = enum_content {
                    cx.error_span(
                        span,
                        format_args!(
                            "In {mode} the #[{ATTR}(content)] attribute is only supported on enums"
                        ),
                    );
                }

                body.validate(cx, mode, "struct", "structs");
            }
            BuildData::Enum(en) => {
                match en.packing {
                    (span, Packing::Transparent) => {
                        cx.error_span(
                            span,
                            format_args!("In {mode} an enum cannot be #[{ATTR}(transparent)]"),
                        );
                    }
                    (span, Packing::Packed) => {
                        cx.error_span(
                            span,
                            format_args!("In {mode} an enum cannot be #[{ATTR}(packed)]"),
                        );
                    }
                    (_, Packing::Untagged) => {
                        if let Some(span) = enum_tag {
                            cx.error_span(
                                span,
                                format_args!(
                                    "In {mode} a #[{ATTR}(untagged)] enum cannot use #[{ATTR}(tag)]"
                                ),
                            );
                        }

                        if let Some(span) = enum_content {
                            cx.error_span(span, format_args!("In {mode} a #[{ATTR}(untagged)] enum cannot use #[{ATTR}(content)]"));
                        }
                    }
                    _ => (),
                }

                for v in &en.variants {
                    v.st.validate(cx, mode, "variant", "variants");

                    if let ((_, Packing::Packed), Some(span)) = (v.st.packing, enum_tag) {
                        cx.error_span(
                            span,
                            format_args!(
                                "In {mode} a #[{ATTR}(packed)] variant cannot be used in an enum using #[{ATTR}(tag)]"
                            ),
                        );
                    }
                }
            }
        }
    }
}

pub(crate) struct Body<'a> {
    pub(crate) name: Name<'a, syn::LitStr>,
    pub(crate) all_fields: Vec<Field<'a>>,
    pub(crate) packing: (Span, Packing),
    pub(crate) kind: StructKind,
    pub(crate) path: syn::Path,
}

impl<'a> Body<'a> {
    /// Iterate over unskipped fields.
    #[inline]
    pub(crate) fn unskipped_fields(&self) -> impl Iterator<Item = &Field<'a>> {
        self.all_fields.iter().filter(|f| f.skip.is_none())
    }

    /// Construct field tests.
    #[inline]
    pub(crate) fn field_tests(&self) -> impl Iterator<Item = impl ToTokens> + '_ {
        self.unskipped_fields().flat_map(|f| {
            let Field {
                skip_encoding_if,
                access,
                var,
                ..
            } = f;

            let (_, path) = skip_encoding_if.as_ref()?;

            Some(quote! {
                let #var = !#path(#access);
            })
        })
    }

    /// Access the single transparent field in the body.
    pub(crate) fn transparent_field(&self) -> Result<&Field<'a>, ()> {
        let mut it = self.unskipped_fields();
        let f = it.next().ok_or(())?;

        if it.next().is_some() {
            return Err(());
        }

        Ok(f)
    }

    pub(crate) fn validate(
        &self,
        cx: &Ctxt,
        mode: &Mode<'_>,
        singular: &'static str,
        plural: &'static str,
    ) {
        match self.packing {
            (_, Packing::Packed) => {
                let mut last_default = None;

                for f in self.unskipped_fields() {
                    if let Some((span, _)) = f.default_attr {
                        last_default = Some(span);
                    } else if let Some(span) = last_default {
                        cx.error_span(
                            span,
                            format_args!(
                                "Only #[{ATTR}(default)] fields can be used at the end of packed containers"
                            ),
                        );
                    }
                }

                if let Some(span) = self.name.span {
                    cx.error_span(
                        span,
                        format_args!(
                            "In {mode} a #[{ATTR}(packed)] {singular} cannot have named fields"
                        ),
                    );
                }
            }
            (_, Packing::Transparent) if self.transparent_field().is_err() => {
                'done: {
                    if self.all_fields.is_empty() {
                        cx.error_span(
                            self.packing.0,
                            format_args!(
                                "A #[{ATTR}(transparent)] {singular} must have a single field"
                            ),
                        );

                        break 'done;
                    }

                    if self.unskipped_fields().next().is_none() {
                        cx.error_span(
                            self.packing.0,
                            format_args!(
                                "A #[{ATTR}(transparent)] {singular} must have a single unskipped field"
                            ),
                        );

                        break 'done;
                    }

                    cx.error_span(
                        self.packing.0,
                        format_args!(
                            "A #[{ATTR}(transparent)] attribute can only be used on {plural} which have a single field",
                        ),
                    );
                };
            }
            (span, Packing::Untagged) => {
                cx.error_span(
                    span,
                    format_args!("In {mode} a {singular} cannot be #[{ATTR}(untagged)]"),
                );
            }
            _ => {}
        }

        if let (_, packing @ (Packing::Transparent | Packing::Packed)) = self.packing {
            for f in &self.all_fields {
                if let Some(span) = f.name_span {
                    cx.error_span(
                        span,
                        format_args!("A #[{ATTR}({packing})]{singular} cannot have named fields"),
                    );
                }

                if let Some(span) = f.pattern {
                    cx.error_span(
                        span.span(),
                        format_args!("A #[{ATTR}({packing})]{singular} cannot have field patterns"),
                    );
                }
            }
        }

        if matches!(self.packing, (_, Packing::Transparent)) {
            for f in &self.all_fields {
                if let Some(&(span, _)) = f.skip_encoding_if {
                    cx.error_span(
                        span,
                        format_args!(
                            "A #[{ATTR}(transparent)] {singular} cannot have an optional field"
                        ),
                    );
                }
            }
        }
    }
}

pub(crate) struct Enum<'a> {
    pub(crate) name: Name<'a, syn::LitStr>,
    pub(crate) enum_tagging: EnumTagging<'a>,
    pub(crate) packing: (Span, Packing),
    pub(crate) variants: Vec<Variant<'a>>,
    pub(crate) fallback: Option<&'a syn::Ident>,
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
    pub(crate) allocator_param: AllocatorParam<'a>,
    pub(crate) encode_path: (Span, DefaultOrCustom<'a>),
    pub(crate) decode_path: (Span, DefaultOrCustom<'a>),
    pub(crate) size_hint_path: Option<(Span, DefaultOrCustom<'a>)>,
    pub(crate) name: syn::Expr,
    pub(crate) name_span: Option<Span>,
    pub(crate) pattern: Option<&'a syn::Pat>,
    /// Skip field entirely and always initialize with the specified expresion,
    /// or default value through `default_attr`.
    pub(crate) skip: Option<Span>,
    pub(crate) skip_encoding_if: Option<&'a (Span, syn::Path)>,
    /// Fill with default value, if missing.
    pub(crate) default_attr: Option<(Span, Option<&'a syn::Path>)>,
    pub(crate) access: syn::Expr,
    pub(crate) member: syn::Member,
    pub(crate) var: syn::Ident,
    pub(crate) ty: &'a syn::Type,
}

impl Field<'_> {
    /// If the field is skipped, set up the expression which initializes the
    /// field to its default value.
    pub(crate) fn init_default(&self, b: &Build<'_>) -> Option<TokenStream> {
        let span = *self.skip.as_ref()?;

        let Tokens {
            default_function, ..
        } = b.tokens;

        let ty = self.ty;

        Some(match &self.default_attr {
            Some((_, Some(path))) => quote_spanned!(span => #path()),
            _ => quote_spanned!(span => #default_function::<#ty>()),
        })
    }
}

/// Setup a build.
///
/// Handles mode decoding, and construction of parameters which might give rise to errors.
pub(crate) fn setup<'a>(
    e: &'a Expander,
    expansion: Expansion<'a>,
    mode: Mode<'a>,
    tokens: &'a Tokens<'a>,
    p: Parameters,
) -> Result<Build<'a>> {
    let data = match &e.data {
        Data::Struct(data) => {
            BuildData::Struct(Box::new(setup_struct(e, &mode, data, &p.allocator_ident)))
        }
        Data::Enum(data) => {
            BuildData::Enum(Box::new(setup_enum(e, &mode, data, &p.allocator_ident)))
        }
        Data::Union => {
            e.cx.error_span(e.input.ident.span(), "musli: not supported for unions");
            return Err(());
        }
    };

    if e.cx.has_errors() {
        return Err(());
    }

    let decode_t_decode = mode.decode_t_decode(
        FieldEncoding::Default,
        AllocatorParam::Ident(p.allocator_ident.clone()),
    );
    let encode_t_encode = mode.encode_t_encode(FieldEncoding::Default);

    let bounds = e.type_attr.bounds(&mode);
    let decode_bounds = e.type_attr.decode_bounds(&mode);
    let enum_tag = e.type_attr.tag_value(&mode).map(|(s, _)| *s);
    let enum_content = e.type_attr.content_value(&mode).map(|(s, _)| *s);

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
        enum_tag,
        enum_content,
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
    let mut all_fields = Vec::with_capacity(data.fields.len());

    let packing = e
        .type_attr
        .packing(mode)
        .map(|&(span, p)| (span, p))
        .unwrap_or_else(|| (Span::call_site(), Packing::default()));

    let (name_all, name_type, name_method, name_span) = match data.kind {
        StructKind::Indexed(..) if e.type_attr.is_name_type_ambiguous(mode) => {
            let name_all = NameAll::Index;
            (name_all, name_all.ty(), NameMethod::Sized, None)
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
        all_fields.push(setup_field(e, mode, f, name_all, None, allocator_ident));
    }

    Body {
        name: Name {
            span: name_span,
            value: &data.name,
            ty: name_type,
            method: name_method,
            format_with: e.type_attr.name_format_with(mode),
        },
        all_fields,
        packing,
        kind: data.kind,
        path,
    }
}

fn setup_enum<'a>(
    e: &'a Expander,
    mode: &Mode<'a>,
    data: &'a EnumData<'a>,
    allocator_ident: &syn::Ident,
) -> Enum<'a> {
    let mut variants = Vec::with_capacity(data.variants.len());
    let mut fallback = None;

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

    let packing = e
        .type_attr
        .packing(mode)
        .map(|&(span, p)| (span, p))
        .unwrap_or_else(|| (Span::call_site(), Packing::default()));

    let (_, name_type, name_method, name_span) = split_name(
        mode.kind,
        e.type_attr.name_type(mode),
        e.type_attr.name_all(mode),
        e.type_attr.name_method(mode),
    );

    for v in &data.variants {
        variants.push(setup_variant(e, mode, v, &mut fallback, allocator_ident));
    }

    Enum {
        name: Name {
            span: name_span,
            value: &data.name,
            ty: name_type,
            method: name_method,
            format_with: e.type_attr.name_format_with(mode),
        },
        enum_tagging,
        packing,
        variants,
        fallback,
    }
}

fn setup_variant<'a>(
    e: &'a Expander<'_>,
    mode: &Mode<'a>,
    data: &'a VariantData<'a>,
    fallback: &mut Option<&'a syn::Ident>,
    allocator_ident: &syn::Ident,
) -> Variant<'a> {
    let mut all_fields = Vec::with_capacity(data.fields.len());

    let packing = data
        .attr
        .packing(mode)
        .map(|&(span, v)| (span, v))
        .unwrap_or_else(|| (Span::call_site(), Packing::default()));

    let (name_all, name_type, name_method, name_span) = match data.kind {
        StructKind::Indexed(..) if data.attr.is_name_type_ambiguous(mode) => {
            let name_all = NameAll::Index;
            (name_all, name_all.ty(), NameMethod::Sized, None)
        }
        _ => split_name(
            mode.kind,
            data.attr.name_type(mode),
            data.attr.name_all(mode),
            data.attr.name_method(mode),
        ),
    };

    let (type_name_all, _, _, _) = split_name(
        mode.kind,
        e.type_attr.name_type(mode),
        e.type_attr.name_all(mode),
        e.type_attr.name_method(mode),
    );

    let (name, _) = expander::expand_name(data, mode, type_name_all, Some(data.ident));

    let pattern = data.attr.pattern(mode).map(|(_, p)| p);

    let mut path = syn::Path::from(syn::Ident::new("Self", data.span));
    path.segments.push(data.ident.clone().into());

    if let Some(&(span, _)) = data.attr.default_variant(mode) {
        if !data.fields.is_empty() {
            e.cx.error_span(
                span,
                format_args!("In {mode} the #[{ATTR}(default)] variant must be empty"),
            );
        } else if fallback.is_some() {
            e.cx.error_span(
                span,
                format_args!("In {mode} only one #[{ATTR}(default)] variant is supported"),
            );
        } else {
            *fallback = Some(data.ident);
        }
    }

    let mut patterns = Punctuated::default();

    for f in &data.fields {
        let field = setup_field(e, mode, f, name_all, Some(&mut patterns), allocator_ident);

        all_fields.push(field);
    }

    let st = Body {
        name: Name {
            span: name_span,
            value: &data.name,
            ty: name_type,
            method: name_method,
            format_with: data.attr.name_format_with(mode),
        },
        all_fields,
        packing,
        kind: data.kind,
        path,
    };

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
    patterns: Option<&mut Punctuated<syn::FieldPat, Token![,]>>,
    allocator_ident: &syn::Ident,
) -> Field<'a> {
    let allocator_param = match data.attr.alloc(mode) {
        None => AllocatorParam::Ident(allocator_ident.clone()),
        Some(alloc) => AllocatorParam::Alloc(alloc),
    };

    let encode_path = data.attr.encode_path_expanded(mode, data.span);
    let decode_path = data
        .attr
        .decode_path_expanded(mode, data.span, allocator_param.clone());
    let size_hint_path = data.attr.size_hint_path_expanded(mode, data.span);

    let (name, name_span) = expander::expand_name(data, mode, name_all, data.ident);
    let pattern = data.attr.pattern(mode);

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

    let access;

    if let Some(patterns) = patterns {
        match data.ident {
            Some(ident) => {
                let colon_token;
                let pat: syn::Pat;

                if skip.is_none() {
                    colon_token = None;
                    pat = syn::parse_quote!(ref #ident);
                    access = syn::parse_quote!(#ident);
                } else {
                    colon_token = Some(<Token![:]>::default());
                    pat = syn::parse_quote!(_);
                    access = syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(quote::format_ident!("skipped_{ident}")),
                    });
                };

                patterns.push(syn::FieldPat {
                    attrs: Vec::new(),
                    member: syn::Member::Named(ident.clone()),
                    colon_token,
                    pat: Box::new(pat),
                });
            }
            None => {
                let pat;
                let path;

                if skip.is_none() {
                    let var = quote::format_ident!("v{}", data.index);
                    pat = syn::parse_quote!(ref #var);
                    path = var.into();
                } else {
                    pat = syn::parse_quote!(_);
                    path = syn::Path::from(quote::format_ident!("skipped_{}", data.index));
                };

                access = syn::Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                });

                patterns.push(syn::FieldPat {
                    attrs: Vec::new(),
                    member: syn::Member::Unnamed(syn::Index::from(data.index)),
                    colon_token: Some(<Token![:]>::default()),
                    pat: Box::new(pat),
                });
            }
        }
    } else {
        access = syn::parse_quote!(&self.#member);
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
        allocator_param,
        encode_path,
        decode_path,
        size_hint_path,
        name,
        name_span,
        pattern: pattern.map(|(_, p)| p),
        skip,
        skip_encoding_if,
        default_attr,
        access,
        member,
        var,
        ty: data.ty,
    }
}

pub(crate) fn split_name(
    kind: Option<&ModeKind>,
    ty: Option<&(Span, syn::Type)>,
    all: Option<&(Span, NameAll)>,
    method: Option<&(Span, NameMethod)>,
) -> (NameAll, syn::Type, NameMethod, Option<Span>) {
    let kind_name_all = kind.and_then(ModeKind::default_name_all);

    let span = ty
        .map(|&(span, _)| span)
        .or(all.map(|&(span, _)| span))
        .or(method.map(|&(span, _)| span));

    let all = all.map(|&(_, v)| v);
    let method = method.map(|&(_, v)| v);

    let Some((_, ty)) = ty else {
        let all = all.or(kind_name_all).unwrap_or_default();
        let method = method.unwrap_or_else(|| all.name_method());
        return (all, all.ty(), method, span);
    };

    let (method, default_all) = match method {
        Some(method) => (method, method.name_all()),
        None => determine_name_method(ty),
    };

    let all = all.or(default_all).unwrap_or_default();
    (all, ty.clone(), method, span)
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

/// Extract existing ident bounds which are present, so that the default type bounds can be excluded on this basis.
pub(crate) fn existing_bounds(bounds: &[(Span, MusliBound)]) -> HashSet<&syn::Ident> {
    let mut idents = HashSet::new();

    for (_, bound) in bounds {
        let Some(ident) = bound.as_ident() else {
            continue;
        };

        idents.insert(ident);
    }

    idents
}
