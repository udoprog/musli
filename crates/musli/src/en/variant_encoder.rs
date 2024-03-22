use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a variant.
pub trait VariantEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeTag<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeValue<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return the encoder for the first element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn encode_tag(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::EncodeTag<'_>, <Self::Cx as Context>::Error>;

    /// Return encoder for the second element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn encode_value(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Self::EncodeValue<'_>, <Self::Cx as Context>::Error>;

    /// End the variant encoder.
    fn end(self, cx: &Self::Cx) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Insert the variant immediately.
    #[inline]
    fn insert_variant<T, V>(
        mut self,
        cx: &Self::Cx,
        tag: T,
        value: V,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        T: Encode<<Self::Cx as Context>::Mode>,
        V: Encode<<Self::Cx as Context>::Mode>,
    {
        tag.encode(cx, self.encode_tag(cx)?)?;
        value.encode(cx, self.encode_value(cx)?)?;
        self.end(cx)
    }
}
