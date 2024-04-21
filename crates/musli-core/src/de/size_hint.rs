use core::fmt;

/// A length hint.
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum SizeHint {
    /// The length isn't known.
    #[default]
    Any,
    /// The length is exactly known.
    Exact(usize),
}

impl SizeHint {
    /// Get a size hint or a default value.
    pub fn or_default(self) -> usize {
        match self {
            SizeHint::Any => 0,
            SizeHint::Exact(n) => n,
        }
    }
}

impl From<Option<usize>> for SizeHint {
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(n) => SizeHint::Exact(n),
            None => SizeHint::Any,
        }
    }
}

impl fmt::Display for SizeHint {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeHint::Any => write!(f, "unknown length"),
            SizeHint::Exact(length) => write!(f, "{length} items"),
        }
    }
}

impl SizeHint {
    /// Coerce into a size hint.
    pub fn size_hint(self) -> usize {
        match self {
            SizeHint::Any => 0,
            SizeHint::Exact(len) => len,
        }
    }

    /// Coerce into an `Option`.
    pub fn into_option(self) -> Option<usize> {
        match self {
            SizeHint::Any => None,
            SizeHint::Exact(len) => Some(len),
        }
    }
}
