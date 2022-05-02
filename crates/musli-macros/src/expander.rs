use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::internals::attr::{self, DefaultTag, TypeAttr};
use crate::internals::symbol::*;
use crate::internals::tokens::Tokens;
use crate::internals::{Ctxt, Mode, ModePath};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TagMethod {
    /// The default tag method.
    Default,
    /// Special method that requires generating a visitor.
    String,
}

impl Default for TagMethod {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ExpansionMode<'a> {
    Generic { mode_ident: &'a syn::Ident },
    Default,
    Moded { mode_ident: &'a syn::ExprPath },
}

impl ExpansionMode<'_> {
    pub(crate) fn as_mode<'a>(&'a self, tokens: &'a Tokens) -> Mode<'a> {
        match *self {
            ExpansionMode::Generic { mode_ident } => Mode {
                ident: None,
                mode_path: ModePath::Ident(mode_ident),
                tokens,
            },
            ExpansionMode::Default => Mode {
                ident: None,
                mode_path: ModePath::Path(&tokens.default_mode),
                tokens,
            },
            ExpansionMode::Moded { mode_ident } => Mode {
                ident: Some(mode_ident),
                mode_path: ModePath::Path(mode_ident),
                tokens,
            },
        }
    }

    /// Coerce into impl generics.
    pub(crate) fn as_impl_generics(
        &self,
        generics: syn::Generics,
        tokens: &Tokens,
    ) -> (syn::Generics, syn::ExprPath, Option<syn::WhereClause>) {
        match *self {
            ExpansionMode::Generic { mode_ident } => {
                let mut impl_generics = generics.clone();

                impl_generics
                    .params
                    .push(syn::TypeParam::from(mode_ident.clone()).into());

                let path = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: syn::Path::from(mode_ident.clone()),
                };

                let mut where_clause = syn::WhereClause {
                    where_token: <syn::Token![where]>::default(),
                    predicates: Default::default(),
                };

                let mut bounds: syn::punctuated::Punctuated<syn::TypeParamBound, _> =
                    Default::default();

                bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: Default::default(),
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: Default::default(),
                    path: tokens.mode_t.path.clone(),
                }));

                where_clause
                    .predicates
                    .push(syn::WherePredicate::Type(syn::PredicateType {
                        lifetimes: None,
                        bounded_ty: syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: syn::Path::from(mode_ident.clone()),
                        }),
                        colon_token: <syn::Token![:]>::default(),
                        bounds,
                    }));

                (impl_generics, path, Some(where_clause))
            }
            ExpansionMode::Default => (generics, tokens.default_mode.clone(), None),
            ExpansionMode::Moded { mode_ident } => (generics, mode_ident.clone(), None),
        }
    }
}

pub(crate) struct FieldData<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) name: Option<syn::LitStr>,
    pub(crate) attr: attr::FieldAttr,
    pub(crate) ident: Option<&'a syn::Ident>,
}

pub(crate) struct StructData<'a> {
    pub(crate) span: Span,
    pub(crate) fields: Vec<FieldData<'a>>,
}

pub(crate) struct VariantData<'a> {
    pub(crate) span: Span,
    pub(crate) name: syn::LitStr,
    pub(crate) index: usize,
    pub(crate) attr: attr::VariantAttr,
    pub(crate) ident: &'a syn::Ident,
    pub(crate) fields: Vec<FieldData<'a>>,
}

pub(crate) struct EnumData<'a> {
    pub(crate) span: Span,
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
    pub(crate) type_name: syn::LitStr,
    pub(crate) data: Data<'a>,
    pub(crate) tokens: Tokens,
}

#[derive(Clone, Copy)]
pub(crate) struct ExpanderWithMode<'a> {
    pub(crate) input: &'a syn::DeriveInput,
    pub(crate) cx: &'a Ctxt,
    pub(crate) type_attr: &'a TypeAttr,
    pub(crate) type_name: &'a syn::LitStr,
    pub(crate) data: &'a Data<'a>,
    pub(crate) tokens: &'a Tokens,
    pub(crate) mode: Mode<'a>,
}

impl<'a> Expander<'a> {
    pub(crate) fn new(input: &'a syn::DeriveInput) -> Self {
        fn fields<'a>(cx: &Ctxt, fields: &'a syn::Fields) -> Vec<FieldData<'a>> {
            fields
                .iter()
                .enumerate()
                .map(|(index, field)| FieldData {
                    span: field.span(),
                    index,
                    name: field
                        .ident
                        .as_ref()
                        .map(|ident| syn::LitStr::new(&ident.to_string(), ident.span()).into()),
                    attr: attr::field_attrs(&cx, &field.attrs),
                    ident: field.ident.as_ref(),
                })
                .collect()
        }

        let cx = Ctxt::new();
        let type_attr = attr::type_attrs(&cx, &input.attrs);
        let type_name = syn::LitStr::new(&input.ident.to_string(), input.ident.span());

        let data = match &input.data {
            syn::Data::Struct(st) => Data::Struct(StructData {
                span: Span::call_site(),
                fields: fields(&cx, &st.fields),
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
                    });

                Data::Enum(EnumData {
                    span: Span::call_site(),
                    variants: variants.collect(),
                })
            }
            syn::Data::Union(..) => Data::Union,
        };

        let prefix = type_attr.crate_or_default();

        Self {
            input,
            cx,
            type_attr,
            type_name,
            data,
            tokens: Tokens::new(input.ident.span(), &prefix),
        }
    }

    /// Coerce into errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        self.cx.into_errors()
    }

    /// Expand Encode implementation.
    pub(crate) fn expand_encode(&self) -> Option<TokenStream> {
        let modes = self.cx.modes();

        if modes.is_empty() {
            let mode_ident = syn::Ident::new("M", self.type_name.span());

            return crate::en::expand_encode_entry(
                self,
                ExpansionMode::Generic {
                    mode_ident: &mode_ident,
                },
            );
        }

        let mut out = TokenStream::new();

        for mode in modes {
            out.extend(crate::en::expand_encode_entry(
                self,
                ExpansionMode::Moded { mode_ident: &mode },
            )?);
        }

        out.extend(crate::en::expand_encode_entry(
            self,
            ExpansionMode::Default,
        )?);
        Some(out)
    }

    /// Expand Decode implementation.
    pub(crate) fn expand_decode(&self) -> Option<TokenStream> {
        let modes = self.cx.modes();

        if modes.is_empty() {
            let mode_ident = syn::Ident::new("M", self.type_name.span());

            return crate::de::expand_decode_entry(
                self,
                ExpansionMode::Generic {
                    mode_ident: &mode_ident,
                },
            );
        }

        let mut out = TokenStream::new();

        for mode in modes {
            out.extend(crate::de::expand_decode_entry(
                self,
                ExpansionMode::Moded { mode_ident: &mode },
            )?);
        }

        out.extend(crate::de::expand_decode_entry(
            self,
            ExpansionMode::Default,
        )?);
        Some(out)
    }
}

impl ExpanderWithMode<'_> {
    /// Emit diagnostics for a transparent encode / decode that failed because
    /// the wrong number of fields existed.
    pub(crate) fn transparent_diagnostics(&self, span: Span, fields: &[FieldData]) {
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

/// Expand the given configuration to the appropriate tag expression and
/// [TagMethod].
pub(crate) fn expand_tag(
    cx: &Ctxt,
    span: Span,
    rename: Option<&(Span, syn::Expr)>,
    default_field_tag: DefaultTag,
    index: usize,
    ident: Option<&syn::LitStr>,
) -> Option<(syn::Expr, TagMethod)> {
    let (lit, tag_method) = match (rename, default_field_tag, ident) {
        (Some((_, rename)), _, _) => {
            return Some((rename_lit(rename), determine_tag_method(rename)))
        }
        (None, DefaultTag::Index, _) => (usize_int(index, span).into(), TagMethod::Default),
        (None, DefaultTag::Name, None) => {
            cx.error_span(
                span,
                format!(
                    "#[{}({} = \"name\")] is not supported with unnamed fields",
                    ATTR, TAG
                ),
            );
            return None;
        }
        (None, DefaultTag::Name, Some(ident)) => (ident.clone().into(), TagMethod::String),
    };

    let tag = syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit,
    });

    Some((tag, tag_method))
}

/// Process rename literal to ensure it's always typed.
fn rename_lit(expr: &syn::Expr) -> syn::Expr {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) if int.suffix().is_empty() => syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: syn::LitInt::new(&format!("{}usize", int), int.span()).into(),
        }),
        expr => expr.clone(),
    }
}

/// Try and determine tag method from the given expression.
fn determine_tag_method(expr: &syn::Expr) -> TagMethod {
    let lit = match expr {
        syn::Expr::Lit(lit) => lit,
        _ => return TagMethod::Default,
    };

    match lit {
        syn::ExprLit {
            lit: syn::Lit::Str(..),
            ..
        } => TagMethod::String,
        _ => TagMethod::Default,
    }
}

/// Usize-suffixed integer.
pub(crate) fn usize_int(index: usize, span: Span) -> syn::LitInt {
    syn::LitInt::new(&format!("{}usize", index), span)
}

/// Integer used for tuple initialization.
pub(crate) fn field_int(index: usize, span: Span) -> syn::LitInt {
    syn::LitInt::new(&index.to_string(), span)
}

pub(crate) struct TagMethods<'a> {
    cx: &'a Ctxt,
    methods: BTreeSet<TagMethod>,
}

impl<'a> TagMethods<'a> {
    pub(crate) fn new(cx: &'a Ctxt) -> Self {
        Self {
            cx,
            methods: BTreeSet::new(),
        }
    }

    /// Insert a tag method and error in case it's invalid.
    pub(crate) fn insert(&mut self, span: Span, method: TagMethod) {
        let before = self.methods.len();
        self.methods.insert(method);

        if before == 1 && self.methods.len() > 1 {
            self.cx
                .error_span(span, format!("#[{}({})] conflicting tag kind", ATTR, TAG));
        }
    }

    /// Pick a tag method to use.
    pub(crate) fn pick(self) -> TagMethod {
        self.methods.into_iter().next().unwrap_or_default()
    }
}
