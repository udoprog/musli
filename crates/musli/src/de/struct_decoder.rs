use crate::Context;

use super::{Decoder, SizeHint, StructFieldDecoder, StructFieldsDecoder};

/// Trait governing how to decode fields in a struct.
pub trait StructDecoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context;
    /// The decoder to use for a key.
    type DecodeField<'this>: StructFieldDecoder<'de, Cx = Self::Cx>
    where
        Self: 'this;
    /// Decoder for a sequence of struct pairs.
    type IntoStructFields: StructFieldsDecoder<'de, Cx = Self::Cx>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::struct_decoder]`][crate::struct_decoder]
    /// attribute macro when implementing [`MapDecoder`].
    #[doc(hidden)]
    type __UseMusliStructDecoderAttributeMacro;

    /// Get a size hint of known remaining fields.
    fn size_hint(&self, cx: &Self::Cx) -> SizeHint;

    /// Decode the next field.
    #[must_use = "Decoders must be consumed"]
    fn decode_field(
        &mut self,
        cx: &Self::Cx,
    ) -> Result<Option<Self::DecodeField<'_>>, <Self::Cx as Context>::Error>;

    /// End the struct decoder.
    ///
    /// If there are any remaining elements in the sequence of pairs, this
    /// indicates that they should be flushed.
    fn end(mut self, cx: &Self::Cx) -> Result<(), <Self::Cx as Context>::Error> {
        while let Some(mut item) = self.decode_field(cx)? {
            item.decode_field_name(cx)?.skip(cx)?;
            item.skip_field_value(cx)?;
        }

        Ok(())
    }

    /// Simplified decoding of a struct which has an expected `len` number of
    /// elements.
    ///
    /// The `len` indicates how many fields the decoder is *expecting* depending
    /// on how many fields are present in the underlying struct being decoded,
    /// butit should only be considered advisory.
    ///
    /// The size of a struct might therefore change from one session to another.
    #[inline]
    fn into_struct_fields(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::IntoStructFields, <Self::Cx as Context>::Error>
    where
        Self: Sized,
    {
        Err(cx.message("Decoder does not support StructPairs decoding"))
    }
}
