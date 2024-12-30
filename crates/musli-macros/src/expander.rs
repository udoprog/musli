use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::Token;

use crate::internals::attr::{self, ModeIdent, ModeKind, TypeAttr};
use crate::internals::build::Build;
use crate::internals::name::NameAll;
use crate::internals::tokens::Tokens;
use crate::internals::{Ctxt, Expansion, Mode, Only, Result};

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

pub(crate) struct NameType {
    pub(crate) ty: syn::Type,
    pub(crate) method: NameMethod,
}

impl NameType {
    pub(crate) fn expr(&self, ident: syn::Ident) -> syn::Expr {
        match self.method {
            NameMethod::Unsized(..) => syn::parse_quote!(#ident),
            NameMethod::Value => syn::parse_quote!(&#ident),
        }
    }

    pub(crate) fn ty(&self) -> syn::Type {
        match self.method {
            NameMethod::Unsized(..) => syn::Type::Reference(syn::TypeReference {
                and_token: <Token![&]>::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(self.ty.clone()),
            }),
            NameMethod::Value => self.ty.clone(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) enum NameMethod {
    /// Load the tag by value.
    #[default]
    Value,
    /// Load the tag by visit.
    Unsized(UnsizedMethod),
}

impl NameMethod {
    pub(crate) fn name_all(&self) -> Option<NameAll> {
        match self {
            Self::Value => None,
            Self::Unsized(_) => Some(NameAll::Name),
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
    pub(crate) span: Span,
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
                span: Span::call_site(),
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
                    span: Span::call_site(),
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
                ident: syn::Ident::new("Binary", Span::call_site()),
            },
            ModeIdent {
                kind: ModeKind::Text,
                ident: syn::Ident::new("Text", Span::call_site()),
            },
        ];

        Self {
            input,
            cx,
            type_attr,
            data,
            tokens: Tokens::new(input.ident.span(), prefix),
            default,
        }
    }

    /// Coerce into errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        self.cx.into_errors()
    }

    fn setup_builds<'b>(&'b self, modes: &'b [ModeIdent], only: Only) -> Result<Vec<Build<'b>>> {
        let mut builds = Vec::new();

        let mut missing = BTreeMap::new();

        for default in &self.default {
            missing.insert(&default.kind, default);
        }

        for mode_ident in modes {
            missing.remove(&mode_ident.kind);

            builds.push(crate::internals::build::setup(
                self,
                Expansion { mode_ident },
                only,
            )?);
        }

        for (_, mode_ident) in missing {
            builds.push(crate::internals::build::setup(
                self,
                Expansion { mode_ident },
                only,
            )?);
        }

        Ok(builds)
    }

    /// Expand Encode implementation.
    pub(crate) fn expand_encode(&self) -> Result<TokenStream> {
        let modes = self.cx.modes();
        let builds = self.setup_builds(&modes, Only::Encode)?;

        let mut out = TokenStream::new();

        for build in builds {
            out.extend(crate::en::expand_insert_entry(build)?);
        }

        Ok(out)
    }

    /// Expand Decode implementation.
    pub(crate) fn expand_decode(&self) -> Result<TokenStream> {
        let modes = self.cx.modes();
        let builds = self.setup_builds(&modes, Only::Decode)?;

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
    fn name(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)>;
    /// The index of the taggable item.
    fn index(&self) -> usize;
}

/// Expand the given configuration to the appropriate tag expression.
pub(crate) fn expand_name(
    taggable: &dyn Taggable,
    mode: Mode<'_>,
    name_all: NameAll,
    ident: Option<&syn::Ident>,
) -> syn::Expr {
    let lit = 'out: {
        if let Some((_, rename)) = taggable.name(mode) {
            return rename.clone();
        }

        if let (Some(ident), name_all) = (ident, name_all) {
            if let Some(name) = name_all.apply(&ident.to_string()) {
                break 'out syn::LitStr::new(&name, ident.span()).into();
            }
        }

        usize_suffixed(taggable.index(), taggable.span()).into()
    };

    syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit,
    })
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

    fn name(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
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

    fn name(&self, mode: Mode<'_>) -> Option<&(Span, syn::Expr)> {
        self.attr.name(mode)
    }

    fn index(&self) -> usize {
        self.index
    }
}
