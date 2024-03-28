use crate::Context;

use super::Decoder;

/// Trait governing how to decode a struct field.
pub trait StructFieldDecoder<'de> {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for a tuple field index.
    type DecodeFieldName<'this>: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The decoder to use for a tuple field value.
    type DecodeFieldValue: Decoder<
        'de,
        Cx = Self::Cx,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >;

    /// Return the decoder for the field name.
    #[must_use = "Decoders must be consumed"]
    fn decode_field_name(
        &mut self,
    ) -> Result<Self::DecodeFieldName<'_>, <Self::Cx as Context>::Error>;

    /// Decode the field value.
    #[must_use = "Decoders must be consumed"]
    fn decode_field_value(self) -> Result<Self::DecodeFieldValue, <Self::Cx as Context>::Error>;
}
