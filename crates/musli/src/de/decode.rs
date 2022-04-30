use crate::de::Decoder;
use crate::mode::{DefaultMode, Mode};
pub use musli_macros::Decode;

/// Trait governing how types are decoded.
pub trait Decode<'de, M = DefaultMode>: Sized
where
    M: Mode,
{
    /// Decode the given input.
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>;
}
