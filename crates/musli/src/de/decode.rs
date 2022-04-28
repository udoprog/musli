use crate::{de::Decoder, mode::DefaultMode};
pub use musli_macros::Decode;

/// Trait governing how types are decoded.
pub trait Decode<'de, Mode = DefaultMode>: Sized {
    /// Decode the given input.
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>;
}
