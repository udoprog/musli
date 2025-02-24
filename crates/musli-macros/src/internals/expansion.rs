use super::attr::{ModeIdent, ModeKind};
use super::mode::ModePath;
use super::{Mode, Only, Tokens};

#[derive(Clone, Copy)]
pub(crate) struct Expansion<'a> {
    pub(crate) mode_ident: &'a ModeIdent,
}

impl<'a> Expansion<'a> {
    pub(crate) fn as_mode(&self, tokens: &Tokens<'a>, only: Only) -> Mode<'a> {
        Mode {
            kind: Some(&self.mode_ident.kind),
            mode_path: self.mode_path(tokens),
            encode_packed_t: tokens.encode_packed_t,
            encode_bytes_t: tokens.encode_bytes_t,
            trace_encode_t: tokens.trace_encode_t,
            encode_t: tokens.encode_t,
            decode_packed_t: tokens.decode_packed_t,
            decode_bytes_t: tokens.decode_bytes_t,
            trace_decode_t: tokens.trace_decode_t,
            decode_t: tokens.decode_t,
            only,
        }
    }

    pub(crate) fn mode_path(&self, tokens: &Tokens<'a>) -> ModePath<'a> {
        match &self.mode_ident.kind {
            ModeKind::Custom => ModePath::Ident(&self.mode_ident.ident),
            _ => ModePath::Musli(tokens.prefix, &self.mode_ident.ident),
        }
    }
}
