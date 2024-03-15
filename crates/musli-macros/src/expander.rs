use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::internals::attr::{self, DefaultTag, TypeAttr};
use crate::internals::build::Build;
use crate::internals::symbol::*;
use crate::internals::tokens::Tokens;
use crate::internals::{Ctxt, Expansion, Mode, Only};

pub(crate) type Result<T, E = ()> = std::result::Result<T, E>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TagMethod {
    /// Special method that requires generating a visitor.
    String,
    /// The default tag method.
    Any,
}

impl Default for TagMethod {
    fn default() -> Self {
        Self::Any
    }
}

pub(crate) struct FieldData<'a> {
    pub(crate) span: Span,
    pub(crate) index: usize,
    pub(crate) name: Option<syn::LitStr>,
    pub(crate) attr: attr::Field,
    pub(crate) ident: Option<&'a syn::Ident>,
}

pub(crate) struct StructData<'a> {
    pub(crate) span: Span,
    pub(crate) name: syn::LitStr,
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
    pub(crate) tokens: Tokens,
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
                        .map(|ident| syn::LitStr::new(&ident.to_string(), ident.span())),
                    attr: attr::field_attrs(cx, &field.attrs),
                    ident: field.ident.as_ref(),
                })
                .collect()
        }

        let cx = Ctxt::new();
        let type_attr = attr::type_attrs(&cx, &input.attrs);

        let data = match &input.data {
            syn::Data::Struct(st) => Data::Struct(StructData {
                span: Span::call_site(),
                name: syn::LitStr::new(&input.ident.to_string(), input.ident.span()),
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
                    name: syn::LitStr::new(&input.ident.to_string(), input.ident.span()),
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
            data,
            tokens: Tokens::new(input.ident.span(), &prefix),
        }
    }

    /// Coerce into errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        self.cx.into_errors()
    }

    fn setup_builds<'b>(
        &'b self,
        modes: &'b [syn::Path],
        mode_ident: &'b syn::Ident,
        only: Only,
    ) -> Result<Vec<Build<'b>>> {
        let mut builds = Vec::new();

        if modes.is_empty() {
            builds.push(crate::internals::build::setup(
                self,
                Expansion::Generic { mode_ident },
                only,
            )?);
        } else {
            for mode_ident in modes {
                builds.push(crate::internals::build::setup(
                    self,
                    Expansion::Moded { mode_ident },
                    only,
                )?);
            }

            builds.push(crate::internals::build::setup(
                self,
                Expansion::Default,
                only,
            )?);
        }

        Ok(builds)
    }

    /// Expand Encode implementation.
    pub(crate) fn expand_encode(&self) -> Result<TokenStream> {
        let modes = self.cx.modes();
        let mode_ident = syn::Ident::new("M", Span::call_site());

        let builds = self.setup_builds(&modes, &mode_ident, Only::Encode)?;

        let mut out = TokenStream::new();

        for build in builds {
            out.extend(crate::en::expand_insert_entry(build)?);
        }

        Ok(out)
    }

    /// Expand Decode implementation.
    pub(crate) fn expand_decode(&self) -> Result<TokenStream> {
        let modes = self.cx.modes();
        let mode_ident = syn::Ident::new("M", Span::call_site());

        let builds = self.setup_builds(&modes, &mode_ident, Only::Decode)?;

        let mut out = TokenStream::new();

        for build in builds {
            out.extend(crate::de::expand_decode_entry(build)?);
        }

        Ok(out)
    }
}

/// A thing that determines how it's tagged.
pub(crate) trait Taggable {
    /// The span of the taggable item.
    fn span(&self) -> Span;
    /// The rename configuration the taggable item currently has.
    fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)>;
    /// The index of the taggable item.
    fn index(&self) -> usize;
    /// The string name of the taggable item.
    fn name(&self) -> Option<&syn::LitStr>;

    /// Expand the given configuration to the appropriate tag expression and
    /// [TagMethod].
    fn expand_tag(
        &self,
        e: &Expander<'_>,
        mode: Mode<'_>,
        default_tag: Option<DefaultTag>,
    ) -> Result<(syn::Expr, Option<TagMethod>)>
    where
        Self: Sized,
    {
        let (lit, tag_method) = match (self.rename(mode), default_tag, self.name()) {
            (Some((_, rename)), _, _) => {
                return Ok((rename_lit(rename), determine_tag_method(rename)))
            }
            (None, Some(DefaultTag::Index), _) => (
                usize_suffixed(self.index(), self.span()).into(),
                Some(TagMethod::Any),
            ),
            (None, Some(DefaultTag::Name), None) => {
                e.cx.error_span(
                    self.span(),
                    format_args!(
                        "#[{ATTR}({DEFAULT_FIELD_NAME} = \"name\")] is not supported with unnamed fields",
                    ),
                );
                return Err(());
            }
            (None, Some(DefaultTag::Name), Some(ident)) => {
                (ident.clone().into(), Some(TagMethod::String))
            }
            _ => (usize_suffixed(self.index(), self.span()).into(), None),
        };

        let tag = syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit,
        });

        Ok((tag, tag_method))
    }
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

    fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        self.attr.rename(mode)
    }

    fn index(&self) -> usize {
        self.index
    }

    fn name(&self) -> Option<&syn::LitStr> {
        self.name.as_ref()
    }
}

impl Taggable for VariantData<'_> {
    fn span(&self) -> Span {
        self.span
    }

    fn rename(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        self.attr.rename(mode)
    }

    fn index(&self) -> usize {
        self.index
    }

    fn name(&self) -> Option<&syn::LitStr> {
        Some(&self.name)
    }
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
pub(crate) fn determine_tag_method(expr: &syn::Expr) -> Option<TagMethod> {
    let lit = match expr {
        syn::Expr::Lit(lit) => lit,
        _ => return None,
    };

    match lit {
        syn::ExprLit {
            lit: syn::Lit::Str(..),
            ..
        } => Some(TagMethod::String),
        syn::ExprLit {
            lit: syn::Lit::Int(..),
            ..
        } => Some(TagMethod::Any),
        _ => None,
    }
}
