use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

use crate::expander::field_int;
use crate::expander::{
    Data, EnumData, Expander, ExpansionMode, FieldData, Result, StructData, TagMethod, VariantData,
};
use crate::internals::attr::{DefaultTag, EnumTagging, Packing};
use crate::internals::tokens::Tokens;
use crate::internals::{symbol::*, ModePath};
use crate::{
    expander::Taggable,
    internals::{Ctxt, Mode},
};

pub(crate) struct Build<'a> {
    pub(crate) input: &'a syn::DeriveInput,
    pub(crate) cx: &'a Ctxt,
    pub(crate) type_name: &'a syn::LitStr,
    pub(crate) tokens: &'a Tokens,
    pub(crate) expansion: ExpansionMode<'a>,
    pub(crate) data: BuildData<'a>,
    pub(crate) decode_t_decode: TokenStream,
    pub(crate) encode_t_encode: TokenStream,
    pub(crate) mode_ident: ModePath<'a>,
}

impl Build<'_> {
    /// Emit diagnostics for when we try to implement `Decode` for an enum which
    /// is marked as `#[musli(transparent)]`.
    pub(crate) fn encode_transparent_enum_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format!(
                "#[{}({})] cannot be used to encode enums",
                ATTR, TRANSPARENT
            ),
        );
    }

    /// Emit diagnostics indicating that we tried to implement decode for a
    /// packed enum.
    pub(crate) fn decode_packed_enum_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format!("#[{}({})] cannot be used to decode enums", ATTR, PACKED),
        );
    }

    /// Emit diagnostics indicating that we tried to use a `#[musli(default)]`
    /// annotation on a packed container.
    pub(crate) fn packed_default_diagnostics(&self, span: Span) {
        self.cx.error_span(
            span,
            format!(
                "#[{}({})] fields cannot be used in an packed container",
                ATTR, DEFAULT
            ),
        );
    }

    /// Emit diagnostics for a transparent encode / decode that failed because
    /// the wrong number of fields existed.
    pub(crate) fn transparent_diagnostics(&self, span: Span, fields: &[FieldBuild]) {
        if fields.is_empty() {
            self.cx.error_span(
                span,
                format!(
                    "#[{}({})] types must have a single field",
                    ATTR, TRANSPARENT
                ),
            );
        } else {
            self.cx.error_span(
                span,
                format!(
                    "#[{}({})] can only be used on types which have a single field",
                    ATTR, TRANSPARENT
                ),
            );
        }
    }
}

/// Build model for enums and structs.
pub(crate) enum BuildData<'a> {
    Struct(StructBuild<'a>),
    Enum(EnumBuild<'a>),
}

pub(crate) struct StructBuild<'a> {
    pub(crate) span: Span,
    pub(crate) fields: Vec<FieldBuild<'a>>,
    pub(crate) tag_type: Option<&'a (Span, syn::Type)>,
    pub(crate) packing: Packing,
    pub(crate) path: syn::Path,
    pub(crate) field_tag_method: TagMethod,
}

pub(crate) struct EnumBuild<'a> {
    pub(crate) span: Span,
    pub(crate) variants: Vec<VariantBuild<'a>>,
    pub(crate) fallback: Option<&'a syn::Ident>,
    pub(crate) variant_tag_method: TagMethod,
    pub(crate) tag_type: Option<&'a (Span, syn::Type)>,
    pub(crate) packing_span: Option<&'a (Span, Packing)>,
}

pub(crate) struct VariantBuild<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) name: &'a syn::LitStr,
    pub(crate) fields: Vec<FieldBuild<'a>>,
    pub(crate) packing: Packing,
    pub(crate) enum_tagging: Option<&'a EnumTagging>,
    pub(crate) enum_packing: Packing,
    pub(crate) tag: syn::Expr,
    pub(crate) tag_type: Option<&'a (Span, syn::Type)>,
    pub(crate) field_tag_method: TagMethod,
    pub(crate) is_default: bool,
    pub(crate) path: syn::Path,
    patterns: Vec<TokenStream>,
}

impl VariantBuild<'_> {
    /// Generate constructor for this variant.
    pub(crate) fn constructor(&self) -> TokenStream {
        let patterns = &self.patterns;
        let path = &self.path;
        quote_spanned!(self.span => #path { #(#patterns),* })
    }
}

pub(crate) struct FieldBuild<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) ident: Option<&'a syn::Ident>,
    pub(crate) encode_path: (Span, TokenStream),
    pub(crate) decode_path: (Span, TokenStream),
    pub(crate) tag: syn::Expr,
    pub(crate) skip_encoding_if: Option<(Span, &'a syn::ExprPath)>,
    pub(crate) default_attr: Option<Span>,
    pub(crate) access: TokenStream,
    pub(crate) packing: Packing,
}

/// Setup a build.
///
/// Handles mode decoding, and construction of parameters which might give rise to errors.
pub(crate) fn setup<'a>(e: &'a Expander, expansion: ExpansionMode<'a>) -> Result<Build<'a>> {
    let mode = expansion.as_mode(&e.tokens);

    e.validate_attributes(mode)?;

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
        type_name: &e.type_name,
        tokens: &e.tokens,
        expansion,
        data,
        decode_t_decode: mode.decode_t_decode(),
        encode_t_encode: mode.encode_t_encode(),
        mode_ident: mode.mode_ident(),
    })
}

fn setup_struct<'a>(
    e: &'a Expander,
    mode: Mode<'_>,
    data: &'a StructData<'a>,
) -> Result<StructBuild<'a>> {
    let mut fields = Vec::with_capacity(data.fields.len());

    let default_field_tag = e.type_attr.default_field_tag(mode);
    let tag_type = e.type_attr.tag_type(mode);
    let packing = e.type_attr.packing(mode).cloned().unwrap_or_default();
    let path = syn::Path::from(syn::Ident::new("Self", e.input.ident.span()));
    let mut tag_methods = TagMethods::new(&e.cx);

    for f in &data.fields {
        fields.push(setup_field(
            e,
            mode,
            f,
            default_field_tag,
            packing,
            None,
            &mut tag_methods,
        )?);
    }

    Ok(StructBuild {
        span: data.span,
        fields,
        tag_type,
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
    let tag_type = e.type_attr.tag_type(mode);
    let packing_span = e.type_attr.packing_span(mode);
    // Keep track of variant index manually since fallback variants do not
    // count.
    let mut tag_methods = TagMethods::new(&e.cx);

    for v in &data.variants {
        variants.push(setup_variant(e, mode, v, &mut fallback, &mut tag_methods)?);
    }

    Ok(EnumBuild {
        span: data.span,
        variants,
        fallback,
        variant_tag_method: tag_methods.pick(),
        tag_type,
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
        .cloned()
        .unwrap_or_default();

    let default_field_tag = data
        .attr
        .default_field_tag(mode)
        .or_else(|| e.type_attr.default_field_tag(mode));

    let enum_tagging = e.type_attr.enum_tagging(mode);
    let enum_packing = e.type_attr.packing(mode).cloned().unwrap_or_default();

    let (tag, tag_method) = data.expand_tag(e, mode, e.type_attr.default_variant_tag(mode))?;
    tag_methods.insert(data.span, tag_method);

    let mut path = syn::Path::from(syn::Ident::new("Self", data.span));
    path.segments.push(data.ident.clone().into());

    let tag_type = data.attr.tag_type(mode);

    let is_default = if data.attr.default_attr(mode).is_some() {
        if !data.fields.is_empty() {
            e.cx.error_span(
                data.span,
                format!("#[{}({})] variant must be empty", ATTR, DEFAULT),
            );

            false
        } else if fallback.is_some() {
            e.cx.error_span(
                data.span,
                format!(
                    "#[{}({})] only one fallback variant is supported",
                    ATTR, DEFAULT
                ),
            );

            false
        } else {
            *fallback = Some(data.ident);
            true
        }
    } else {
        false
    };

    let mut patterns = Vec::new();
    let mut field_tag_methods = TagMethods::new(&e.cx);

    for f in &data.fields {
        fields.push(setup_field(
            e,
            mode,
            f,
            default_field_tag,
            variant_packing,
            Some(&mut patterns),
            &mut field_tag_methods,
        )?);
    }

    Ok(VariantBuild {
        span: data.span,
        index: data.index,
        name: &data.name,
        fields,
        packing: variant_packing,
        enum_tagging,
        enum_packing,
        tag,
        tag_type,
        field_tag_method: field_tag_methods.pick(),
        is_default,
        path,
        patterns,
    })
}

fn setup_field<'a>(
    e: &'a Expander,
    mode: Mode<'_>,
    data: &'a FieldData<'a>,
    default_field_tag: Option<DefaultTag>,
    packing: Packing,
    patterns: Option<&mut Vec<TokenStream>>,
    tag_methods: &mut TagMethods,
) -> Result<FieldBuild<'a>> {
    let encode_path = data.attr.encode_path(mode, data.span);
    let decode_path = data.attr.decode_path(mode, data.span);
    let (tag, tag_method) = data.expand_tag(e, mode, default_field_tag)?;
    tag_methods.insert(data.span, tag_method);
    let skip_encoding_if = data.attr.skip_encoding_if(mode);
    let default_attr = data.attr.default_attr(mode);

    let access = if let Some(patterns) = patterns {
        match &data.ident {
            Some(ident) => {
                patterns.push(quote_spanned!(data.span => #ident));
                quote_spanned!(data.span => #ident)
            }
            None => {
                let index = field_int(data.index, data.span);
                let var = syn::Ident::new(&format!("v{}", data.index), data.span);
                patterns.push(quote_spanned!(data.span => #index: #var));
                quote_spanned!(data.span => #var)
            }
        }
    } else {
        match &data.ident {
            Some(ident) => quote_spanned!(data.span => &self.#ident),
            None => {
                let n = field_int(data.index, data.span);
                quote_spanned!(data.span => &self.#n)
            }
        }
    };

    Ok(FieldBuild {
        span: data.span,
        index: data.index,
        ident: data.ident,
        encode_path,
        decode_path,
        tag,
        skip_encoding_if,
        default_attr,
        access,
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
                    .error_span(span, format!("#[{}({})] conflicting tag kind", ATTR, TAG));
            }
        }
    }

    /// Pick a tag method to use.
    fn pick(self) -> TagMethod {
        self.methods.into_iter().next().unwrap_or_default()
    }
}
