use crate::Context;

use super::{Encode, StructFieldEncoder};

/// Encoder for a struct.
pub trait StructEncoder<C: ?Sized + Context> {
    /// Result type of the encoder.
    type Ok;
    /// Encoder for the next struct field.
    type EncodeField<'this>: StructFieldEncoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Encode the next field.
    fn encode_field(&mut self, cx: &C) -> Result<Self::EncodeField<'_>, C::Error>;

    /// Finish encoding the struct.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;

    /// Insert a field immediately.
    #[inline]
    fn insert_field<F, V>(&mut self, cx: &C, field: F, value: V) -> Result<(), C::Error>
    where
        F: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        self.encode_field(cx)?.insert_field(cx, field, value)?;
        Ok(())
    }
}
