use std::collections::BTreeSet;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::expander::Taggable;
use crate::expander::{
    Data, EnumData, Expander, FieldData, Result, StructData, TagMethod, VariantData,
};
use crate::internals::attr::{DefaultTag, EnumTagging, Packing};
use crate::internals::symbol::*;
use crate::internals::tokens::Tokens;
use crate::internals::{Ctxt, Expansion, Mode, ModePath, Only};

pub(crate) struct Build<'a> {
    pub(crate) input: &'a syn::DeriveInput,
    pub(crate) cx: &'a Ctxt,
    pub(crate) tokens: &'a Tokens,
    pub(crate) bounds: &'a [(Span, syn::WherePredicate)],
    pub(crate) decode_bounds: &'a [(Span, syn::WherePredicate)],
    pub(crate) expansion: Expansion<'a>,
    pub(crate) data: BuildData<'a>,
    pub(crate) decode_t_decode: syn::Path,
    pub(crate) encode_t_encode: syn::Path,
    pub(crate) mode_ident: ModePath<'a>,
    pub(crate) enum_tagging_span: Option<Span>,
}

impl Build<'_> {
    /// Emit diagnostics for when we try to implement `Decode` for an enum which
    /// is marked as `#[musli(transparent)]`.
    pub(crate) fn encode_transparent_enum_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format_args!("#[{ATTR}({TRANSPARENT})] cannot be used to encode enums",),
        );
    }

    /// Emit diagnostics indicating that we tried to implement decode for a
    /// packed enum.
    pub(crate) fn decode_packed_enum_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format_args!("#[{ATTR}({PACKED})] cannot be used to decode enums"),
        );
    }

    /// Emit diagnostics indicating that we tried to use a `#[musli(default)]`
    /// annotation on a packed container.
    pub(crate) fn packed_default_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format_args!("#[{ATTR}({DEFAULT})] fields cannot be used in an packed container",),
        );
    }

    /// Emit diagnostics for a transparent encode / decode that failed because
    /// the wrong number of fields existed.
    pub(crate) fn transparent_diagnostics(&self, span: Span, fields: &[FieldBuild]) {
        if fields.is_empty() {
            self.cx.error_span(
                span,
                format_args!("#[{ATTR}({TRANSPARENT})] types must have a single field",),
            );
        } else {
            self.cx.error_span(
                span,
                format_args!(
                    "#[{ATTR}({TRANSPARENT})] can only be used on types which have a single field",
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
                            "#[{ATTR}({TAG})] and #[{ATTR}({CONTENT})] are only supported on enums"
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
    Struct(StructBuild<'a>),
    Enum(EnumBuild<'a>),
}

pub(crate) struct StructBuild<'a> {
    pub(crate) span: Span,
    pub(crate) name: &'a syn::LitStr,
    pub(crate) fields: Vec<FieldBuild<'a>>,
    pub(crate) name_type: Option<&'a (Span, syn::Type)>,
    pub(crate) name_format_with: Option<&'a (Span, syn::Path)>,
    pub(crate) packing: Packing,
    pub(crate) path: syn::Path,
    pub(crate) field_tag_method: TagMethod,
}

pub(crate) struct EnumBuild<'a> {
    pub(crate) span: Span,
    pub(crate) name: &'a syn::LitStr,
    pub(crate) enum_tagging: Option<EnumTagging<'a>>,
    pub(crate) enum_packing: Packing,
    pub(crate) variants: Vec<VariantBuild<'a>>,
    pub(crate) fallback: Option<&'a syn::Ident>,
    pub(crate) variant_tag_method: TagMethod,
    pub(crate) name_type: Option<&'a (Span, syn::Type)>,
    pub(crate) name_format_with: Option<&'a (Span, syn::Path)>,
    pub(crate) packing_span: Option<&'a (Span, Packing)>,
}

pub(crate) struct VariantBuild<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) tag: syn::Expr,
    pub(crate) is_default: bool,
    pub(crate) st_: StructBuild<'a>,
    pub(crate) patterns: Punctuated<syn::FieldPat, Token![,]>,
}

pub(crate) struct FieldBuild<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) encode_path: (Span, syn::Path),
    pub(crate) decode_path: (Span, syn::Path),
    pub(crate) tag: syn::Expr,
    pub(crate) skip_encoding_if: Option<&'a (Span, syn::Path)>,
    pub(crate) default_attr: Option<Span>,
    pub(crate) self_access: syn::Expr,
    pub(crate) member: syn::Member,
    pub(crate) packing: Packing,
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
        decode_t_decode: mode.decode_t_decode(Span::call_site()),
        encode_t_encode: mode.encode_t_encode(Span::call_site()),
        mode_ident: mode.mode_ident(),
        enum_tagging_span: e.type_attr.enum_tagging_span(mode),
    })
}

fn setup_struct<'a>(
    e: &'a Expander,
    mode: Mode<'_>,
    data: &'a StructData<'a>,
) -> Result<StructBuild<'a>> {
    let mut fields = Vec::with_capacity(data.fields.len());

    let default_field_name = e.type_attr.default_field_name(mode).map(|&(_, v)| v);
    let packing = e
        .type_attr
        .packing(mode)
        .map(|&(_, p)| p)
        .unwrap_or_default();
    let path = syn::Path::from(syn::Ident::new("Self", e.input.ident.span()));
    let mut tag_methods = TagMethods::new(&e.cx);

    for f in &data.fields {
        fields.push(setup_field(
            e,
            mode,
            f,
            default_field_name,
            packing,
            None,
            &mut tag_methods,
        )?);
    }

    Ok(StructBuild {
        span: data.span,
        name: &data.name,
        fields,
        name_type: e.type_attr.name_type(mode),
        name_format_with: e.type_attr.name_format_with(mode),
        packing,
        path,
        field_tag_method: tag_methods.pick(),
    })
}

fn setup_enum<'a>(
    e: &'a Expander,
    mode: Mode<'_>,
    data: &'a EnumData<'a>,
) -> Result<EnumBuild<'a>> {
    let mut variants = Vec::with_capacity(data.variants.len());
    let mut fallback = None;
    // Keep track of variant index manually since fallback variants do not
    // count.
    let mut tag_methods = TagMethods::new(&e.cx);
    let enum_tagging = e.type_attr.enum_tagging(mode);

    let packing_span = e.type_attr.packing(mode);

    let enum_packing = e
        .type_attr
        .packing(mode)
        .map(|&(_, p)| p)
        .unwrap_or_default();

    if enum_tagging.is_some() {
        match packing_span {
            Some((_, Packing::Tagged)) => (),
            Some(&(span, packing)) => {
                e.cx.error_span(span, format_args!("#[{ATTR}({packing})] cannot be combined with #[{ATTR}({TAG})] or #[{ATTR}({CONTENT})]"));
                return Err(());
            }
            _ => (),
        }
    }

    for v in &data.variants {
        variants.push(setup_variant(e, mode, v, &mut fallback, &mut tag_methods)?);
    }

    Ok(EnumBuild {
        span: data.span,
        name: &data.name,
        enum_tagging,
        enum_packing,
        variants,
        fallback,
        variant_tag_method: tag_methods.pick(),
        name_type: e.type_attr.name_type(mode),
        name_format_with: e.type_attr.name_format_with(mode),
        packing_span,
    })
}

fn setup_variant<'a>(
    e: &'a Expander<'_>,
    mode: Mode<'_>,
    data: &'a VariantData<'a>,
    fallback: &mut Option<&'a syn::Ident>,
    tag_methods: &mut TagMethods,
) -> Result<VariantBuild<'a>> {
    let mut fields = Vec::with_capacity(data.fields.len());

    let variant_packing = data
        .attr
        .packing(mode)
        .or_else(|| e.type_attr.packing(mode))
        .map(|&(_, v)| v)
        .unwrap_or_default();

    let default_field_name = data
        .attr
        .default_field_name(mode)
        .or_else(|| e.type_attr.default_field_name(mode))
        .map(|&(_, v)| v);

    let (tag, tag_method) = data.expand_tag(
        e,
        mode,
        e.type_attr.default_variant_name(mode).map(|&(_, v)| v),
    )?;
    tag_methods.insert(data.span, tag_method);

    let mut path = syn::Path::from(syn::Ident::new("Self", data.span));
    path.segments.push(data.ident.clone().into());

    let is_default = if data.attr.default_attr(mode).is_some() {
        if !data.fields.is_empty() {
            e.cx.error_span(
                data.span,
                format_args!("#[{ATTR}({DEFAULT})] variant must be empty"),
            );

            false
        } else if fallback.is_some() {
            e.cx.error_span(
                data.span,
                format_args!("#[{ATTR}({DEFAULT})] only one fallback variant is supported",),
            );

            false
        } else {
            *fallback = Some(data.ident);
            true
        }
    } else {
        false
    };

    let mut patterns = Punctuated::default();
    let mut field_tag_methods = TagMethods::new(&e.cx);

    for f in &data.fields {
        fields.push(setup_field(
            e,
            mode,
            f,
            default_field_name,
            variant_packing,
            Some(&mut patterns),
            &mut field_tag_methods,
        )?);
    }

    Ok(VariantBuild {
        span: data.span,
        index: data.index,
        tag,
        is_default,
        patterns,
        st_: StructBuild {
            span: data.span,
            name: &data.name,
            fields,
            packing: variant_packing,
            name_type: data.attr.name_type(mode),
            name_format_with: data.attr.name_format_with(mode),
            field_tag_method: field_tag_methods.pick(),
            path,
        },
    })
}

fn setup_field<'a>(
    e: &'a Expander,
    mode: Mode<'_>,
    data: &'a FieldData<'a>,
    default_field_name: Option<DefaultTag>,
    packing: Packing,
    patterns: Option<&mut Punctuated<syn::FieldPat, Token![,]>>,
    tag_methods: &mut TagMethods,
) -> Result<FieldBuild<'a>> {
    let encode_path = data.attr.encode_path_expanded(mode, data.span);
    let decode_path = data.attr.decode_path_expanded(mode, data.span);
    let (tag, tag_method) = data.expand_tag(e, mode, default_field_name)?;
    tag_methods.insert(data.span, tag_method);
    let skip_encoding_if = data.attr.skip_encoding_if(mode);
    let default_attr = data.attr.default_field(mode).map(|&(s, ())| s);

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

    Ok(FieldBuild {
        span: data.span,
        index: data.index,
        encode_path,
        decode_path,
        tag,
        skip_encoding_if,
        default_attr,
        self_access,
        member,
        packing,
    })
}

struct TagMethods<'a> {
    cx: &'a Ctxt,
    methods: BTreeSet<TagMethod>,
}

impl<'a> TagMethods<'a> {
    fn new(cx: &'a Ctxt) -> Self {
        Self {
            cx,
            methods: BTreeSet::new(),
        }
    }

    /// Insert a tag method and error in case it's invalid.
    fn insert(&mut self, span: Span, method: Option<TagMethod>) {
        let before = self.methods.len();

        if let Some(method) = method {
            self.methods.insert(method);

            if before == 1 && self.methods.len() > 1 {
                self.cx
                    .error_span(span, format_args!("#[{ATTR}({TAG})] conflicting tag kind"));
            }
        }
    }

    /// Pick a tag method to use.
    fn pick(self) -> TagMethod {
        self.methods.into_iter().next().unwrap_or_default()
    }
}
