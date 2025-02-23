use super::Encoder;

/// Trait governing how types are encoded specifically for tracing.
///
/// This is used for types where some extra bounds might be necessary to trace a
/// container such as a [`HashMap<K, V>`] where `K` would have to implement
/// [`fmt::Display`].
///
/// [`HashMap<K, V>`]: std::collections::HashMap
/// [`fmt::Display`]: std::fmt::Display
pub trait EncodeTrace<M> {
    /// Encode the given output.
    fn trace_encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>;

    /// The number of fields in the type.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}
