use crate::Context;

use super::Decoder;

/// Trait governing how to decode a sequence of struct pairs.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde deserialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait StructFieldsDecoder<'de, C: ?Sized + Context> {
    /// The decoder to use for a tuple field index.
    type DecodeStructFieldName<'this>: Decoder<'de, C>
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeStructFieldValue<'this>: Decoder<'de, C>
    where
        Self: 'this;

    /// Try to return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_struct_field_name(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeStructFieldName<'_>, C::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_struct_field_value(
        &mut self,
        cx: &C,
    ) -> Result<Self::DecodeStructFieldValue<'_>, C::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_struct_field_value(&mut self, cx: &C) -> Result<bool, C::Error>;

    /// End pair decoding.
    fn end(self, cx: &C) -> Result<(), C::Error>;
}
