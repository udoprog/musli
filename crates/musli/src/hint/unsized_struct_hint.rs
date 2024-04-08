/// A hint passed in when decoding an unsized struct.
#[non_exhaustive]
pub struct UnsizedStructHint {}

impl UnsizedStructHint {
    /// Construct a new empty hint.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::hint::UnsizedStructHint;
    ///
    /// static HINT: UnsizedStructHint = UnsizedStructHint::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}
