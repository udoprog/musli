use crate::{Allocator, Context};

use super::Decoder;

/// Trait that allows a type to be repeatedly coerced into a decoder.
pub trait IntoDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: Context<Error = Self::Error, Allocator = Self::Allocator>;
    /// Error associated with decoding.
    type Error;
    /// The allocator associated with the decoder.
    type Allocator: Allocator;
    /// The mode of the decoder.
    type Mode: 'static;
    /// The decoder we reborrow as.
    type Decoder: Decoder<
            'de,
            Cx = Self::Cx,
            Error = Self::Error,
            Allocator = Self::Allocator,
            Mode = Self::Mode,
        >;

    /// Borrow self as a new decoder.
    fn into_decoder(&self) -> Result<Self::Decoder, Self::Error>;
}
