/// A hint passed in when decoding an unsized struct.
#[non_exhaustive]
pub struct UnsizedMapHint {}

impl UnsizedMapHint {
    /// Construct a new empty hint.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::hint::UnsizedMapHint;
    ///
    /// static HINT: UnsizedMapHint = UnsizedMapHint::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}
