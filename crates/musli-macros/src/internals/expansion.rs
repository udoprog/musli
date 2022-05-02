use crate::internals::tokens::Tokens;
use crate::internals::{Mode, ModePath};

#[derive(Clone, Copy)]
pub(crate) enum Expansion<'a> {
    Generic { mode_ident: &'a syn::Ident },
    Default,
    Moded { mode_ident: &'a syn::ExprPath },
}

impl<'a> Expansion<'a> {
    pub(crate) fn as_mode(&self, tokens: &'a Tokens) -> Mode<'a> {
        match *self {
            Expansion::Generic { mode_ident } => Mode {
                ident: None,
                mode_path: ModePath::Ident(mode_ident),
                tokens,
            },
            Expansion::Default => Mode {
                ident: None,
                mode_path: ModePath::Path(&tokens.default_mode),
                tokens,
            },
            Expansion::Moded { mode_ident } => Mode {
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
            Expansion::Generic { mode_ident } => {
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
            Expansion::Default => (generics, tokens.default_mode.clone(), None),
            Expansion::Moded { mode_ident } => (generics, mode_ident.clone(), None),
        }
    }
}
