use crate::Context;

use super::{SizeHint, StructFieldDecoder};

/// Trait governing how to decode fields in a struct.
pub trait StructDecoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for a key.
    type DecodeField<'this>: StructFieldDecoder<'de, Cx = Self::Cx>
    where
        Self: 'this;

    /// Get a size hint of known remaining fields.
    fn size_hint(&self) -> SizeHint;

    /// Decode the next field.
    #[must_use = "Decoders must be consumed"]
    fn decode_field(
        &mut self,
    ) -> Result<Option<Self::DecodeField<'_>>, <Self::Cx as Context>::Error>;
}
