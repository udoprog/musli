use crate::Context;

use super::Decoder;

/// Trait governing how to decode a sequence of struct pairs.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde deserialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait StructFieldsDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for a tuple field index.
    type DecodeStructFieldName<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeStructFieldValue<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Try to return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "Decoders must be consumed"]
    fn decode_struct_field_name(
        &mut self,
    ) -> Result<Self::DecodeStructFieldName<'_>, <Self::Cx as Context>::Error>;

    /// Decode the second value in the pair..
    #[must_use = "Decoders must be consumed"]
    fn decode_struct_field_value(
        &mut self,
    ) -> Result<Self::DecodeStructFieldValue<'_>, <Self::Cx as Context>::Error>;

    /// End struct fields decoding.
    fn end_struct_fields(self) -> Result<(), <Self::Cx as Context>::Error>;
}
