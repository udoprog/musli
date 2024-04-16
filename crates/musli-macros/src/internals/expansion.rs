use crate::internals::tokens::Tokens;
use crate::internals::{Mode, Only};

use super::mode::ModePath;

#[derive(Clone, Copy)]
pub(crate) enum Expansion<'a> {
    Generic { mode_ident: &'a syn::Ident },
    Default,
    Moded { mode_ident: &'a syn::Path },
}

impl<'a> Expansion<'a> {
    pub(crate) fn as_mode(&self, tokens: &'a Tokens, only: Only) -> Mode<'a> {
        match *self {
            Expansion::Generic { mode_ident, .. } => Mode {
                ident: None,
                mode_path: ModePath::Ident(mode_ident),
                tokens,
                only,
            },
            Expansion::Default => Mode {
                ident: None,
                mode_path: ModePath::Path(&tokens.binary_mode),
                tokens,
                only,
            },
            Expansion::Moded { mode_ident } => Mode {
                ident: Some(mode_ident),
                mode_path: ModePath::Path(mode_ident),
                tokens,
                only,
            },
        }
    }

    /// Coerce into impl generics.
    pub(crate) fn as_impl_generics(
        &self,
        mut generics: syn::Generics,
        tokens: &Tokens,
    ) -> (syn::Generics, syn::Path) {
        match *self {
            Expansion::Generic { mode_ident } => {
                generics
                    .params
                    .push(syn::TypeParam::from(mode_ident.clone()).into());

                let path = syn::Path::from(mode_ident.clone());
                (generics, path)
            }
            Expansion::Default => (generics, tokens.binary_mode.clone()),
            Expansion::Moded { mode_ident } => (generics, mode_ident.clone()),
        }
    }
}
