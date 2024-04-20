use crate::internals::tokens::Tokens;
use crate::internals::{Mode, Only};

use super::attr::{ModeIdent, ModeKind};
use super::mode::ModePath;

#[derive(Clone, Copy)]
pub(crate) enum Expansion<'a> {
    // TODO: Should this be removed or made to work?
    #[allow(unused)]
    Generic {
        mode_ident: &'a syn::Ident,
    },
    Moded {
        mode_ident: &'a ModeIdent,
    },
}

impl<'a> Expansion<'a> {
    pub(crate) fn as_mode(&'a self, tokens: &'a Tokens, only: Only) -> Mode<'a> {
        match *self {
            Expansion::Generic { mode_ident, .. } => Mode {
                kind: None,
                mode_path: ModePath::Ident(mode_ident),
                tokens,
                only,
            },
            Expansion::Moded { mode_ident } => Mode {
                kind: Some(&mode_ident.kind),
                mode_path: match &mode_ident.kind {
                    ModeKind::Custom(..) => ModePath::Ident(&mode_ident.ident),
                    _ => ModePath::Musli(&tokens.prefix, &mode_ident.ident),
                },
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
        let mode_path = match *self {
            Expansion::Generic { mode_ident } => {
                generics
                    .params
                    .push(syn::TypeParam::from(mode_ident.clone()).into());

                let path = syn::Path::from(mode_ident.clone());
                return (generics, path);
            }
            Expansion::Moded { mode_ident } => match &mode_ident.kind {
                ModeKind::Custom(..) => ModePath::Ident(&mode_ident.ident),
                _ => ModePath::Musli(&tokens.prefix, &mode_ident.ident),
            },
        };

        (generics, mode_path.as_path())
    }
}
