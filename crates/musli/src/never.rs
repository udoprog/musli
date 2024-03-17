//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.
//!
//! This is a private module of musli, and is not intended for use outside of
//! the implementation attributes:
//!
//! * [`#[musli::encoder]`][crate::encoder].
//! * [`#[musli::decoder]`][crate::decoder].

use core::fmt;
use core::marker;

use crate::no_std::ToOwned;

use crate::de::{
    AsDecoder, Decoder, MapDecoder, MapEntriesDecoder, MapEntryDecoder, NumberVisitor, PackDecoder,
    SequenceDecoder, SizeHint, StructDecoder, StructFieldDecoder, StructFieldsDecoder,
    ValueVisitor, VariantDecoder,
};
use crate::en::{
    Encoder, MapEncoder, MapEntriesEncoder, MapEntryEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use crate::{Buf, Context};

/// An empty buffer.
pub enum NeverBuffer {}

impl Buf for NeverBuffer {
    #[inline(always)]
    fn write(&mut self, _: &[u8]) -> bool {
        false
    }

    #[inline(always)]
    fn len(&self) -> usize {
        0
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        &[]
    }
}

/// Marker type used for the [`Never`] type.
#[doc(hidden)]
pub enum NeverMarker {}

/// An uninhabitable never type which implements all possible encoders and
/// decoders. This can be used if your [Encoder] implementation doesn't
/// implement a particular function.
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::Decoder;
///
/// struct MyDecoder(u32);
///
/// #[musli::decoder]
/// impl<C: ?Sized + Context> Decoder<'_, C> for MyDecoder where {
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
///         if self.0 == 42 {
///             return Ok(self.0);
///         }
///
///         Err(cx.custom("I do not have the answer..."))
///     }
/// }
/// ```
pub struct Never<A = NeverMarker, B: ?Sized = NeverMarker> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<(A, B)>,
}

impl<'de, C: ?Sized + Context> Decoder<'de, C> for Never {
    type WithContext<U> = Self
    where
        U: Context;
    type DecodeBuffer = Self;
    type DecodePack = Self;
    type DecodeSequence = Self;
    type DecodeTuple = Self;
    type DecodeMap = Self;
    type DecodeSome = Self;
    type DecodeStruct = Self;
    type DecodeVariant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<C: ?Sized + Context> AsDecoder<C> for Never {
    type Decoder<'this> = Self;

    #[inline]
    fn as_decoder(&self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> StructFieldDecoder<'de, C> for Never {
    type DecodeFieldName<'this> = Self;
    type DecodeFieldValue = Self;

    #[inline]
    fn decode_field_name(&mut self, _: &C) -> Result<Self::DecodeFieldName<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_field_value(self, _: &C) -> Result<Self::DecodeFieldValue, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_field_value(self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapEntriesDecoder<'de, C> for Never {
    type DecodeMapEntryKey<'this> = Self;
    type DecodeMapEntryValue<'this> = Self;

    #[inline]
    fn decode_map_entry_key(
        &mut self,
        _: &C,
    ) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_map_entry_value(&mut self, _: &C) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_map_entry_value(&mut self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> StructDecoder<'de, C> for Never {
    type DecodeField<'this> = Self;
    type IntoStructFields = Self;

    type __UseMusliStructDecoderAttributeMacro = ();

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn into_struct_fields(self, _: &C) -> Result<Self::IntoStructFields, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_field(&mut self, _: &C) -> Result<Option<Self::DecodeField<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> StructFieldsDecoder<'de, C> for Never {
    type DecodeStructFieldName<'this> = Self;
    type DecodeStructFieldValue<'this> = Self;

    #[inline]
    fn decode_struct_field_name(
        &mut self,
        _: &C,
    ) -> Result<Self::DecodeStructFieldName<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_struct_field_value(
        &mut self,
        _: &C,
    ) -> Result<Self::DecodeStructFieldValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_struct_field_value(&mut self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> VariantDecoder<'de, C> for Never {
    type DecodeTag<'this> = Self;
    type DecodeVariant<'this> = Self;

    #[inline]
    fn decode_tag(&mut self, _: &C) -> Result<Self::DecodeTag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_value(&mut self, _: &C) -> Result<Self::DecodeVariant<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_value(&mut self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapDecoder<'de, C> for Never {
    type DecodeEntry<'this> = Self;
    type IntoMapEntries = Self;

    type __UseMusliMapDecoderAttributeMacro = ();

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn into_map_entries(self, _: &C) -> Result<Self::IntoMapEntries, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_entry(&mut self, _: &C) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapEntryDecoder<'de, C> for Never {
    type DecodeMapKey<'this> = Self;
    type DecodeMapValue = Self;

    #[inline]
    fn decode_map_key(&mut self, _: &C) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_map_value(self, _: &C) -> Result<Self::DecodeMapValue, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_map_value(self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> SequenceDecoder<'de, C> for Never {
    type DecodeNext<'this> = Self;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn decode_next(&mut self, _: &C) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> PackDecoder<'de, C> for Never {
    type DecodeNext<'this> = Self;

    #[inline]
    fn decode_next(&mut self, _: &C) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<C: ?Sized + Context, O: 'static> Encoder<C> for Never<O> {
    type Ok = O;
    type WithContext<U> = Self where U: Context;
    type EncodePack<'this> = Self where C: 'this;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeTuple = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeStruct = Self;
    type EncodeVariant = Self;
    type EncodeTupleVariant = Self;
    type EncodeStructVariant = Self;
    type __UseMusliEncoderAttributeMacro = ();

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, O, C: ?Sized + Context> NumberVisitor<'de, C> for Never<O> {
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, C, O: 'static, T> ValueVisitor<'de, C, T> for Never<O, T>
where
    C: ?Sized + Context,
    T: ?Sized + ToOwned,
{
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> SequenceEncoder<C> for Never<O> {
    type Ok = O;
    type EncodeNext<'this> = Self;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEncoder<C> for Never<O> {
    type Ok = O;
    type EncodeEntry<'this> = Self;

    #[inline]
    fn encode_entry(&mut self, _: &C) -> Result<Self::EncodeEntry<'_>, C::Error> {
        match self._never {}
    }

    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEntryEncoder<C> for Never<O> {
    type Ok = O;
    type EncodeMapKey<'this> = Self;
    type EncodeMapValue<'this> = Self;

    #[inline]
    fn encode_map_key(&mut self, _: &C) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_map_value(&mut self, _: &C) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEntriesEncoder<C> for Never<O> {
    type Ok = O;
    type EncodeMapEntryKey<'this> = Self;
    type EncodeMapEntryValue<'this> = Self;

    #[inline]
    fn encode_map_entry_key(&mut self, _: &C) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_map_entry_value(&mut self, _: &C) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> StructEncoder<C> for Never<O> {
    type Ok = O;
    type EncodeField<'this> = Self;

    #[inline]
    fn encode_field(&mut self, _: &C) -> Result<Self::EncodeField<'_>, C::Error> {
        match self._never {}
    }

    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> StructFieldEncoder<C> for Never<O> {
    type Ok = O;
    type EncodeFieldName<'this> = Self;
    type EncodeFieldValue<'this> = Self;

    #[inline]
    fn encode_field_name(&mut self, _: &C) -> Result<Self::EncodeFieldName<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_field_value(&mut self, _: &C) -> Result<Self::EncodeFieldValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> VariantEncoder<C> for Never<O> {
    type Ok = O;
    type EncodeTag<'this> = Self;
    type EncodeValue<'this> = Self;

    #[inline]
    fn encode_tag(&mut self, _: &C) -> Result<Self::EncodeTag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_value(&mut self, _: &C) -> Result<Self::EncodeValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}
