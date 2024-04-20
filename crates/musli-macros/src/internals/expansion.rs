use crate::internals::tokens::Tokens;
use crate::internals::{Mode, Only};

use super::attr::{ModeIdent, ModeKind};
use super::mode::ModePath;

#[derive(Clone, Copy)]
pub(crate) struct Expansion<'a> {
    pub(crate) mode_ident: &'a ModeIdent,
}

impl<'a> Expansion<'a> {
    pub(crate) fn as_mode(&'a self, tokens: &'a Tokens, only: Only) -> Mode<'a> {
        Mode {
            kind: Some(&self.mode_ident.kind),
            mode_path: self.mode_path(tokens),
            tokens,
            only,
        }
    }

    pub(crate) fn mode_path(&self, tokens: &'a Tokens) -> ModePath<'a> {
        match &self.mode_ident.kind {
            ModeKind::Custom(..) => ModePath::Ident(&self.mode_ident.ident),
            _ => ModePath::Musli(&tokens.prefix, &self.mode_ident.ident),
        }
    }
}
