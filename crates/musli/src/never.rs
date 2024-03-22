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
/// use std::marker::PhantomData;
///
/// use musli::Context;
/// use musli::de::Decoder;
///
/// struct MyDecoder<C: ?Sized>(u32, PhantomData<C>);
///
/// #[musli::decoder]
/// impl<C: ?Sized + Context> Decoder<'_> for MyDecoder<C> where {
///     type Cx = C;
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
///         if self.0 == 42 {
///             return Ok(self.0);
///         }
///
///         Err(cx.message("I do not have the answer..."))
///     }
/// }
/// ```
pub struct Never<A = NeverMarker, B: ?Sized = NeverMarker> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<(A, B)>,
}

impl<'de, C: ?Sized + Context> Decoder<'de> for Never<(), C> {
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<U> = Never<(), U>
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

impl<C: ?Sized + Context> AsDecoder for Never<(), C> {
    type Cx = C;
    type Decoder<'this> = Self where Self: 'this;

    #[inline]
    fn as_decoder(&self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> StructFieldDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeFieldName<'this> = Self where Self: 'this;
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

impl<'de, C: ?Sized + Context> MapEntriesDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeMapEntryKey<'this> = Self where Self: 'this;
    type DecodeMapEntryValue<'this> = Self where Self: 'this;

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

impl<'de, C: ?Sized + Context> StructDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeField<'this> = Self where Self: 'this;
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

impl<'de, C: ?Sized + Context> StructFieldsDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeStructFieldName<'this> = Self where Self: 'this;
    type DecodeStructFieldValue<'this> = Self where Self: 'this;

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

impl<'de, C: ?Sized + Context> VariantDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeTag<'this> = Self where Self: 'this;
    type DecodeVariant<'this> = Self where Self: 'this;

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

impl<'de, C: ?Sized + Context> MapDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeEntry<'this> = Self where Self: 'this;
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

impl<'de, C: ?Sized + Context> MapEntryDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeMapKey<'this> = Self where Self: 'this;
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

impl<'de, C: ?Sized + Context> SequenceDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeNext<'this> = Self where Self: 'this;

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

impl<'de, C: ?Sized + Context> PackDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeNext<'this> = Self where Self: 'this;

    #[inline]
    fn decode_next(&mut self, _: &C) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<C: ?Sized + Context, O: 'static> Encoder for Never<O, C> {
    type Cx = C;
    type Error = C::Error;
    type Ok = O;
    type Mode = C::Mode;
    type WithContext<U> = Never<O, U> where U: Context;
    type EncodePack<'this> = Self where Self::Cx: 'this;
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

impl<O: 'static, C: ?Sized + Context> SequenceEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeNext<'this> = Self where Self: 'this;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeEntry<'this> = Self where Self: 'this;

    #[inline]
    fn encode_entry(&mut self, _: &C) -> Result<Self::EncodeEntry<'_>, C::Error> {
        match self._never {}
    }

    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEntryEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeMapKey<'this> = Self where Self: 'this;
    type EncodeMapValue<'this> = Self where Self: 'this;

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

impl<O: 'static, C: ?Sized + Context> MapEntriesEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeMapEntryKey<'this> = Self where Self: 'this;
    type EncodeMapEntryValue<'this> = Self where Self: 'this;

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

impl<O: 'static, C: ?Sized + Context> StructEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeField<'this> = Self where Self: 'this;

    #[inline]
    fn encode_field(&mut self, _: &C) -> Result<Self::EncodeField<'_>, C::Error> {
        match self._never {}
    }

    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> StructFieldEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeFieldName<'this> = Self where Self: 'this;
    type EncodeFieldValue<'this> = Self where Self: 'this;

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

impl<O: 'static, C: ?Sized + Context> VariantEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeTag<'this> = Self where Self: 'this;
    type EncodeValue<'this> = Self where Self: 'this;

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
