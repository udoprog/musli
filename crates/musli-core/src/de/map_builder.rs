use crate::Context;

use super::Decoder;

/// A builder for a map centered around decoders.
pub trait MapBuilder<'de, M, C>
where
    M: 'static,
    C: Context,
{
    /// The output of the decoder.
    type Output;

    /// Decode a field into the current map.
    ///
    /// Returns `true` if the field was decoded into the map.
    fn insert_field<A, B>(&mut self, key: A, value: B) -> Result<bool, C::Error>
    where
        A: Decoder<'de, Cx = C, Error = C::Error, Allocator = C::Allocator, Mode = M>,
        B: Decoder<'de, Cx = C, Error = C::Error, Allocator = C::Allocator, Mode = M>,
    {
        let _ = key;
        let _ = value;
        Ok(false)
    }

    /// Finish decoding and build the result.
    fn build(self) -> Result<Self::Output, C::Error>;
}
