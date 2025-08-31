//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.
//!
//! This is a private module of musli, and is not intended for use outside of
//! the implementation attributes:
//!
//! * [`#[musli::trait_defaults]`][crate::trait_defaults].

use core::fmt;
use core::marker;

use crate::alloc::ToOwned;

use crate::de::{
    AsDecoder, Decode, DecodeUnsized, DecodeUnsizedBytes, Decoder, EntriesDecoder, EntryDecoder,
    MapDecoder, SequenceDecoder, SizeHint, UnsizedVisitor, VariantDecoder,
};
use crate::en::{
    Encode, Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder,
};
use crate::Context;

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
/// struct MyDecoder<C, M> {
///     cx: C,
///     number: u32,
///     _marker: PhantomData<M>,
/// }
///
/// #[musli::trait_defaults]
/// impl<'de, C, M> Decoder<'de> for MyDecoder<C, M>
/// where
///     C: Context,
///     M: 'static,
/// {
///     #[inline]
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     #[inline]
///     fn decode_u32(self) -> Result<u32, Self::Error> {
///         if self.number == 42 {
///             return Ok(self.number);
///         }
///
///         Err(self.cx.message("I do not have the answer..."))
///     }
/// }
/// ```
pub struct Never<T: ?Sized = NeverMarker> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<T>,
}

impl<'de, C, M> Decoder<'de> for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = Self;
    type DecodeBuffer = Self;
    type DecodePack = Self;
    type DecodeSequence = Self;
    type DecodeMapEntries = Self;
    type DecodeSome = Self;
    type DecodeMap = Self;
    type DecodeVariant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        match self._never {}
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode, Self::Allocator>,
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

impl<C, M> AsDecoder for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type Decoder<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, Self::Error> {
        match self._never {}
    }
}

impl<C, M> EntriesDecoder<'_> for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntryKey<'this>
        = Self
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<C, M> VariantDecoder<'_> for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeTag<'this>
        = Self
    where
        Self: 'this;
    type DecodeValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error> {
        match self._never {}
    }
}

impl<C, M> MapDecoder<'_> for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntry<'this>
        = Self
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, Self::Error> {
        match self._never {}
    }
}

impl<C, M> EntryDecoder<'_> for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeKey<'this>
        = Self
    where
        Self: 'this;
    type DecodeValue = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, Self::Error> {
        match self._never {}
    }
}

impl<C, M> SequenceDecoder<'_> for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeNext<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, Self::Error> {
        match self._never {}
    }
}

impl<C, M> Encoder for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodePack = Self;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeVariant = Self;
    type EncodeSequenceVariant = Self;
    type EncodeMapVariant = Self;
    type __UseMusliEncoderAttributeMacro = ();

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }

    #[inline]
    fn encode<T>(self, _: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        match self._never {}
    }
}

impl<C, O, T> UnsizedVisitor<'_, C, T> for Never<(O, T)>
where
    C: Context,
    T: ?Sized + ToOwned,
{
    type Ok = O;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type __UseMusliUnsizedVisitorAttributeMacro = ();

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<C, M> SequenceEncoder for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<C, M> MapEncoder for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        match self._never {}
    }

    fn finish_map(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<C, M> EntryEncoder for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeKey<'this>
        = Self
    where
        Self: 'this;
    type EncodeValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_entry(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<C, M> EntriesEncoder for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = Self
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_entries(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<C, M> VariantEncoder for Never<(C, M)>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeTag<'this>
        = Self
    where
        Self: 'this;
    type EncodeData<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_variant(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}
