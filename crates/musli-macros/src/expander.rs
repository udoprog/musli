use std::borrow::Cow;
use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

use crate::internals::attr::{self, ModeIdent, ModeKind, TypeAttr};
use crate::internals::{Build, Ctxt, Expansion, Mode, NameAll, Only, Parameters, Result, Tokens};

#[derive(Clone, Copy)]
pub(crate) enum UnsizedMethod {
    Default,
    Bytes,
}

impl UnsizedMethod {
    /// Get corresponding decoder method name to use.
    pub(crate) fn as_method_name(&self) -> syn::Ident {
        match self {
            Self::Default => syn::Ident::new("decode_unsized", Span::call_site()),
            Self::Bytes => syn::Ident::new("decode_unsized_bytes", Span::call_site()),
        }
    }
}

pub(crate) struct Name<'a, T> {
    pub(crate) span: Option<Span>,
    pub(crate) value: &'a T,
    pub(crate) ty: syn::Type,
    pub(crate) method: NameMethod,
    pub(crate) format_with: Option<&'a (Span, syn::Path)>,
}

impl<T> Name<'_, T> {
    pub(crate) fn expr(&self, ident: syn::Ident) -> syn::Expr {
        match self.method {
            NameMethod::Unsized(..) => syn::parse_quote!(#ident),
            NameMethod::Sized => syn::parse_quote!(&#ident),
        }
    }

    pub(crate) fn ty(&self) -> Cow<'_, syn::Type> {
        match self.method {
            NameMethod::Unsized(..) => {
                let ty = &self.ty;
                Cow::Owned(syn::parse_quote!(&#ty))
            }
            NameMethod::Sized => Cow::Borrowed(&self.ty),
        }
    }

    pub(crate) fn name_format(&self, value: &syn::Ident) -> syn::Expr {
        match self.format_with {
            Some((_, path)) => syn::parse_quote!(#path(&#value)),
            None => syn::parse_quote!(&#value),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) enum NameMethod {
    /// Load the tag by value.
    #[default]
    Sized,
    /// Load the tag by visit.
    Unsized(UnsizedMethod),
}

impl NameMethod {
    pub(crate) fn name_all(&self) -> Option<NameAll> {
        match self {
            Self::Sized => None,
            Self::Unsized(_) => Some(NameAll::Name),
        }
    }
}

impl Parse for NameMethod {
    fn parse(input: ParseStream<'_>) -> Result<Self, syn::Error> {
        let string: syn::LitStr = input.parse()?;
        let s = string.value();

        match s.as_str() {
            "sized" => Ok(Self::Sized),
            "unsized" => Ok(Self::Unsized(UnsizedMethod::Default)),
            "unsized_bytes" => Ok(Self::Unsized(UnsizedMethod::Bytes)),
            _ => Err(syn::Error::new_spanned(
                string,
                "#[musli(name(method = ..))]: Bad value, expected one of \"value\", \"unsized\", \"unsized_bytes\"",
            )),
        }
    }
}

pub(crate) struct FieldData<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) attr: attr::Field,
    pub(crate) ident: Option<&'a syn::Ident>,
    pub(crate) ty: &'a syn::Type,
}

pub(crate) struct StructData<'a> {
    pub(crate) name: syn::LitStr,
    pub(crate) fields: Vec<FieldData<'a>>,
    pub(crate) kind: StructKind,
}

pub(crate) struct VariantData<'a> {
    pub(crate) span: Span,
    pub(crate) name: syn::LitStr,
    pub(crate) index: usize,
    pub(crate) attr: attr::VariantAttr,
    pub(crate) ident: &'a syn::Ident,
    pub(crate) fields: Vec<FieldData<'a>>,
    pub(crate) kind: StructKind,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum StructKind {
    Indexed(usize),
    Named,
    Empty,
}

pub(crate) struct EnumData<'a> {
    pub(crate) name: syn::LitStr,
    pub(crate) variants: Vec<VariantData<'a>>,
}

pub(crate) enum Data<'a> {
    Struct(StructData<'a>),
    Enum(EnumData<'a>),
    Union,
}

pub(crate) struct Expander<'a> {
    pub(crate) input: &'a syn::DeriveInput,
    pub(crate) cx: Ctxt,
    pub(crate) type_attr: TypeAttr,
    pub(crate) data: Data<'a>,
    pub(crate) prefix: syn::Path,
    pub(crate) default: Vec<ModeIdent>,
}

impl<'a> Expander<'a> {
    pub(crate) fn new(input: &'a syn::DeriveInput, default_crate: &str) -> Self {
        fn fields<'a>(cx: &Ctxt, fields: &'a syn::Fields) -> Vec<FieldData<'a>> {
            fields
                .iter()
                .enumerate()
                .map(|(index, field)| FieldData {
                    span: field.span(),
                    index,
                    attr: attr::field_attrs(cx, &field.attrs),
                    ident: field.ident.as_ref(),
                    ty: &field.ty,
                })
                .collect()
        }

        let cx = Ctxt::new();
        let type_attr = attr::type_attrs(&cx, &input.attrs);

        let data = match &input.data {
            syn::Data::Struct(st) => Data::Struct(StructData {
                name: syn::LitStr::new(&input.ident.to_string(), input.ident.span()),
                fields: fields(&cx, &st.fields),
                kind: match &st.fields {
                    syn::Fields::Unit => StructKind::Empty,
                    syn::Fields::Unnamed(f) => StructKind::Indexed(f.unnamed.len()),
                    syn::Fields::Named(..) => StructKind::Named,
                },
            }),
            syn::Data::Enum(en) => {
                let variants = en
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| VariantData {
                        span: variant.span(),
                        index,
                        name: syn::LitStr::new(&variant.ident.to_string(), variant.ident.span()),
                        attr: attr::variant_attrs(&cx, &variant.attrs),
                        ident: &variant.ident,
                        fields: fields(&cx, &variant.fields),
                        kind: match &variant.fields {
                            syn::Fields::Unit => StructKind::Empty,
                            syn::Fields::Unnamed(f) => StructKind::Indexed(f.unnamed.len()),
                            syn::Fields::Named(..) => StructKind::Named,
                        },
                    });

                Data::Enum(EnumData {
                    name: syn::LitStr::new(&input.ident.to_string(), input.ident.span()),
                    variants: variants.collect(),
                })
            }
            syn::Data::Union(..) => Data::Union,
        };

        let prefix = type_attr.crate_or_default(default_crate);

        let default = vec![
            ModeIdent {
                kind: ModeKind::Binary,
                path: syn::Path::from(syn::PathSegment::from(syn::Ident::new(
                    "Binary",
                    Span::call_site(),
                ))),
            },
            ModeIdent {
                kind: ModeKind::Text,
                path: syn::Path::from(syn::PathSegment::from(syn::Ident::new(
                    "Text",
                    Span::call_site(),
                ))),
            },
        ];

        Self {
            input,
            cx,
            type_attr,
            data,
            prefix,
            default,
        }
    }

    pub(crate) fn tokens(&self) -> Tokens<'_> {
        Tokens::new(&self.prefix)
    }

    /// Coerce into errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        self.cx.into_errors()
    }

    fn setup_builds<'b>(
        &'b self,
        modes: &'b [ModeIdent],
        tokens: &'b Tokens<'b>,
        only: Only,
    ) -> Result<Vec<Build<'b>>> {
        let mut builds = Vec::new();

        let mut missing = BTreeMap::new();

        for default in &self.default {
            missing.insert(&default.kind, default);
        }

        let (lt, lt_exists) = 'out: {
            if let Some(lt) = self.input.generics.lifetimes().next() {
                break 'out (lt.lifetime.clone(), true);
            }

            (syn::Lifetime::new("'__de", Span::call_site()), false)
        };

        let (allocator_ident, allocator_exists) = 'out: {
            for p in self.input.generics.type_params() {
                if p.ident == "A" {
                    break 'out (p.ident.clone(), true);
                }
            }

            (
                self.cx.type_with_span_permanent("__A", Span::call_site()),
                false,
            )
        };

        let p = Parameters {
            lt,
            lt_exists,
            allocator_ident,
            allocator_exists,
        };

        for mode_ident in modes {
            missing.remove(&mode_ident.kind);

            let expansion = Expansion { mode_ident };

            let mode = expansion.as_mode(tokens, only);
            let p = self.decorate(&p, &mode);

            builds.push(crate::internals::build::setup(
                self, expansion, mode, tokens, p,
            )?);
        }

        for (_, mode_ident) in missing {
            let expansion = Expansion { mode_ident };

            let mode = expansion.as_mode(tokens, only);
            let p = self.decorate(&p, &mode);

            builds.push(crate::internals::build::setup(
                self, expansion, mode, tokens, p,
            )?);
        }

        Ok(builds)
    }

    fn decorate(&self, p: &Parameters, mode: &Mode<'_>) -> Parameters {
        let (lt, lt_exists) = 'out: {
            let list = self.type_attr.decode_bounds_lifetimes(mode);

            if let [_, rest @ ..] = list {
                for &(span, _) in rest {
                    self.cx
                        .error_span(span, "More than one decoder lifetime bound is specified");
                }
            }

            if let Some((_, ty)) = list.first() {
                break 'out (ty, false);
            }

            (&p.lt, p.lt_exists)
        };

        let (allocator_ident, allocator_exists) = 'out: {
            let list = self.type_attr.decode_bounds_types(mode);

            if let [_, rest @ ..] = list {
                for &(span, _) in rest {
                    self.cx
                        .error_span(span, "More than one decoder allocator bound is specified");
                }
            }

            if let Some((_, ty)) = list.first() {
                break 'out (ty, false);
            }

            (&p.allocator_ident, p.allocator_exists)
        };

        Parameters {
            lt: lt.clone(),
            lt_exists,
            allocator_ident: allocator_ident.clone(),
            allocator_exists,
        }
    }

    /// Expand Encode implementation.
    pub(crate) fn expand_encode(&self) -> Result<TokenStream> {
        let modes = self.cx.modes();
        let tokens = self.tokens();
        let builds = self.setup_builds(&modes, &tokens, Only::Encode)?;

        let mut out = TokenStream::new();

        for build in builds {
            out.extend(crate::en::expand_encode_entry(&build)?);
        }

        Ok(out)
    }

    /// Expand Decode implementation.
    pub(crate) fn expand_decode(&self) -> Result<TokenStream> {
        let modes = self.cx.modes();
        let tokens = self.tokens();
        let builds = self.setup_builds(&modes, &tokens, Only::Decode)?;

        let mut out = TokenStream::new();

        for build in builds {
            out.extend(crate::de::expand_decode_entry(&build)?);
        }

        Ok(out)
    }
}

/// A thing that determines how it's tagged.
pub(crate) trait Taggable {
    /// The span of the taggable item.
    fn span(&self) -> Span;
    /// The rename configuration the taggable item currently has.
    fn name(&self, mode: &Mode<'_>) -> Option<&(Span, syn::Expr)>;
    /// The index of the taggable item.
    fn index(&self) -> usize;
}

/// Expand the given configuration to the appropriate tag expression.
pub(crate) fn expand_name(
    taggable: &dyn Taggable,
    mode: &Mode<'_>,
    name_all: NameAll,
    ident: Option<&syn::Ident>,
) -> (syn::Expr, Option<Span>) {
    let lit = 'out: {
        if let Some(&(span, ref rename)) = taggable.name(mode) {
            return (rename.clone(), Some(span));
        }

        if let (Some(ident), name_all) = (ident, name_all) {
            let ident = ident.to_string();
            let ident = ident.trim_start_matches("r#");

            if let Some(name) = name_all.apply(ident) {
                break 'out syn::LitStr::new(&name, ident.span()).into();
            }
        }

        usize_suffixed(taggable.index(), taggable.span()).into()
    };

    let expr = syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit,
    });

    (expr, None)
}

/// Ensure that the given integer is usize-suffixed so that it is treated as the
/// appropriate type.
pub(crate) fn usize_suffixed(index: usize, span: Span) -> syn::LitInt {
    syn::LitInt::new(&format!("{}usize", index), span)
}

impl Taggable for FieldData<'_> {
    fn span(&self) -> Span {
        self.span
    }

    fn name(&self, mode: &Mode<'_>) -> Option<&(Span, syn::Expr)> {
        self.attr.name(mode)
    }

    fn index(&self) -> usize {
        self.index
    }
}

impl Taggable for VariantData<'_> {
    fn span(&self) -> Span {
        self.span
    }

    fn name(&self, mode: &Mode<'_>) -> Option<&(Span, syn::Expr)> {
        self.attr.name_expr(mode)
    }

    fn index(&self) -> usize {
        self.index
    }
}
