use crate::Context;

use super::{Encode, Encoder};

/// Trait governing how to encode a sequence of pairs.
pub trait StructFieldEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// The encoder returned when advancing the map encoder to encode the key.
    type EncodeFieldName<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;
    /// The encoder returned when advancing the map encoder to encode the value.
    type EncodeFieldValue<'this>: Encoder<
        Cx = Self::Cx,
        Ok = Self::Ok,
        Error = <Self::Cx as Context>::Error,
        Mode = <Self::Cx as Context>::Mode,
    >
    where
        Self: 'this;

    /// Return the encoder for the field in the struct.
    #[must_use = "Encoders must be consumed"]
    fn encode_field_name(
        &mut self,
    ) -> Result<Self::EncodeFieldName<'_>, <Self::Cx as Context>::Error>;

    /// Return encoder for the field value in the struct.
    #[must_use = "Encoders must be consumed"]
    fn encode_field_value(
        &mut self,
    ) -> Result<Self::EncodeFieldValue<'_>, <Self::Cx as Context>::Error>;

    /// Stop encoding this field.
    fn finish_field(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Insert the pair immediately.
    #[inline]
    fn insert_field<N, V>(
        mut self,
        name: N,
        value: V,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        Self: Sized,
        N: Encode<<Self::Cx as Context>::Mode>,
        V: Encode<<Self::Cx as Context>::Mode>,
    {
        self.encode_field_name()?.encode(name)?;
        self.encode_field_value()?.encode(value)?;
        self.finish_field()
    }
}
