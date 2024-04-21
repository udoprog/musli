/// A hint passed in when encoding a sequence.
#[non_exhaustive]
pub struct SequenceHint {
    /// The size for the sequence being encoded.
    pub size: usize,
}

impl SequenceHint {
    /// Create a new sequence hint with the specified size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::hint::SequenceHint;
    ///
    /// static HINT: SequenceHint = SequenceHint::with_size(16);
    ///
    /// assert_eq!(HINT.size, 16);
    /// ```
    #[inline]
    pub const fn with_size(size: usize) -> Self {
        Self { size }
    }
}
