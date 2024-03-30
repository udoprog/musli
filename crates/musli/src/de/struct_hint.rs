/// A hint passed in when decoding a struct.
#[non_exhaustive]
pub struct StructHint {
    /// The number of fields the decoded struct will have, excluding skipped
    /// fields.
    pub fields: usize,
}
