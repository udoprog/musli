/// Indicates if skipping was performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Skip {
    /// Indicates that skipping was not supported.
    Unsupported,
    /// Indicates that skipping was successfully performed.
    Skipped,
}

impl Skip {
    /// Indicates if a skip was not supported.
    #[inline(always)]
    pub fn is_unsupported(self) -> bool {
        self == Skip::Unsupported
    }

    /// Indicates if a skip was performed.
    #[inline(always)]
    pub fn is_skipped(self) -> bool {
        self == Skip::Skipped
    }
}
