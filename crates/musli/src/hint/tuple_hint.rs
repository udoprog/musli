/// A hint passed in when encoding a tuple.
#[non_exhaustive]
pub struct TupleHint {
    /// The size for the tuple being encoded.
    pub size: usize,
}

impl TupleHint {
    /// Create a new tuple hint with the specified size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::hint::TupleHint;
    ///
    /// static HINT: TupleHint = TupleHint::with_size(16);
    ///
    /// assert_eq!(HINT.size, 16);
    /// ```
    #[inline]
    pub const fn with_size(size: usize) -> Self {
        Self { size }
    }
}
