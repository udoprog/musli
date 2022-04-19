pub(crate) enum NeedsKind {
    Unused,
    Used,
}

pub(crate) struct Needs {
    pub(crate) inline: bool,
    pub(crate) kind: NeedsKind,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            inline: true,
            kind: NeedsKind::Unused,
        }
    }
}

impl Needs {
    /// Mark that the implementation of the decode function should be inlined.
    pub(crate) fn mark_inline(&mut self) {
        self.inline = true;
    }

    /// Mark that the decoder is used.
    pub(crate) fn mark_used(&mut self) {
        if let NeedsKind::Unused = self.kind {
            self.kind = NeedsKind::Used;
        }
    }
}
