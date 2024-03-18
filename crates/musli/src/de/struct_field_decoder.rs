use crate::Context;

use super::Decoder;

/// Trait governing how to decode a struct field.
pub trait StructFieldDecoder<'de, C: ?Sized + Context> {
    /// The decoder to use for a tuple field index.
    type DecodeFieldName<'this>: Decoder<'de, C>
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeFieldValue: Decoder<'de, C>;

    /// Return the decoder for the field name.
    #[must_use = "Decoders must be consumed"]
    fn decode_field_name(&mut self, cx: &C) -> Result<Self::DecodeFieldName<'_>, C::Error>;

    /// Decode the field value.
    #[must_use = "Decoders must be consumed"]
    fn decode_field_value(self, cx: &C) -> Result<Self::DecodeFieldValue, C::Error>;

    /// Indicate that the field value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_field_value(self, cx: &C) -> Result<bool, C::Error>;
}
