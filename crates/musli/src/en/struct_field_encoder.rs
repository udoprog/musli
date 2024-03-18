use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a sequence of pairs.
pub trait StructFieldEncoder<C: ?Sized + Context> {
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeFieldName<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeFieldValue<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Return the encoder for the field in the struct.
    #[must_use = "Encoders must be consumed"]
    fn encode_field_name(&mut self, cx: &C) -> Result<Self::EncodeFieldName<'_>, C::Error>;

    /// Return encoder for the field value in the struct.
    #[must_use = "Encoders must be consumed"]
    fn encode_field_value(&mut self, cx: &C) -> Result<Self::EncodeFieldValue<'_>, C::Error>;

    /// Stop encoding this field.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_field<N, V>(mut self, cx: &C, name: N, value: V) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        N: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        name.encode(cx, self.encode_field_name(cx)?)?;
        value.encode(cx, self.encode_field_value(cx)?)?;
        self.end(cx)
    }
}
