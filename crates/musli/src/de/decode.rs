use crate::de::Decoder;
use crate::mode::{DefaultMode, Mode};
use crate::Context;

/// Please refer to the main [musli documentation](https://docs.rs/musli).
#[doc(inline)]
pub use musli_macros::Decode;

/// Trait governing how types are decoded.
pub trait Decode<'de, M = DefaultMode>: Sized
where
    M: Mode,
{
    /// Decode the given input.
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>;
}
