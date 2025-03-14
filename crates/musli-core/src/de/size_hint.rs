use core::fmt;

#[derive(Default, Debug, Clone, Copy)]
enum SizeHintKind {
    /// The length isn't known.
    #[default]
    Any,
    /// The length is exactly known.
    Exact(usize),
}

/// A length hint.
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
#[doc(hidden)]
pub struct SizeHint {
    kind: SizeHintKind,
}

impl SizeHint {
    /// Construct a size hint of unknown size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::SizeHint;
    ///
    /// let hint = SizeHint::any();
    /// assert_eq!(hint.or_default(), 0);
    /// ```
    #[inline]
    pub const fn any() -> Self {
        SizeHint {
            kind: SizeHintKind::Any,
        }
    }

    /// Construct an exactly sized hint.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::SizeHint;
    ///
    /// let hint = SizeHint::exact(16);
    /// assert_eq!(hint.or_default(), 16);
    /// ```
    #[inline]
    pub const fn exact(length: usize) -> Self {
        SizeHint {
            kind: SizeHintKind::Exact(length),
        }
    }

    /// Get a size hint or a default value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::SizeHint;
    ///
    /// let hint = SizeHint::any();
    /// assert_eq!(hint.or_default(), 0);
    /// ```
    #[inline]
    pub fn or_default(self) -> usize {
        match self.kind {
            SizeHintKind::Any => 0,
            SizeHintKind::Exact(n) => n,
        }
    }
}

impl From<Option<usize>> for SizeHint {
    #[inline]
    fn from(value: Option<usize>) -> Self {
        let kind = match value {
            Some(n) => SizeHintKind::Exact(n),
            None => SizeHintKind::Any,
        };

        SizeHint { kind }
    }
}

impl fmt::Display for SizeHint {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SizeHintKind::Any => write!(f, "unknown length"),
            SizeHintKind::Exact(length) => write!(f, "{length} items"),
        }
    }
}

impl SizeHint {
    /// Coerce into an `Option`.
    #[inline]
    pub fn into_option(self) -> Option<usize> {
        match self.kind {
            SizeHintKind::Any => None,
            SizeHintKind::Exact(len) => Some(len),
        }
    }
}
