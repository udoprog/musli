use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a variant.
#[must_use = "Must call end_variant to finish encoding"]
pub trait VariantEncoder: Sized {
    /// Context associated with the encoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with encoding.
    type Error;
    /// The mode of the encoder.
    type Mode: 'static;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeTag<'this>: Encoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeData<'this>: Encoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>
    where
        Self: 'this;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// Return the encoder for the first element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, Self::Error>;

    /// Return encoder for the second element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, Self::Error>;

    /// End the variant encoder.
    fn finish_variant(self) -> Result<(), Self::Error>;

    /// Insert the variant immediately.
    #[inline]
    fn insert_variant<T, V>(mut self, tag: T, value: V) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
        V: Encode<Self::Mode>,
    {
        self.encode_tag()?.encode(tag)?;
        self.encode_data()?.encode(value)?;
        self.finish_variant()
    }
}
