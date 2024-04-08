/// A hint passed in when encoding a struct.
///
/// This requires that the struct has a known size.
#[non_exhaustive]
pub struct StructHint {
    /// The size for the struct being encoded.
    pub size: usize,
}

impl StructHint {
    /// Create a new struct hint with the specified size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::hint::StructHint;
    ///
    /// static HINT: StructHint = StructHint::with_size(16);
    ///
    /// assert_eq!(HINT.size, 16);
    /// ```
    #[inline]
    pub const fn with_size(size: usize) -> Self {
        Self { size }
    }
}
