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
    AsDecoder, Decode, DecodeUnsized, DecodeUnsizedBytes, Decoder, MapDecoder, MapEntriesDecoder,
    MapEntryDecoder, NumberVisitor, PackDecoder, SequenceDecoder, SizeHint, TupleDecoder,
    ValueVisitor, VariantDecoder,
};
use crate::en::{
    Encode, Encoder, MapEncoder, MapEntriesEncoder, MapEntryEncoder, PackEncoder, SequenceEncoder,
    TupleEncoder, VariantEncoder,
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

    #[inline(always)]
    fn write_fmt(&mut self, _: fmt::Arguments<'_>) -> Result<(), crate::buf::Error> {
        Err(crate::buf::Error)
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
/// use musli::de::{Decoder, Decode};
///
/// struct MyDecoder<'a, C: ?Sized> {
///     cx: &'a C,
///     number: u32,
/// }
///
/// #[musli::decoder]
/// impl<'de, C: ?Sized + Context> Decoder<'de> for MyDecoder<'_, C> where {
///     type Cx = C;
///
///     fn cx(&self) -> &C {
///         self.cx
///     }
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self) -> Result<u32, C::Error> {
///         if self.number == 42 {
///             return Ok(self.number);
///         }
///
///         Err(self.cx.message("I do not have the answer..."))
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
    type WithContext<'this, U> = Never<(), U>
    where
        U: 'this + Context;
    type DecodeBuffer = Self;
    type DecodePack = Self;
    type DecodeSequence = Self;
    type DecodeTuple = Self;
    type DecodeMap = Self;
    type DecodeMapEntries = Self;
    type DecodeSome = Self;
    type DecodeStruct = Self;
    type DecodeUnsizedStruct = Self;
    type DecodeVariant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn cx(&self) -> &Self::Cx {
        match self._never {}
    }

    #[inline]
    fn with_context<U>(self, _: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode>,
    {
        match self._never {}
    }

    #[inline]
    fn decode_unsized<T, F, O>(self, _: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsized<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn decode_unsized_bytes<T, F, O>(self, _: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsizedBytes<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        match self._never {}
    }
}

impl<C: ?Sized + Context> AsDecoder for Never<(), C> {
    type Cx = C;
    type Decoder<'this> = Self where Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapEntriesDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeMapEntryKey<'this> = Self where Self: 'this;
    type DecodeMapEntryValue<'this> = Self where Self: 'this;

    #[inline]
    fn decode_map_entry_key(&mut self) -> Result<Option<Self::DecodeMapEntryKey<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_map_entry_value(&mut self) -> Result<Self::DecodeMapEntryValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end_map_entries(self) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> VariantDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeTag<'this> = Self where Self: 'this;
    type DecodeValue<'this> = Self where Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeEntry<'this> = Self where Self: 'this;
    type DecodeRemainingEntries<'this> = Self where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, <Self::Cx as Context>::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapEntryDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeMapKey<'this> = Self where Self: 'this;
    type DecodeMapValue = Self;

    #[inline]
    fn decode_map_key(&mut self) -> Result<Self::DecodeMapKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_map_value(self) -> Result<Self::DecodeMapValue, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> SequenceDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeNext<'this> = Self where Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> PackDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeNext<'this> = Self where Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> TupleDecoder<'de> for Never<(), C> {
    type Cx = C;
    type DecodeNext<'this> = Self where Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self._never {}
    }
}

impl<C: ?Sized + Context, O: 'static> Encoder for Never<O, C> {
    type Cx = C;
    type Error = C::Error;
    type Ok = O;
    type Mode = C::Mode;
    type WithContext<'this, U> = Never<O, U> where U: 'this + Context;
    type EncodePack = Self;
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
    fn cx(&self) -> &Self::Cx {
        match self._never {}
    }

    #[inline]
    fn with_context<U>(self, _: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }

    #[inline]
    fn encode<T>(self, _: T) -> Result<Self::Ok, C::Error>
    where
        T: Encode<Self::Mode>,
    {
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

impl<O: 'static, C: ?Sized + Context> PackEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodePacked<'this> = Self where Self: 'this;

    #[inline]
    fn encode_packed(&mut self) -> Result<Self::EncodePacked<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_pack(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> SequenceEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeElement<'this> = Self where Self: 'this;

    #[inline]
    fn encode_element(&mut self) -> Result<Self::EncodeElement<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> TupleEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeTupleField<'this> = Self where Self: 'this;

    #[inline]
    fn encode_tuple_field(&mut self) -> Result<Self::EncodeTupleField<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_tuple(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeMapEntry<'this> = Self where Self: 'this;

    #[inline]
    fn encode_map_entry(&mut self) -> Result<Self::EncodeMapEntry<'_>, C::Error> {
        match self._never {}
    }

    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEntryEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeMapKey<'this> = Self where Self: 'this;
    type EncodeMapValue<'this> = Self where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_map_entry(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> MapEntriesEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeMapEntryKey<'this> = Self where Self: 'this;
    type EncodeMapEntryValue<'this> = Self where Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_map_entry_value(&mut self) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_map_entries(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O: 'static, C: ?Sized + Context> VariantEncoder for Never<O, C> {
    type Cx = C;
    type Ok = O;
    type EncodeTag<'this> = Self where Self: 'this;
    type EncodeValue<'this> = Self where Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_variant(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}
