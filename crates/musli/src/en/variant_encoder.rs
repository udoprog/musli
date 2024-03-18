use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a variant.
pub trait VariantEncoder<C: ?Sized + Context> {
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeTag<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeValue<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Return the encoder for the first element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn encode_tag(&mut self, cx: &C) -> Result<Self::EncodeTag<'_>, C::Error>;

    /// Return encoder for the second element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn encode_value(&mut self, cx: &C) -> Result<Self::EncodeValue<'_>, C::Error>;

    /// End the variant encoder.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;

    /// Insert the variant immediately.
    #[inline]
    fn insert_variant<T, V>(mut self, cx: &C, tag: T, value: V) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        T: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        tag.encode(cx, self.encode_tag(cx)?)?;
        value.encode(cx, self.encode_value(cx)?)?;
        self.end(cx)
    }
}
