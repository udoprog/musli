use crate::Allocator;

use super::Decoder;

/// Trait governing how types are decoded specifically for tracing.
///
/// This is used for types where some extra bounds might be necessary to trace a
/// container such as a [`HashMap<K, V>`] where `K` would have to implement
/// [`fmt::Display`].
///
/// [`HashMap<K, V>`]: std::collections::HashMap
/// [`fmt::Display`]: std::fmt::Display
pub trait DecodeTrace<'de, M, A>
where
    Self: Sized,
    A: Allocator,
{
    /// Decode the given input.
    fn trace_decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>;
}
