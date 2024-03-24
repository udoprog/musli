use crate::Context;

use super::{Encode, StructFieldEncoder};

/// Encoder for a struct.
pub trait StructEncoder {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context;
    /// Result type of the encoder.
    type Ok;
    /// Encoder for the next struct field.
    type EncodeStructField<'this>: StructFieldEncoder<Cx = Self::Cx, Ok = Self::Ok>
    where
        Self: 'this;

    /// Encode the next field.
    fn encode_struct_field(
        &mut self,
    ) -> Result<Self::EncodeStructField<'_>, <Self::Cx as Context>::Error>;

    /// Finish encoding the struct.
    fn end_struct(self) -> Result<Self::Ok, <Self::Cx as Context>::Error>;

    /// Encode the next field using a closure.
    #[inline]
    fn encode_struct_field_fn<F, O>(&mut self, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodeStructField<'_>) -> Result<O, <Self::Cx as Context>::Error>,
    {
        let mut encoder = self.encode_struct_field()?;
        let output = f(&mut encoder)?;
        encoder.end_field()?;
        Ok(output)
    }

    /// Insert a field immediately.
    #[inline]
    fn insert_struct_field<F, V>(
        &mut self,
        field: F,
        value: V,
    ) -> Result<(), <Self::Cx as Context>::Error>
    where
        F: Encode<<Self::Cx as Context>::Mode>,
        V: Encode<<Self::Cx as Context>::Mode>,
    {
        self.encode_struct_field()?.insert_field(field, value)?;
        Ok(())
    }
}
