use crate::de::Decoder;
use crate::mode::DefaultMode;
use crate::Context;

/// Please refer to the main [musli documentation](https://docs.rs/musli).
#[doc(inline)]
pub use musli_macros::Decode;

/// Trait governing how types are decoded.
pub trait Decode<'de, M = DefaultMode>: Sized {
    /// Decode the given input.
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>;
}

/// Trait governing how types are decoded specifically for tracing.
///
/// This is used for types where some extra bounds might be necessary to trace a
/// container such as a [`HashMap<K, V>`] where `K` would have to implement
/// [`fmt::Display`].
///
/// [`HashMap<K, V>`]: std::collections::HashMap
/// [`fmt::Display`]: std::fmt::Display
pub trait TraceDecode<'de, M = DefaultMode>: Sized {
    /// Decode the given input.
    fn trace_decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>;
}
